import type { ProfileId } from "./types";

export type Report =
	| { kind: "session"; avg: number; low1: number; profile: ProfileId }
	| { kind: "bench"; avg: number; low1: number; min: number; profile: ProfileId }
	| { kind: "profileChanged"; profile: ProfileId }
	| { kind: "webglLost"; early: boolean }
	| { kind: "foundryUrl"; url: string };

/** Единственный канал наружу. Если Tauri не выдал IPC этому origin — тихо ничего не делаем. */
function invoke(cmd: string, args: unknown = {}): void {
	const internals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ as
		| { invoke?: (cmd: string, args: unknown) => Promise<unknown> }
		| undefined;
	if (typeof internals?.invoke !== "function") return;
	try {
		internals.invoke(cmd, args).catch(() => {});
	} catch {
		/* IPC недоступен для этого origin */
	}
}

export function send(report: Report): void {
	invoke("report_telemetry", { report });
}

/** F11: окно, в котором нажали, уходит в полный экран без рамки и обратно. */
export function toggleFullscreen(): void {
	invoke("toggle_fullscreen");
}
