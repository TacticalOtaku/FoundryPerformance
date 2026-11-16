import { describe, expect, it } from "vitest";
import type { Report } from "./bridge";
import type { SettingsHost } from "./client-settings";
import type { HudData } from "./hud";
import type { VideoLike } from "./levers/video";
import { type AgentHost, AgentRuntime } from "./runtime";
import type { Boot, Levers } from "./types";

const quality: Levers = {
	perfMode: 2,
	maxFps: 144,
	resMin: 1,
	resMax: 1,
	adaptive: false,
	pixelRatioScaling: true,
	lightAnimation: true,
	visionAnimation: true,
	mipmap: true,
	video: "play",
	uiBlur: true,
	sequencer: true,
	fxmaster: true,
	unfocusedFps: 60
};

const balance: Levers = {
	perfMode: 1,
	maxFps: 60,
	resMin: 0.7,
	resMax: 1,
	adaptive: true,
	pixelRatioScaling: false,
	lightAnimation: true,
	visionAnimation: false,
	mipmap: true,
	video: "pauseUnfocused",
	uiBlur: false,
	sequencer: true,
	fxmaster: true,
	unfocusedFps: 15
};

const potato: Levers = {
	perfMode: 0,
	maxFps: 45,
	resMin: 0.55,
	resMax: 0.85,
	adaptive: true,
	pixelRatioScaling: false,
	lightAnimation: false,
	visionAnimation: false,
	mipmap: false,
	video: "static",
	uiBlur: false,
	sequencer: false,
	fxmaster: false,
	unfocusedFps: 10
};

function makeBoot(over: Partial<Boot> = {}): Boot {
	return {
		serverId: "s1",
		profileId: "balance",
		presets: { quality, balance, potato },
		locale: "en",
		measureOnly: false,
		safe: false,
		...over
	};
}

