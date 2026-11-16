import { type AdaptiveConfig, type AdaptiveState, configFor, initAdaptive, resetTimers, stepAdaptive } from "./adaptive";
import type { Report } from "./bridge";
import { clearPreboot, coreValues, defaultValues, moduleValues, reconcile, type SettingsHost, writePreboot } from "./client-settings";
import { FrameStats, summarize } from "./fps-meter";
import type { HudData } from "./hud";
import { fmt, type Strings, strings } from "./i18n";
import { VideoController, type VideoLike } from "./levers/video";
import type { Boot, Levers, ProfileId } from "./types";

export interface HudLike {
	toggle(): void;
	toggleMenu(): void;
	status(text: string): void;
	update(data: HudData): void;
}

export interface AgentHost {
	isFoundry(): boolean;
	pageUrl(): string;
	hasWebgl2(): boolean;
	isFocused(): boolean;
	devicePixelRatio(): number;
	now(): number;
	wait(ms: number): Promise<void>;
	measureRefresh(): Promise<number>;
	storage: {
		getItem(k: string): string | null;
		setItem(k: string, v: string): void;
		removeItem(k: string): void;
	};
	session: {
		getItem(k: string): string | null;
		setItem(k: string, v: string): void;
	};
	setUiBlur(enabled: boolean): void;
	settingsHost(): (SettingsHost & { isModuleActive(id: string): boolean }) | undefined;
	renderer(): { resolution: number; screen?: { width: number; height: number } } | undefined;
	triggerResize(): void;
	ticker(): { maxFPS: number; deltaMS: number; add(cb: () => void): void } | undefined;
	onWebglContextLost(cb: () => void): void;
	videos(): VideoLike[];
	benchScene():
		| {
				rect: { x: number; y: number; width: number; height: number };
				animatePan(opts: { x: number; y: number; scale: number; duration: number }): Promise<unknown>;
		  }
		| undefined;
	createHud(handlers: { onProfile(id: ProfileId): void; onBench(): void }): HudLike;
	send(report: Report): void;
	toggleFullscreen(): void;
	saveDiag(fp: Record<string, unknown>): void;
	every(ms: number, fn: () => void): void;
}

/** Deep orchestrator for Foundry agent lifecycle, lever application, telemetry, and diagnostics. */
export class AgentRuntime {
	private readonly t: Strings;
	private profileId: ProfileId;
	private levers: Levers;
	private refreshHz = Infinity;
	private adaptiveCfg: AdaptiveConfig;
	private adaptive: AdaptiveState;
	private readyAt = 0;
	private skipped = 0;
	private measureOnly: boolean;
	private tickerAttached = false;
	private focused = true;
	private resScale = 1;
	private benchSamples: number[] | null = null;
	private diagRunning = false;
	private hud: HudLike | null = null;
	private readonly stats = new FrameStats(4096);
	private readonly sparkline: number[] = [];
	private readonly video: VideoController;

	constructor(
		private readonly boot: Boot,
		private readonly host: AgentHost
	) {
		this.t = strings(boot.locale);
		this.profileId = boot.profileId;
		this.levers = boot.presets[this.profileId];
		this.measureOnly = boot.measureOnly;
		this.adaptiveCfg = configFor(this.levers);
		this.adaptive = initAdaptive(this.adaptiveCfg);
		this.video = new VideoController(() => (this.measureOnly ? [] : this.host.videos()));
	}

	get safe(): boolean {
		return this.boot.safe;
	}

	onDomReady(): void {
		if (!this.host.isFoundry()) return;
		if (this.boot.safe) {
			try {
				clearPreboot(this.host.storage);
			} catch {
				/* localStorage unavailable */
			}
			return;
		}
		if (!this.measureOnly) {
			this.writePrebootSafe();
			this.host.setUiBlur(this.levers.uiBlur);
		}
		if (!this.host.hasWebgl2() && !this.host.session.getItem("fp.nowebgl")) {
			this.host.session.setItem("fp.nowebgl", "1");
			this.host.send({ kind: "webglLost", early: true });
		}
		void this.host.measureRefresh().then((hz) => {
			this.refreshHz = hz;
			this.adaptiveCfg = configFor(this.levers, hz);
		});
	}

