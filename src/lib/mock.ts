import type { LauncherBackend } from "./api";
import { type GpuProbe, probeGpu } from "./gpu";
import type { InstallProgress, InstallStep, Levers, ModeDto, ProfileId, Server, Settings, StateDto } from "./types";

const lv = (o: Partial<Levers>): Levers => ({
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
	unfocusedFps: 15,
	...o
});

/** Копия пресетов из src-tauri/src/profile.rs — только для превью и тестов. */
export const DEFAULT_PRESETS: Record<ProfileId, Levers> = {
	quality: lv({ perfMode: 2, resMin: 1, adaptive: false, pixelRatioScaling: true, visionAnimation: true, video: "play", uiBlur: true, unfocusedFps: 60 }),
	balance: lv({}),
	potato: lv({ perfMode: 0, maxFps: 45, resMin: 0.55, resMax: 0.85, lightAnimation: false, mipmap: false, video: "static", sequencer: false, fxmaster: false, unfocusedFps: 10 })
};

export interface MemoryBackendOptions {
	delayMs?: number;
	search?: string;
	onNavigateSearch?: (search: string) => void;
	seed?: Partial<StateDto>;
}

const sleep = (ms: number) => (ms > 0 ? new Promise<void>((r) => setTimeout(r, ms)) : Promise.resolve());

