import { refreshFromDeltas } from "./adaptive";
import { ipcStatus, send, toggleFullscreen } from "./bridge";
import { AGENT_VERSION, collectDiag, downloadDiag } from "./diag";
import { g } from "./foundry";
import { Hud } from "./hud";
import { strings } from "./i18n";
import { collectVideos } from "./levers/video";
import { type AgentHost, AgentRuntime } from "./runtime";
import type { Boot, ProfileId } from "./types";

const NO_BLUR_ID = "fp-no-blur";
const NO_BLUR_CSS = "*,*::before,*::after{backdrop-filter:none!important;-webkit-backdrop-filter:none!important}";

function applyUiBlur(enabled: boolean): void {
	const existing = document.getElementById(NO_BLUR_ID);
	if (enabled) {
		existing?.remove();
		return;
	}
	if (existing) return;
	const style = document.createElement("style");
	style.id = NO_BLUR_ID;
	style.textContent = NO_BLUR_CSS;
	(document.head ?? document.documentElement).appendChild(style);
}

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

/** ~40 кадров requestAnimationFrame → частота монитора (браузер не рисует чаще). */
function measureRefresh(frames = 40): Promise<number> {
	return new Promise((resolve) => {
		const deltas: number[] = [];
		let last = 0;
		const tick = (t: number) => {
			if (last) deltas.push(t - last);
			last = t;
			if (deltas.length < frames) requestAnimationFrame(tick);
			else resolve(refreshFromDeltas(deltas));
		};
		requestAnimationFrame(tick);
	});
}

function createBrowserHost(locale: Boot["locale"]): AgentHost {
	const t = strings(locale);
	return {
		isFoundry: () => Boolean(g.foundry || g.Hooks),
		pageUrl: () => location.origin + location.pathname,
		hasWebgl2: () => Boolean(document.createElement("canvas").getContext("webgl2")),
		isFocused: () => document.hasFocus() && !document.hidden,
		devicePixelRatio: () => window.devicePixelRatio || 1,
		now: () => performance.now(),
		wait: (ms) => new Promise((r) => setTimeout(r, ms)),
		measureRefresh: () => measureRefresh(),
		storage: localStorage,
		session: sessionStorage,
		setUiBlur: applyUiBlur,
		settingsHost: () => {
			const game = g.game;
			if (!game?.settings) return undefined;
			return {
				settings: game.settings,
				isModuleActive: (id: string) => Boolean(game.modules?.get(id)?.active)
			};
		},
		renderer: () => g.canvas?.app?.renderer,
		triggerResize: () => void window.dispatchEvent(new Event("resize")),
		ticker: () => g.canvas?.app?.ticker,
		onWebglContextLost: (cb) => {
			const view = g.canvas?.app?.renderer?.view ?? g.canvas?.app?.canvas;
			view?.addEventListener?.("webglcontextlost", cb);
		},
		videos: () => collectVideos(g.canvas),
		benchScene: () => {
			const canvas = g.canvas;
			if (!canvas?.ready || !canvas?.dimensions) return undefined;
			const rect = canvas.dimensions.sceneRect ?? canvas.dimensions.rect;
			return {
				rect,
				animatePan: (opts) => canvas.animatePan(opts)
			};
		},
		createHud: (handlers) => new Hud(t, handlers),
		send,
		toggleFullscreen,
		saveDiag: (fp) => downloadDiag(collectDiag({ fp: { ...fp, ipc: ipcStatus() } })),
		every: (ms, fn) => void setInterval(fn, ms)
	};
}

const boot = g.__FP_BOOT__ as Boot | undefined;
// Окно игры открывает и чужие сайты — им незачем видеть id сервера и профили
delete g.__FP_BOOT__;

if (boot && (boot.safe || !g.__FP__)) {
	const runtime = new AgentRuntime(boot, createBrowserHost(boot.locale));
	whenDom(() => runtime.onDomReady());
	if (!runtime.safe) {
		waitFor(
			() => g.Hooks,
			(hooks) => {
				window.addEventListener("keydown", (e) => runtime.onKey(e), true);
				hooks.once("ready", () => void runtime.onReady());
				hooks.on("canvasReady", () => runtime.onCanvasReady());
			}
		);
		g.__FP__ = {
			version: AGENT_VERSION,
			toggleHud: () => runtime.toggleHud(),
			setProfile: (id: ProfileId) => runtime.setProfile(id, true),
			bench: () => runtime.bench(),
			diag: () => runtime.diag()
		};
	}
}
