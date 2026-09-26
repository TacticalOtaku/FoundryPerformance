import type { ProbeResult, Server, Settings, StateDto } from "./types";

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
	clearNotice: () => call<void>("clear_notice")
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
