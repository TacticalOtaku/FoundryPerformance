import { type GpuProbe, probeGpu } from "./gpu";
import type { InstallProgress, ModeDto, ProbeResult, Server, Settings, StateDto, UpdateInfo } from "./types";

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** В браузерном превью (vite dev без Tauri) команды обслуживает mock.ts. */
async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
	if (inTauri) {
		const { invoke } = await import("@tauri-apps/api/core");
		return invoke<T>(cmd, args);
	}
	const { mockInvoke } = await import("./mock");
	return (await mockInvoke(cmd, args ?? {})) as T;
}

export const api = {
	getState: () => call<StateDto>("get_state"),
	saveServer: (server: Server) => call<Server[]>("save_server", { server }),
	deleteServer: (id: string) => call<Server[]>("delete_server", { id }),
	saveSettings: (settings: Settings) => call<Settings>("save_settings", { settings }),
	probe: (url: string) => call<ProbeResult>("probe_server", { url }),
	launch: (serverId: string, safeMode: boolean) => call<void>("launch", { serverId, safeMode }),
	clearNotice: () => call<void>("clear_notice"),
	cacheSize: () => call<number>("cache_size"),
	clearCache: () => call<{ freed: number; pending: boolean }>("clear_cache")
};

/** Проба видеокарты; в браузерном превью её можно подменить: `?gpu=ok|software|none`. */
export async function gpuProbe(): Promise<GpuProbe> {
	if (!inTauri) {
		const { mockGpu } = await import("./mock");
		const forced = mockGpu();
		if (forced) return forced;
	}
	return probeGpu();
}

export const setupApi = {
	getMode: () => call<ModeDto>("get_mode"),
	pickDir: (current: string) => call<string | null>("pick_install_dir", { current }),
	checkDir: (dir: string) => call<void>("check_install_dir", { dir }),
	install: (opts: { dir: string; desktop: boolean; launch: boolean }) => call<void>("install", { opts }),
	openInstalled: () => call<void>("open_installed"),
	uninstall: (wipeData: boolean) => call<void>("uninstall", { wipeData }),
	/** Подписка на прогресс установки; возвращает функцию отписки. */
	async onProgress(cb: (p: InstallProgress) => void): Promise<() => void> {
		if (inTauri) {
			const { listen } = await import("@tauri-apps/api/event");
			return listen<InstallProgress>("install-progress", (e) => cb(e.payload));
		}
		const { mockOnProgress } = await import("./mock");
		return mockOnProgress(cb);
	}
};

export const updateApi = {
	check: () => call<UpdateInfo | null>("check_update"),
	apply: () => call<void>("apply_update"),
	/** Проценты загрузки; возвращает функцию отписки. */
	async onProgress(cb: (pct: number) => void): Promise<() => void> {
		if (inTauri) {
			const { listen } = await import("@tauri-apps/api/event");
			return listen<number>("update-progress", (e) => cb(e.payload));
		}
		const { mockOnUpdateProgress } = await import("./mock");
		return mockOnUpdateProgress(cb);
	}
};

async function currentWindow() {
	if (!inTauri) return null;
	const { getCurrentWindow } = await import("@tauri-apps/api/window");
	return getCurrentWindow();
}

export const windowControls = {
	minimize: async () => void (await currentWindow())?.minimize(),
	close: async () => void (await currentWindow())?.close(),
	destroy: async () => void (await currentWindow())?.destroy()
};