	async onReady(): Promise<void> {
		if (this.boot.safe) return;
		this.readyAt = this.host.now();
		this.host.send({ kind: "foundryUrl", url: this.host.pageUrl() });
		this.video.setMode(this.levers.video);
		if (!this.measureOnly) this.host.setUiBlur(this.levers.uiBlur);
		await this.applySettings();
		this.attachTicker();
		if (!this.measureOnly) this.applyResolution(this.targetScale());
		this.host.every(250, () => this.tick());
		this.host.every(30000, () => this.sessionReport());
		this.host.every(2000, () => this.video.sync());
		if (this.skipped > 0) this.ensureHud().status(fmt(this.t.missing, { n: this.skipped }));
	}

	onCanvasReady(): void {
		if (this.boot.safe) return;
		this.attachTicker();
		if (this.measureOnly) return;
		this.applyFocus();
		this.applyResolution(this.targetScale());
		this.video.sync();
	}

	tick(): void {
		const window1s = Math.max(10, Math.round(this.levers.maxFps));
		const avg = this.stats.avgMs(window1s);
		const focused = this.host.isFocused();
		if (focused !== this.focused) {
			this.focused = focused;
			if (!this.measureOnly) this.applyFocus();
			this.video.setFocused(focused);
			this.adaptive = resetTimers(this.adaptive);
		}
		// Never step adaptive resolution while diag or bench holds the canvas
		if (this.levers.adaptive && !this.measureOnly && focused && avg > 0 && !this.diagRunning && !this.benchSamples) {
			const next = stepAdaptive(this.adaptive, this.adaptiveCfg, avg, this.host.now());
			if (next.scale !== this.adaptive.scale) this.applyResolution(next.scale);
			this.adaptive = next;
		}
		if (avg > 0) {
			this.sparkline.push(1000 / avg);
			if (this.sparkline.length > 40) this.sparkline.shift();
		}
		this.hud?.update({
			fps: avg > 0 ? 1000 / avg : 0,
			low1: summarize(this.stats.recent(window1s * 5)).low1,
			ms: avg,
			res: this.resScale,
			profile: this.measureOnly ? null : this.profileId,
			spark: this.sparkline
		});
	}

	sessionReport(): void {
		if (this.measureOnly || !this.focused || this.stats.size < 60) return;
		const s = summarize(this.stats.recent(Math.round(this.levers.maxFps * 30)));
		this.host.send({ kind: "session", avg: s.avg, low1: s.low1, profile: this.profileId });
	}

	async setProfile(id: ProfileId, persist = true): Promise<void> {
		this.measureOnly = false;
		this.profileId = id;
		this.levers = this.boot.presets[id];
		this.adaptiveCfg = configFor(this.levers, this.refreshHz);
		this.adaptive = initAdaptive(this.adaptiveCfg);
		this.writePrebootSafe();
		this.host.setUiBlur(this.levers.uiBlur);
		this.video.setMode(this.levers.video);
		await this.applySettings();
		// Re-apply after reconcile because core.maxFPS caps ticker.maxFPS at 60
		this.applyFocus();
		this.applyResolution(this.targetScale());
		if (this.skipped > 0) this.hud?.status(fmt(this.t.missing, { n: this.skipped }));
		if (persist) this.host.send({ kind: "profileChanged", profile: id });
	}

	async bench(durationMs = 30000): Promise<void> {
		if (this.benchSamples) return;
		const hud = this.ensureHud();
		const scene = this.host.benchScene();
		if (!scene) {
			hud.status(this.t.benchNoScene);
			return;
		}
		hud.status(this.t.benchRunning);
		const r = scene.rect;
		const at = (fx: number, fy: number, scale: number) => ({ x: r.x + r.width * fx, y: r.y + r.height * fy, scale });
		const route = [at(0.5, 0.5, 0.5), at(0.2, 0.2, 1), at(0.8, 0.2, 1.5), at(0.8, 0.8, 1), at(0.2, 0.8, 0.75), at(0.5, 0.5, 1)];
		const segment = durationMs / route.length;
		this.benchSamples = [];
		let out: { avg: number; low1: number; min: number };
		try {
			for (const p of route) await scene.animatePan({ ...p, duration: segment });
			out = summarize(this.benchSamples);
		} catch {
			hud.status(this.t.benchNoScene);
			return;
		} finally {
			this.benchSamples = null;
		}
		hud.status(fmt(this.t.benchDone, { avg: Math.round(out.avg), low: Math.round(out.low1), min: Math.round(out.min) }));
		if (!this.measureOnly) this.host.send({ kind: "bench", avg: out.avg, low1: out.low1, min: out.min, profile: this.profileId });
	}