/** Typed in-memory adapter implementing LauncherBackend for Vitest and browser preview. */
export function createMemoryBackend(opts: MemoryBackendOptions = {}): LauncherBackend {
	const delayMs = opts.delayMs ?? 0;
	const wait = (ms = delayMs) => sleep(ms);
	const params = () => new URLSearchParams(opts.search ?? "");
	const setSearch = (q: URLSearchParams) => {
		const s = q.toString();
		if (opts.onNavigateSearch) opts.onNavigateSearch(s);
		else if (typeof location !== "undefined") location.search = s;
	};

	const progressListeners = new Set<(p: InstallProgress) => void>();
	const updateListeners = new Set<(pct: number) => void>();
	let cacheBytes = 91_000_000;
	let cancelled = false;

	let state: StateDto = {
		settings: opts.seed?.settings ?? {
			schema: 1,
			profile: "balance",
			overrides: {},
			engine: { angle: "d3d11", diskCacheMb: 2048, extraArgs: "" },
			locale: "auto",
			theme: "auto",
			accent: "peach",
			lastServer: "a1"
		},
		servers: opts.seed?.servers ?? [
			{ id: "a1", name: "Проклятие Страда", url: "https://vtt.example.com/", profile: null, overrides: {} },
			{ id: "a3", name: "Aldarion", url: "https://www.sqyre.app/games/aldarionv210-3c11b9b6/", profile: null, overrides: {} },
			{ id: "a2", name: "Ваншот по пятницам", url: "http://192.168.1.40:30000/", profile: "quality", overrides: { maxFps: 45 } }
		],
		stats: opts.seed?.stats ?? { a1: { lastSession: { avg: 52.4, low1: 31, profile: "balance", at: 0 }, lastBench: null } },
		presets: opts.seed?.presets ?? DEFAULT_PRESETS,
		notice: opts.seed?.notice ?? null,
		locale: opts.seed?.locale ?? "ru",
		version: opts.seed?.version ?? "0.3.0"
	};

	const modeDto = (): ModeDto => {
		const q = params();
		const mode = (q.get("mode") ?? "launcher") as ModeDto["mode"];
		const installState = (q.get("state") ?? "fresh") as "fresh" | "upgrade" | "current";
		return {
			mode,
			install:
				mode === "install"
					? {
							currentVersion: state.version,
							defaultDir: "C:\\Users\\Niki\\AppData\\Local\\Programs\\FoundryPerformance",
							existingDir: installState === "fresh" ? null : "C:\\Users\\Niki\\AppData\\Local\\Programs\\FoundryPerformance",
							existingVersion: installState === "upgrade" ? "0.2.5" : installState === "current" ? state.version : null,
							state: installState
						}
					: null
		};
	};

	const previewGpu = (): GpuProbe | null => {
		switch (params().get("gpu")) {
			case "ok":
				return { webgl2: true, hardware: true };
			case "software":
				return { webgl2: true, hardware: false };
			case "none":
				return { webgl2: false, hardware: false };
			default:
				return null;
		}
	};

	return {
		async getState() {
			await wait();
			return structuredClone(state);
		},
		async saveServer(server) {
			await wait();
			if (!server.name.trim()) throw "err.nameRequired";
			if (!/^(https?:\/\/)?[\w.-]+(:\d+)?(\/.*)?$/.test(server.url.trim())) throw "err.urlInvalid";
			const stored = state.servers.find((x) => x.id === server.id);
			const next: Server = { ...server, gameUrl: stored && stored.url === server.url ? stored.gameUrl : undefined };
			state.servers = stored ? state.servers.map((x) => (x.id === server.id ? next : x)) : [...state.servers, next];
			return structuredClone(state.servers);
		},
		async deleteServer(id) {
			await wait();
			state.servers = state.servers.filter((s) => s.id !== id);
			delete state.stats[id];
			if (state.settings.lastServer === id) state.settings = { ...state.settings, lastServer: null };
			return structuredClone(state.servers);
		},
		async saveSettings(next) {
			await wait();
			state.settings = { ...next, lastServer: state.settings.lastServer };
			return structuredClone(state.settings);
		},
		async probe(url) {
			await wait(delayMs > 0 ? 700 : 0);
			if (url.includes("sqyre.app/games")) return { reachable: true, foundry: true, active: true, version: null, world: null, system: null, users: null };
			return url.includes("192.168")
				? { reachable: false, foundry: false, active: false, version: null, world: null, system: null, users: null }
				: { reachable: true, foundry: true, active: true, version: "14.349", world: "strahd", system: "dnd5e", users: 3 };
		},
		async launch(serverId, safeMode) {
			await wait();
			if (!safeMode) state.settings = { ...state.settings, lastServer: serverId };
		},
		async clearNotice() {
			await wait();
			state.notice = null;
		},
		async cacheSize() {
			await wait();
			return cacheBytes;
		},
		async clearCache() {
			await wait(delayMs > 0 ? 500 : 0);
			const freed = cacheBytes;
			cacheBytes = 0;
			return { freed, pending: false };
		},
		async gpuProbe() {
			return previewGpu() ?? probeGpu();
		},
		async getMode() {
			await wait();
			return modeDto();
		},
		async pickDir() {
			await wait();
			return "D:\\Games\\FoundryPerformance";
		},
		async checkDir(dir) {
			await wait();
			const d = dir.trim();
			if (!/^[a-z]:\\/i.test(d)) throw "install.err.notAbsolute";
			if (/^[a-z]:\\?$/i.test(d)) throw "install.err.root";
			if (/^c:\\windows/i.test(d)) throw "install.err.system";
		},
		async install() {
			const steps: InstallStep[] = ["check", "copy", "shortcuts", "register", "done"];
			const pcts = [5, 30, 60, 85, 100];
			for (let i = 0; i < steps.length; i++) {
				progressListeners.forEach((cb) => cb({ step: steps[i], pct: pcts[i] }));
				await wait(delayMs > 0 ? 450 : 0);
				if (params().get("fail") === steps[i]) throw `install.err.${steps[i]}`;
			}
		},
		async openInstalled() {
			await wait();
		},
		async uninstall() {
			await wait(delayMs > 0 ? 600 : 0);
		},
		async onInstallProgress(cb) {
			progressListeners.add(cb);
			return () => progressListeners.delete(cb);
		},
		async checkUpdate() {
			switch (params().get("update")) {
				case "1":
				case "bad":
					await wait(delayMs > 0 ? 600 : 0);
					return { version: "0.3.1", notes: "- Сплэш обновления\n- Иконка в стиле Tactile" };
				case "fail":
					await wait(delayMs > 0 ? 800 : 0);
					throw "update.err.check";
				default:
					await wait(delayMs > 0 ? 600 : 0);
					return null;
			}
		},
		async applyUpdate() {
			cancelled = false;
			for (let p = 0; p <= 100; p += 5) {
				if (cancelled) throw "update.err.cancelled";
				updateListeners.forEach((cb) => cb(p));
				await wait(delayMs > 0 ? 150 : 0);
			}
			await wait(delayMs > 0 ? 900 : 0);
			if (params().get("update") === "bad") throw "update.err.signature";
		},
		async cancelUpdate() {
			cancelled = true;
		},
		async restartToUpdate() {
			const q = params();
			q.set("splash", "");
			setSearch(q);
		},
		async onUpdateProgress(cb) {
			updateListeners.add(cb);
			return () => updateListeners.delete(cb);
		},
		async splashFlags() {
			return { updated: params().get("update") === "updated" ? "0.3.1" : null };
		},
		async splashDone() {
			const q = params();
			q.delete("splash");
			q.delete("update");
			setSearch(q);
		},
		async windowLabel() {
			return params().has("splash") ? "splash" : "main";
		},
		async minimize() {},
		async toggleMaximize() {},
		async onMaximized() {
			return () => {};
		},
		async close() {}
	};
}

export function createPreviewBackend(): LauncherBackend {
	return createMemoryBackend({
		delayMs: 120,
		search: typeof location !== "undefined" ? location.search : ""
	});
}
