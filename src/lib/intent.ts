import type { SlotStatus } from "./status";
import type { ProfileId, StateDto } from "./types";

export type IntentReason = "last" | "manual";

/** Почему выбран этот сервер: его запускали последним или его выбрали руками. Без запусков — без причины. */
export function intentReason(selectedId: string | null, lastServer: string | null): IntentReason | null {
	if (!selectedId || !lastServer) return null;
	return selectedId === lastServer ? "last" : "manual";
}

/** При старте выбран последний запущенный сервер, если он ещё в списке, иначе первый. */
export function initialSelection(servers: readonly { id: string }[], lastServer: string | null): string | null {
	return servers.find((s) => s.id === lastServer)?.id ?? servers[0]?.id ?? null;
}

export type Tone = "good" | "warn" | "danger" | "plain";

export const STATUS_TONE: Record<SlotStatus, Tone> = {
	running: "good",
	idle: "warn",
	stopped: "danger",
	unknown: "plain",
	checking: "plain"
};

/** Подробности статуса для подсказки. */
export const STATUS_DETAIL: Record<SlotStatus, string> = {
	running: "led.ok",
	idle: "led.idle",
	stopped: "led.err",
	unknown: "led.unknown",
	checking: "led.pending"
};

export interface LastRun {
	fps: number;
	profile: ProfileId;
	kind: "bench" | "session";
}

/** Последний замер важнее последнего сеанса: он снят в одинаковых условиях. */
export function lastRun(stats: StateDto["stats"][string] | undefined): LastRun | null {
	const b = stats?.lastBench;
	if (b) return { fps: Math.round(b.avg), profile: b.profile, kind: "bench" };
	const s = stats?.lastSession;
	return s ? { fps: Math.round(s.avg), profile: s.profile, kind: "session" } : null;
}

/** «https://vtt.example.com:30000/game» → «vtt.example.com:30000». */
export function hostOf(url: string): string {
	try {
		return new URL(/^https?:\/\//i.test(url) ? url : `http://${url}`).host || url;
	} catch {
		return url;
	}
}