	async diag(): Promise<void> {
		if (this.diagRunning) return;
		this.diagRunning = true;
		try {
			await this.diagOnce();
		} finally {
			this.diagRunning = false;
		}
	}

	onKey(e: { key: string; shiftKey: boolean; repeat: boolean; preventDefault(): void; stopPropagation(): void }): void {
		if (e.key !== "F9" && e.key !== "F10" && e.key !== "F11") return;
		e.preventDefault();
		e.stopPropagation();
		if (e.repeat) return;
		if (e.key === "F9" && e.shiftKey) this.ensureHud().toggleMenu();
		else if (e.key === "F9") this.ensureHud().toggle();
		else if (e.key === "F10") void this.diag();
		else this.host.toggleFullscreen();
	}

	toggleHud(): void {
		this.ensureHud().toggle();
	}

	private ensureHud(): HudLike {
		return (this.hud ??= this.host.createHud({
			onProfile: (id) => void this.setProfile(id, true),
			onBench: () => void this.bench()
		}));
	}

	private targetScale(): number {
		return this.levers.adaptive ? this.adaptive.scale : this.levers.resMax;
	}

	private resolutionTargetFor(scale: number): number {
		const base = this.levers.pixelRatioScaling ? this.host.devicePixelRatio() || 1 : 1;
		return Math.max(0.25, Math.round(base * scale * 100) / 100);
	}

	private applyResolution(scale: number): boolean {
		const r = this.host.renderer();
		if (!r) return false;
		this.resScale = scale;
		const target = this.resolutionTargetFor(scale);
		if (Math.abs(r.resolution - target) < 0.005) return true;
		r.resolution = target;
		this.host.triggerResize();
		return true;
	}

	private applyFocus(): void {
		const t = this.host.ticker();
		if (t) t.maxFPS = this.focused ? this.levers.maxFps : this.levers.unfocusedFps;
	}

	private writePrebootSafe(): void {
		try {
			writePreboot(this.host.storage, coreValues(this.levers));
		} catch {
			/* localStorage unavailable — reconcile applies on ready */
		}
	}

	private async applySettings(): Promise<void> {
		const game = this.host.settingsHost();
		if (!game) return;
		const values = this.measureOnly
			? defaultValues(game, Object.keys(coreValues(this.levers)))
			: { ...coreValues(this.levers), ...moduleValues((id) => game.isModuleActive(id), this.levers) };
		const r = await reconcile(game, values);
		this.skipped = r.missing.length + r.failed.length;
		if (this.skipped > 0) console.info("[Foundry Performance] skipped settings", r);
	}

	private attachTicker(): void {
		const ticker = this.host.ticker();
		if (!ticker || this.tickerAttached) return;
		this.tickerAttached = true;
		ticker.add(() => {
			const ms = ticker.deltaMS;
			this.stats.push(ms);
			this.benchSamples?.push(ms);
		});
		this.host.onWebglContextLost(() => this.host.send({ kind: "webglLost", early: this.host.now() - this.readyAt < 60000 }));
		if (!this.measureOnly) this.applyFocus();
	}

	private async diagOnce(): Promise<void> {
		const r = this.host.renderer();
		const before = { scale: this.resScale, rendererResolution: r?.resolution };
		let resolutionTest: unknown = "no-renderer";
		if (r && !this.measureOnly) {
			this.applyResolution(0.8);
			await this.host.wait(700);
			resolutionTest = { target: this.resolutionTargetFor(0.8), after: r.resolution, screen: [r.screen?.width, r.screen?.height] };
			this.applyResolution(before.scale);
		}
		this.host.saveDiag({
			profileId: this.profileId,
			measureOnly: this.measureOnly,
			levers: this.levers,
			adaptive: this.adaptive,
			skipped: this.skipped,
			before,
			resolutionTest,
			tickerAttached: this.tickerAttached,
			bootProfiles: Object.keys(this.boot.presets)
		});
		this.ensureHud().status(this.t.diagSaved);
	}
}