function createFakeHost(opts: {
	foundry?: boolean;
	webgl2?: boolean;
	focused?: boolean;
	dpr?: number;
	refreshHz?: number;
	registered?: Record<string, unknown>;
	activeModules?: string[];
	videos?: VideoLike[];
} = {}) {
	const storage = new Map<string, string>();
	const session = new Map<string, string>();
	const reports: Report[] = [];
	const diags: Record<string, unknown>[] = [];
	const intervals: { ms: number; fn: () => void }[] = [];
	const sets: { key: string; value: unknown }[] = [];
	let blurEnabled: boolean | null = null;
	let resizes = 0;
	let fullscreens = 0;
	let timeMs = 0;
	let hudCreated = 0;
	let hudVisible = false;
	let menuVisible = false;
	let lastHudData: HudData | null = null;
	const statuses: string[] = [];
	let contextLostCb: (() => void) | null = null;

	const reg = new Map(
		Object.entries(
			opts.registered ?? {
				"core.performanceMode": 2,
				"core.maxFPS": 60,
				"core.pixelRatioResolutionScaling": true,
				"core.lightAnimation": true,
				"core.visionAnimation": true,
				"core.mipmap": true
			}
		).map(([k, v]) => [k, { default: `default:${k}`, scope: "client", value: v }])
	);

	const ticker = {
		maxFPS: 60,
		deltaMS: 16.6,
		cbs: [] as (() => void)[],
		add(cb: () => void) {
			this.cbs.push(cb);
		},
		pushFrame(ms: number) {
			this.deltaMS = ms;
			for (const cb of this.cbs) cb();
		}
	};

	const renderer = { resolution: 1, screen: { width: 1920, height: 1080 } };
	const active = new Set(opts.activeModules ?? []);
	let focused = opts.focused ?? true;
	let sceneReady = true;
	const panCalls: { x: number; y: number; scale: number; duration: number }[] = [];

	const settingsHost: SettingsHost & { isModuleActive(id: string): boolean } = {
		isModuleActive: (id) => active.has(id),
		settings: {
			settings: new Map([...reg.entries()].map(([k, v]) => [k, { default: v.default, scope: v.scope }])),
			get: (ns, key) => reg.get(`${ns}.${key}`)?.value,
			set: async (ns, key, value) => {
				const full = `${ns}.${key}`;
				sets.push({ key: full, value });
				const cur = reg.get(full);
				if (cur) cur.value = value;
				// Simulate Foundry capping core.maxFPS directly on the ticker
				if (full === "core.maxFPS" && typeof value === "number") ticker.maxFPS = value;
				return value;
			}
		}
	};

	let hudHandlers: { onProfile(id: Boot["profileId"]): void; onBench(): void } | null = null;

	const host: AgentHost = {
		isFoundry: () => opts.foundry ?? true,
		pageUrl: () => "https://vtt.example.com/game",
		hasWebgl2: () => opts.webgl2 ?? true,
		isFocused: () => focused,
		devicePixelRatio: () => opts.dpr ?? 1,
		now: () => timeMs,
		wait: async (ms) => {
			timeMs += ms;
		},
		measureRefresh: async () => opts.refreshHz ?? 60,
		storage: {
			getItem: (k) => storage.get(k) ?? null,
			setItem: (k, v) => void storage.set(k, v),
			removeItem: (k) => void storage.delete(k)
		},
		session: {
			getItem: (k) => session.get(k) ?? null,
			setItem: (k, v) => void session.set(k, v)
		},
		setUiBlur: (enabled) => {
			blurEnabled = enabled;
		},
		settingsHost: () => settingsHost,
		renderer: () => renderer,
		triggerResize: () => {
			resizes++;
		},
		ticker: () => ticker,
		onWebglContextLost: (cb) => {
			contextLostCb = cb;
		},
		videos: () => opts.videos ?? [],
		benchScene: () =>
			sceneReady
				? {
						rect: { x: 0, y: 0, width: 1000, height: 1000 },
						animatePan: async (p) => {
							panCalls.push(p);
							ticker.pushFrame(20);
							ticker.pushFrame(20);
						}
					}
				: undefined,
		createHud: (handlers) => {
			hudCreated++;
			hudHandlers = handlers;
			return {
				toggle: () => {
					hudVisible = !hudVisible;
				},
				toggleMenu: () => {
					hudVisible = true;
					menuVisible = !menuVisible;
				},
				status: (text) => {
					hudVisible = true;
					statuses.push(text);
				},
				update: (d) => {
					lastHudData = d;
				}
			};
		},
		send: (r) => reports.push(r),
		toggleFullscreen: () => {
			fullscreens++;
		},
		saveDiag: (fp) => {
			diags.push(fp);
		},
		every: (ms, fn) => {
			intervals.push({ ms, fn });
		}
	};

	return {
		host,
		storage,
		session,
		reports,
		diags,
		intervals,
		sets,
		ticker,
		renderer,
		panCalls,
		statuses,
		get blurEnabled() {
			return blurEnabled;
		},
		get resizes() {
			return resizes;
		},
		get fullscreens() {
			return fullscreens;
		},
		get hudCreated() {
			return hudCreated;
		},
		get hudVisible() {
			return hudVisible;
		},
		get menuVisible() {
			return menuVisible;
		},
		get lastHudData() {
			return lastHudData;
		},
		setFocused(v: boolean) {
			focused = v;
		},
		setTime(ms: number) {
			timeMs = ms;
		},
		setSceneReady(v: boolean) {
			sceneReady = v;
		},
		triggerContextLost() {
			contextLostCb?.();
		},
		triggerHudProfile(id: Boot["profileId"]) {
			hudHandlers?.onProfile(id);
		}
	};
}

