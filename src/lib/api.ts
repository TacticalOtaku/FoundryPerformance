import { type GpuProbe, probeGpu } from "./gpu";
import type { InstallProgress, ModeDto, ProbeResult, Server, Settings, StateDto, UpdateInfo } from "./types";

/** Typed seam between frontend launcher/setup flows and host runtime (Tauri or in-memory). */
export interface LauncherBackend {
	getState(): Promise<StateDto>;
	saveServer(server: Server): Promise<Server[]>;
	deleteServer(id: string): Promise<Server[]>;
	saveSettings(settings: Settings): Promise<Settings>;
	probe(url: string): Promise<ProbeResult>;
	launch(serverId: string, safeMode: boolean): Promise<void>;
	clearNotice(): Promise<void>;
	cacheSize(): Promise<number>;
	clearCache(): Promise<{ freed: number; pending: boolean }>;
	gpuProbe(): Promise<GpuProbe>;

	getMode(): Promise<ModeDto>;
	pickDir(current: string): Promise<string | null>;
	checkDir(dir: string): Promise<void>;
	install(opts: { dir: string; desktop: boolean; launch: boolean }): Promise<void>;
	openInstalled(): Promise<void>;
	uninstall(wipeData: boolean): Promise<void>;
	onInstallProgress(cb: (p: InstallProgress) => void): Promise<() => void>;

	checkUpdate(timeoutS?: number): Promise<UpdateInfo | null>;
	applyUpdate(): Promise<void>;
	cancelUpdate(): Promise<void>;
	restartToUpdate(): Promise<void>;
	onUpdateProgress(cb: (pct: number) => void): Promise<() => void>;
	splashFlags(): Promise<{ updated: string | null }>;
	splashDone(): Promise<void>;

	windowLabel(): Promise<string>;
	minimize(): Promise<void>;
	toggleMaximize(): Promise<void>;
	onMaximized(cb: (on: boolean) => void): Promise<() => void>;
	close(): Promise<void>;
}

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
	const { invoke } = await import("@tauri-apps/api/core");
	return invoke<T>(cmd, args);
}

async function tauriListen<T>(event: string, cb: (payload: T) => void): Promise<() => void> {
	const { listen } = await import("@tauri-apps/api/event");
	return listen<T>(event, (e) => cb(e.payload));
}

async function currentWindow() {
	const { getCurrentWindow } = await import("@tauri-apps/api/window");
	return getCurrentWindow();
}

export function createTauriBackend(): LauncherBackend {
	return {
		getState: () => tauriInvoke<StateDto>("get_state"),
		saveServer: (server) => tauriInvoke<Server[]>("save_server", { server }),
		deleteServer: (id) => tauriInvoke<Server[]>("delete_server", { id }),
		saveSettings: (settings) => tauriInvoke<Settings>("save_settings", { settings }),
		probe: (url) => tauriInvoke<ProbeResult>("probe_server", { url }),
		launch: (serverId, safeMode) => tauriInvoke<void>("launch", { serverId, safeMode }),
		clearNotice: () => tauriInvoke<void>("clear_notice"),
		cacheSize: () => tauriInvoke<number>("cache_size"),
		clearCache: () => tauriInvoke<{ freed: number; pending: boolean }>("clear_cache"),
		gpuProbe: async () => probeGpu(),

		getMode: () => tauriInvoke<ModeDto>("get_mode"),
		pickDir: (current) => tauriInvoke<string | null>("pick_install_dir", { current }),
		checkDir: (dir) => tauriInvoke<void>("check_install_dir", { dir }),
		install: (opts) => tauriInvoke<void>("install", { opts }),
		openInstalled: () => tauriInvoke<void>("open_installed"),
		uninstall: (wipeData) => tauriInvoke<void>("uninstall", { wipeData }),
		onInstallProgress: (cb) => tauriListen<InstallProgress>("install-progress", cb),

		checkUpdate: (timeoutS) => tauriInvoke<UpdateInfo | null>("check_update", { timeoutS: timeoutS ?? null }),
		applyUpdate: () => tauriInvoke<void>("apply_update"),
		cancelUpdate: () => tauriInvoke<void>("cancel_update"),
		restartToUpdate: () => tauriInvoke<void>("restart_to_update"),
		onUpdateProgress: (cb) => tauriListen<number>("update-progress", cb),
		splashFlags: () => tauriInvoke<{ updated: string | null }>("splash_flags"),
		splashDone: () => tauriInvoke<void>("splash_done"),

		windowLabel: async () => (await currentWindow()).label,
		minimize: async () => void (await currentWindow()).minimize(),
		toggleMaximize: async () => void (await currentWindow()).toggleMaximize(),
		onMaximized: async (cb) => {
			const w = await currentWindow();
			cb(await w.isMaximized());
			return w.onResized(async () => cb(await w.isMaximized()));
		},
		close: async () => void (await currentWindow()).close()
	};
}

function createLazyPreviewBackend(): LauncherBackend {
	let cached: Promise<LauncherBackend> | undefined;
	const load = () => (cached ??= import("./mock").then((m) => m.createPreviewBackend()));
	const call =
		<K extends keyof LauncherBackend>(method: K): LauncherBackend[K] =>
		((...args: unknown[]) => load().then((b) => (b[method] as (...a: unknown[]) => unknown)(...args))) as LauncherBackend[K];

	return {
		getState: call("getState"),
		saveServer: call("saveServer"),
		deleteServer: call("deleteServer"),
		saveSettings: call("saveSettings"),
		probe: call("probe"),
		launch: call("launch"),
		clearNotice: call("clearNotice"),
		cacheSize: call("cacheSize"),
		clearCache: call("clearCache"),
		gpuProbe: call("gpuProbe"),
		getMode: call("getMode"),
		pickDir: call("pickDir"),
		checkDir: call("checkDir"),
		install: call("install"),
		openInstalled: call("openInstalled"),
		uninstall: call("uninstall"),
		onInstallProgress: call("onInstallProgress"),
		checkUpdate: call("checkUpdate"),
		applyUpdate: call("applyUpdate"),
		cancelUpdate: call("cancelUpdate"),
		restartToUpdate: call("restartToUpdate"),
		onUpdateProgress: call("onUpdateProgress"),
		splashFlags: call("splashFlags"),
		splashDone: call("splashDone"),
		windowLabel: call("windowLabel"),
		minimize: call("minimize"),
		toggleMaximize: call("toggleMaximize"),
		onMaximized: call("onMaximized"),
		close: call("close")
	};
}

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export const backend: LauncherBackend = inTauri ? createTauriBackend() : createLazyPreviewBackend();
