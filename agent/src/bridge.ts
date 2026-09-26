import type { ProfileId } from "./types";

export type Report =
	| { kind: "session"; avg: number; low1: number; profile: ProfileId }
	| { kind: "bench"; avg: number; low1: number; min: number; profile: ProfileId }
	| { kind: "profileChanged"; profile: ProfileId }
	| { kind: "webglLost"; early: boolean };

/** Единственный канал наружу. Если Tauri не выдал IPC этому origin — тихо ничего не делаем. */
export function send(report: Report): void {
	const internals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ as
		| { invoke?: (cmd: string, args: unknown) => Promise<unknown> }
		| undefined;
	if (typeof internals?.invoke !== "function") return;
	try {
		internals.invoke("report_telemetry", { report }).catch(() => {});
	} catch {
		/* IPC недоступен для этого origin */
	}
}