describe("AgentRuntime", () => {
	it("does not touch storage, blur, or WebGL on non-Foundry pages", () => {
		const f = createFakeHost({ foundry: false, webgl2: false });
		f.storage.set("core.mipmap", "true");
		const rt = new AgentRuntime(makeBoot({ safe: false }), f.host);
		rt.onDomReady();
		expect(f.storage.get("core.mipmap")).toBe("true");
		expect(f.blurEnabled).toBeNull();
		expect(f.reports).toEqual([]);
	});

	it("safe mode only removes managed core preboot keys on Foundry pages", () => {
		const f = createFakeHost({ foundry: true, webgl2: false });
		f.storage.set("core.mipmap", "false");
		f.storage.set("core.maxFPS", "45");
		f.storage.set("unrelated.key", "keep");
		const rt = new AgentRuntime(makeBoot({ safe: true }), f.host);
		rt.onDomReady();
		expect(f.storage.has("core.mipmap")).toBe(false);
		expect(f.storage.has("core.maxFPS")).toBe(false);
		expect(f.storage.get("unrelated.key")).toBe("keep");
		expect(f.blurEnabled).toBeNull();
		expect(f.reports).toEqual([]);
	});

	it("writes preboot settings, disables blur, and reports missing WebGL2 once per session", () => {
		const f = createFakeHost({ webgl2: false });
		const rt = new AgentRuntime(makeBoot(), f.host);
		rt.onDomReady();
		expect(JSON.parse(f.storage.get("core.mipmap")!)).toBe(true);
		expect(JSON.parse(f.storage.get("core.visionAnimation")!)).toBe(false);
		expect(f.blurEnabled).toBe(false);
		expect(f.reports).toEqual([{ kind: "webglLost", early: true }]);

		// Second reload in same session does not resend early webglLost
		const rt2 = new AgentRuntime(makeBoot(), f.host);
		rt2.onDomReady();
		expect(f.reports).toHaveLength(1);
	});

	it("measureOnly skips preboot writes and blur CSS on DOM ready", () => {
		const f = createFakeHost({ webgl2: true });
		const rt = new AgentRuntime(makeBoot({ measureOnly: true }), f.host);
		rt.onDomReady();
		expect(f.storage.size).toBe(0);
		expect(f.blurEnabled).toBeNull();
	});

	it("onReady sends foundryUrl, reconciles core and active modules, and sets focus + resolution", async () => {
		const f = createFakeHost({
			activeModules: ["sequencer"],
			registered: {
				"core.performanceMode": 0,
				"core.maxFPS": 30,
				"core.pixelRatioResolutionScaling": false,
				"core.lightAnimation": false,
				"core.visionAnimation": false,
				"core.mipmap": false,
				"sequencer.effectsEnabled": false,
				"fxmaster.enable": false
			},
			dpr: 1.5
		});
		const rt = new AgentRuntime(makeBoot({ profileId: "quality" }), f.host);
		await rt.onReady();
		expect(f.reports).toContainEqual({ kind: "foundryUrl", url: "https://vtt.example.com/game" });
		expect(f.sets).toContainEqual({ key: "sequencer.effectsEnabled", value: true });
		expect(f.sets.find((s) => s.key === "fxmaster.enable")).toBeUndefined();
		// core.maxFPS caps at 60, yet ticker gets 144 after reconcile
		expect(f.sets).toContainEqual({ key: "core.maxFPS", value: 60 });
		expect(f.ticker.maxFPS).toBe(144);
		// dpr 1.5 * resMax 1 = 1.5
		expect(f.renderer.resolution).toBe(1.5);
		expect(f.resizes).toBe(1);
	});

	it("measureOnly onReady resets core settings to defaults and leaves ticker/renderer untouched", async () => {
		const f = createFakeHost({
			registered: {
				"core.performanceMode": 0,
				"core.maxFPS": 45,
				"core.pixelRatioResolutionScaling": false,
				"core.lightAnimation": false,
				"core.visionAnimation": false,
				"core.mipmap": false
			}
		});
		f.ticker.maxFPS = 99;
		f.renderer.resolution = 1.25;
		const rt = new AgentRuntime(makeBoot({ measureOnly: true }), f.host);
		await rt.onReady();
		expect(f.sets).toContainEqual({ key: "core.maxFPS", value: "default:core.maxFPS" });
		expect(f.ticker.maxFPS).toBe(99);
		expect(f.renderer.resolution).toBe(1.25);
		expect(f.resizes).toBe(0);
	});

	it("onCanvasReady re-applies focus ceiling and target resolution after Foundry resets ticker", async () => {
		const f = createFakeHost();
		const rt = new AgentRuntime(makeBoot({ profileId: "quality" }), f.host);
		await rt.onReady();
		f.ticker.maxFPS = 60;
		f.renderer.resolution = 1;
		rt.onCanvasReady();
		expect(f.ticker.maxFPS).toBe(144);
		expect(f.renderer.resolution).toBe(1);
	});

	it("setProfile exits measureOnly, re-applies maxFPS > 60 after core.maxFPS, and persists choice", async () => {
		const f = createFakeHost();
		const rt = new AgentRuntime(makeBoot({ measureOnly: true, profileId: "balance" }), f.host);
		await rt.onReady();
		await rt.setProfile("quality", true);
		expect(f.ticker.maxFPS).toBe(144);
		expect(f.blurEnabled).toBe(true);
		expect(f.reports).toContainEqual({ kind: "profileChanged", profile: "quality" });
	});

	it("tick handles focus loss, video pausing, and adaptive resolution downscaling", async () => {
		const playingVideo: VideoLike = { paused: false, pause() { this.paused = true; }, async play() { this.paused = false; } };
		const userPaused: VideoLike = { paused: true, pause() { this.paused = true; }, async play() { this.paused = false; } };
		const f = createFakeHost({ videos: [playingVideo, userPaused], refreshHz: 60 });
		const rt = new AgentRuntime(makeBoot({ profileId: "balance" }), f.host);
		rt.onDomReady();
		await Promise.resolve();
		await rt.onReady();

		// Lose focus: throttles ticker and pauses only non-paused video
		f.setFocused(false);
		rt.tick();
		expect(f.ticker.maxFPS).toBe(15);
		expect(playingVideo.paused).toBe(true);
		expect(userPaused.paused).toBe(true);

		// Regain focus: restores maxFPS and plays previously paused video
		f.setFocused(true);
		rt.tick();
		expect(f.ticker.maxFPS).toBe(60);
		expect(playingVideo.paused).toBe(false);
		expect(userPaused.paused).toBe(true);

		// Feed sustained slow frames (~33ms = 30fps on 60Hz target) across 2000ms
		for (let i = 0; i < 60; i++) f.ticker.pushFrame(33);
		f.setTime(0);
		rt.tick();
		f.setTime(2000);
		rt.tick();
		expect(f.renderer.resolution).toBe(0.95);
	});

	it("bench guards against concurrent starts, handles missing scene, and reports results", async () => {
		const f = createFakeHost();
		const rt = new AgentRuntime(makeBoot({ profileId: "potato" }), f.host);
		await rt.onReady();

		f.setSceneReady(false);
		await rt.bench(60);
		expect(f.statuses.at(-1)).toBe("NO ACTIVE SCENE");

		f.setSceneReady(true);
		// Start two benches concurrently: second must be ignored
		await Promise.all([rt.bench(60), rt.bench(60)]);
		expect(f.panCalls).toHaveLength(6);
		const benchReports = f.reports.filter((r) => r.kind === "bench");
		expect(benchReports).toHaveLength(1);
		expect(benchReports[0]).toMatchObject({ kind: "bench", profile: "potato", avg: 50 });
	});

	it("diag pauses adaptive stepping during its 700ms resolution probe and restores scale", async () => {
		const f = createFakeHost({ refreshHz: 60 });
		const rt = new AgentRuntime(makeBoot({ profileId: "balance" }), f.host);
		rt.onDomReady();
		await Promise.resolve();
		await rt.onReady();

		for (let i = 0; i < 60; i++) f.ticker.pushFrame(33);
		let resolveWait!: () => void;
		f.host.wait = () => new Promise<void>((r) => (resolveWait = r));

		const running = rt.diag();
		expect(f.renderer.resolution).toBe(0.8);
		// Concurrent diag is ignored
		const ignored = rt.diag();
		// Even if tick fires across >2s while diag holds the canvas, adaptive must not overwrite 0.8
		f.setTime(0);
		rt.tick();
		f.setTime(2500);
		rt.tick();
		expect(f.renderer.resolution).toBe(0.8);

		resolveWait();
		await Promise.all([running, ignored]);
		expect(f.renderer.resolution).toBe(1);
		expect(f.diags).toHaveLength(1);
		expect(f.statuses.at(-1)).toBe("DIAGNOSTICS SAVED TO DOWNLOADS");
	});

	it("onKey ignores key repeat and dispatches F9, Shift+F9, F10, F11", async () => {
		const f = createFakeHost();
		const rt = new AgentRuntime(makeBoot(), f.host);
		await rt.onReady();
		let prevented = 0;
		let stopped = 0;
		const key = (k: string, extra: { shiftKey?: boolean; repeat?: boolean } = {}) => ({
			key: k,
			shiftKey: extra.shiftKey ?? false,
			repeat: extra.repeat ?? false,
			preventDefault: () => void prevented++,
			stopPropagation: () => void stopped++
		});

		rt.onKey(key("F11", { repeat: true }));
		expect(f.fullscreens).toBe(0);
		expect(prevented).toBe(1);

		rt.onKey(key("F11"));
		expect(f.fullscreens).toBe(1);

		rt.onKey(key("F9"));
		expect(f.hudVisible).toBe(true);
		expect(f.menuVisible).toBe(false);

		rt.onKey(key("F9", { shiftKey: true }));
		expect(f.menuVisible).toBe(true);

		rt.onKey(key("a"));
		expect(stopped).toBe(4);
	});

	it("sessionReport emits only when focused, not measureOnly, and >= 60 samples", async () => {
		const f = createFakeHost();
		const rt = new AgentRuntime(makeBoot(), f.host);
		await rt.onReady();
		for (let i = 0; i < 59; i++) f.ticker.pushFrame(20);
		rt.sessionReport();
		expect(f.reports.filter((r) => r.kind === "session")).toHaveLength(0);

		f.ticker.pushFrame(20);
		rt.sessionReport();
		expect(f.reports.filter((r) => r.kind === "session")).toHaveLength(1);
	});

	it("webglcontextlost sends early flag relative to readyAt", async () => {
		const f = createFakeHost();
		const rt = new AgentRuntime(makeBoot(), f.host);
		f.setTime(10_000);
		await rt.onReady();
		f.setTime(50_000);
		f.triggerContextLost();
		f.setTime(80_000);
		f.triggerContextLost();
		expect(f.reports.filter((r) => r.kind === "webglLost")).toEqual([
			{ kind: "webglLost", early: true },
			{ kind: "webglLost", early: false }
		]);
	});

	it("clamps renderer resolution at 0.25 and skips resize when already within tolerance", async () => {
		const f = createFakeHost({ dpr: 1 });
		const extremePotato: Levers = { ...potato, resMin: 0.1, resMax: 0.1, adaptive: false };
		const rt = new AgentRuntime(makeBoot({ profileId: "potato", presets: { quality, balance, potato: extremePotato } }), f.host);
		await rt.onReady();
		expect(f.renderer.resolution).toBe(0.25);
		expect(f.resizes).toBe(1);

		// Calling onCanvasReady again with identical target does not emit a duplicate resize
		rt.onCanvasReady();
		expect(f.resizes).toBe(1);
	});
});
