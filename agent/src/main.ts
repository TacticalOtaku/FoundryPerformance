import { configFor, initAdaptive, resetTimers, stepAdaptive } from "./adaptive";
import { runBench } from "./bench";
import { send } from "./bridge";
import { coreValues, defaultValues, reconcile, writePreboot } from "./client-settings";
import { AGENT_VERSION, collectDiag, downloadDiag } from "./diag";
import { g } from "./foundry";
import { FrameStats, summarize } from "./fps-meter";
import { Hud } from "./hud";
import { fmt, strings } from "./i18n";
import { setUiBlur } from "./levers/blur";
import { FocusThrottle } from "./levers/focus";
import { CanvasResolution } from "./levers/resolution";
import { collectVideos, VideoController } from "./levers/video";
import { moduleValues } from "./modules";
import type { Boot, ProfileId } from "./types";

function whenDom(fn: () => void): void {
	if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", fn, { once: true });
	else fn();
}

function waitFor<T>(get: () => T | undefined, fn: (v: T) => void, timeoutMs = 120000): void {
	const started = Date.now();
	const tick = () => {
		const v = get();
		if (v) fn(v);
		else if (Date.now() - started < timeoutMs) setTimeout(tick, 50);
	};
	tick();
}

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

function start(boot: Boot): void {
	const t = strings(boot.locale);
	let profileId: ProfileId = boot.profileId;
	let levers = boot.presets[profileId];
	let adaptiveCfg = configFor(levers);
	let adaptive = initAdaptive(adaptiveCfg);
	let readyAt = 0;
	let skipped = 0;
	let benchSamples: number[] | null = null;
	let tickerAttached = false;
	// Базовый замер: геттеры ниже отдают «ничего», и рычаги становятся no-op
	let measureOnly = boot.measureOnly;
	const stats = new FrameStats(4096);
	const sparkline: number[] = [];

	const resolution = new CanvasResolution(
		() => (measureOnly ? undefined : g.canvas?.app?.renderer),
		() => (levers.pixelRatioScaling ? window.devicePixelRatio || 1 : 1),
		window
	);
	const video = new VideoController(() => (measureOnly ? [] : collectVideos(g.canvas)));
	const focus = new FocusThrottle(() => (measureOnly ? undefined : g.canvas?.app?.ticker), levers.maxFps, levers.unfocusedFps);

	// Фаза 1: до загрузки Foundry
	if (!measureOnly) {
		try {
			writePreboot(localStorage, coreValues(levers));
		} catch {
			/* localStorage недоступен — reconcile применит на ready */
		}
	}
	whenDom(() => {
		if (!measureOnly) setUiBlur(document, levers.uiBlur);
		const probe = document.createElement("canvas").getContext("webgl2");
		if (!probe && !sessionStorage.getItem("fp.nowebgl")) {
			sessionStorage.setItem("fp.nowebgl", "1");
			send({ kind: "webglLost", early: true });
		}
	});

	let hud: Hud | null = null;
	const ensureHud = () => (hud ??= new Hud(t, { onProfile: (id) => void setProfile(id, true), onBench: () => void bench() }));

	window.addEventListener(
		"keydown",
		(e) => {
			if (e.key === "F9" && e.shiftKey) ensureHud().toggleMenu();
			else if (e.key === "F9") ensureHud().toggle();
			else if (e.key === "F10") void diag();
			else return;
			e.preventDefault();
			e.stopPropagation();
		},
		true
	);

	const targetScale = () => (levers.adaptive ? adaptive.scale : levers.resMax);

	async function applySettings(): Promise<void> {
		if (!g.game?.settings) return;
		const isActive = (id: string) => Boolean(g.game.modules?.get(id)?.active);
		const values = measureOnly
			? defaultValues(g.game, Object.keys(coreValues(levers)))
			: { ...coreValues(levers), ...moduleValues(isActive, levers) };
		const r = await reconcile(g.game, values);
		skipped = r.missing.length + r.failed.length;
		if (skipped > 0) console.info("[Foundry Performance] skipped settings", r);
	}

	function attachTicker(): void {
		const ticker = g.canvas?.app?.ticker;
		if (!ticker || tickerAttached) return;
		tickerAttached = true;
		ticker.add(() => {
			const ms = ticker.deltaMS;
			stats.push(ms);
			benchSamples?.push(ms);
		});
		const view = g.canvas.app.renderer?.view ?? g.canvas.app.canvas;
		view?.addEventListener?.("webglcontextlost", () => send({ kind: "webglLost", early: performance.now() - readyAt < 60000 }));
		focus.apply();
	}

	function loop(): void {
		const window1s = Math.max(10, Math.round(levers.maxFps));
		const avg = stats.avgMs(window1s);
		if (levers.adaptive && !measureOnly && focus.focused && avg > 0) {
			const next = stepAdaptive(adaptive, adaptiveCfg, avg, performance.now());
			if (next.scale !== adaptive.scale) resolution.apply(next.scale);
			adaptive = next;
		}
		if (avg > 0) {
			sparkline.push(1000 / avg);
			if (sparkline.length > 40) sparkline.shift();
		}
		hud?.update({
			fps: avg > 0 ? 1000 / avg : 0,
			low1: summarize(stats.recent(window1s * 5)).low1,
			ms: avg,
			res: resolution.scale,
			profile: measureOnly ? null : profileId,
			spark: sparkline
		});
	}

	function sessionReport(): void {
		if (measureOnly || !focus.focused || stats.size < 60) return;
		const s = summarize(stats.recent(Math.round(levers.maxFps * 30)));
		send({ kind: "session", avg: s.avg, low1: s.low1, profile: profileId });
	}

	function onFocusChange(focused: boolean): void {
		focus.setFocused(focused);
		video.setFocused(focused);
		adaptive = resetTimers(adaptive);
	}

	async function setProfile(id: ProfileId, persist: boolean): Promise<void> {
		measureOnly = false;
		profileId = id;
		levers = boot.presets[id];
		adaptiveCfg = configFor(levers);
		adaptive = initAdaptive(adaptiveCfg);
		try {
			writePreboot(localStorage, coreValues(levers));
		} catch {
			/* см. фазу 1 */
		}
		setUiBlur(document, levers.uiBlur);
		video.setMode(levers.video);
		focus.setFps(levers.maxFps, levers.unfocusedFps);
		await applySettings();
		resolution.apply(targetScale());
		if (skipped > 0) hud?.status(fmt(t.missing, { n: skipped }));
		if (persist) send({ kind: "profileChanged", profile: id });
	}

	async function bench(): Promise<void> {
		const h = ensureHud();
		if (!g.canvas?.ready || !g.canvas?.dimensions) {
			h.status(t.benchNoScene);
			return;
		}
		h.status(t.benchRunning);
		const recorder = {
			start: () => {
				benchSamples = [];
			},
			stop: () => {
				const s = benchSamples ?? [];
				benchSamples = null;
				return s;
			}
		};
		let out: { avg: number; low1: number; min: number };
		try {
			out = await runBench(g.canvas, recorder);
		} catch {
			benchSamples = null;
			h.status(t.benchNoScene);
			return;
		}
		h.status(fmt(t.benchDone, { avg: Math.round(out.avg), low: Math.round(out.low1), min: Math.round(out.min) }));
		if (!measureOnly) send({ kind: "bench", avg: out.avg, low1: out.low1, min: out.min, profile: profileId });
	}

	async function diag(): Promise<void> {
		const r = g.canvas?.app?.renderer;
		const before = { scale: resolution.scale, rendererResolution: r?.resolution };
		let resolutionTest: unknown = "no-renderer";
		if (r && !measureOnly) {
			resolution.apply(0.8);
			await sleep(700);
			resolutionTest = { target: resolution.targetFor(0.8), after: r.resolution, screen: [r.screen?.width, r.screen?.height] };
			resolution.apply(before.scale);
		}
		downloadDiag(
			collectDiag({
				fp: { profileId, measureOnly, levers, adaptive, skipped, before, resolutionTest, tickerAttached, bootProfiles: Object.keys(boot.presets) }
			})
		);
		ensureHud().status(t.diagSaved);
	}

	waitFor(
		() => g.Hooks,
		(hooks) => {
			hooks.once("ready", async () => {
				readyAt = performance.now();
				// Настоящий адрес Foundry: у хостингов он отличается от страницы входа.
				// Без query — там бывают токены сессии.
				send({ kind: "foundryUrl", url: location.origin + location.pathname });
				video.setMode(levers.video);
				await applySettings();
				attachTicker();
				resolution.apply(targetScale());
				window.addEventListener("blur", () => onFocusChange(false));
				window.addEventListener("focus", () => onFocusChange(true));
				document.addEventListener("visibilitychange", () => onFocusChange(!document.hidden));
				setInterval(loop, 250);
				setInterval(sessionReport, 30000);
				setInterval(() => video.sync(), 2000);
				if (skipped > 0) ensureHud().status(fmt(t.missing, { n: skipped }));
			});
			hooks.on("canvasReady", () => {
				attachTicker();
				resolution.apply(targetScale());
				video.sync();
			});
		}
	);

	g.__FP__ = {
		version: AGENT_VERSION,
		toggleHud: () => ensureHud().toggle(),
		setProfile: (id: ProfileId) => setProfile(id, true),
		bench,
		diag
	};
}

const boot = g.__FP_BOOT__ as Boot | undefined;
if (boot && !g.__FP__) start(boot);
