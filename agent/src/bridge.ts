import type { ProfileId } from "./types";

export type Report =
	| { kind: "session"; avg: number; low1: number; profile: ProfileId }
	| { kind: "bench"; avg: number; low1: number; min: number; profile: ProfileId }
	| { kind: "profileChanged"; profile: ProfileId }
	| { kind: "webglLost"; early: boolean }
	| { kind: "foundryUrl"; url: string };

/**
 * Состояние канала к лаунчеру — для диагностики (F10).
 * `missing` на странице самого мира значит, что обновление Tauri сменило внутренний IPC:
 * `__TAURI_INTERNALS__` не публичный API, при обновлении Tauri его надо перепроверять.
 */
export type IpcStatus = { state: "unknown" | "ok" | "missing" | "rejected"; error?: string };

let status: IpcStatus = { state: "unknown" };
let warned = false;

export function ipcStatus(): IpcStatus {
	return status;
}

/** Единственный канал наружу. Если Tauri не выдал IPC этому origin — ничего не ломаем, только запоминаем. */
function invoke(cmd: string, args: unknown = {}): void {
	const internals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ as
		| { invoke?: (cmd: string, args: unknown) => Promise<unknown> }
		| undefined;
	if (typeof internals?.invoke !== "function") {
		status = { state: "missing" };
		if (!warned) {
			warned = true;
			console.warn("[foundry-performance] IPC недоступен: отчёты агента не доходят до лаунчера");
		}
		return;
	}
	const fail = (e: unknown) => (status = { state: "rejected", error: String(e) });
	try {
		internals.invoke(cmd, args).then(() => (status = { state: "ok" }), fail);
	} catch (e) {
		fail(e);
	}
}

export function send(report: Report): void {
	invoke("report_telemetry", { report });
}

/** F11: окно, в котором нажали, уходит в полный экран без рамки и обратно. */
export function toggleFullscreen(): void {
	invoke("toggle_fullscreen");
}
