import type { ProbeResult, Server } from "./types";

export type SlotStatus = "running" | "idle" | "stopped" | "unknown" | "checking";

/** Статус проверяем по настоящему адресу Foundry, если агент его уже сообщил. */
export function probeUrl(s: Server): string {
	return s.gameUrl ?? s.url;
}

export function slotStatus(server: Server, probe: ProbeResult | "pending" | undefined): SlotStatus {
	if (probe === undefined) return "unknown";
	if (probe === "pending") return "checking";
	if (probe.foundry) return probe.active ? "running" : "idle";
	// Страница хостинга (Sqyre и т.п.) — не Foundry, и это нормально, пока адрес игры не узнан
	if (probe.reachable && !server.gameUrl) return "unknown";
	return "stopped";
}
