import type { AngleBackend, LedState, Settings, Verdict } from "./types";

export type AccelState = "on" | "software" | "wrongGpu" | "unchecked";
export type AccelSource = "game" | "launcher" | null;

const LED: Record<AccelState, LedState> = { on: "ok", software: "err", wrongGpu: "warn", unchecked: "off" };
const STATE: Record<Verdict["kind"], AccelState> = { hardware: "on", software: "software", wrongGpu: "wrongGpu", unknown: "unchecked" };

/**
 * Статус ускорения для паспорта. Проверка из игры точнее (флаги игры) — она главная, если актуальна;
 * иначе — WebGL лаунчера: он рисует без --force-high-performance-gpu, поэтому встройка у него законна.
 */
export function accelView(
	settings: Settings,
	current: boolean,
	launcher: Verdict | null
): { state: AccelState; api: AngleBackend; led: LedState; source: AccelSource } {
	const game = current ? settings.gpuCheck?.verdict : undefined;
	if (game && game.kind !== "unknown") {
		const state = STATE[game.kind];
		const backend = "backend" in game ? game.backend : null;
		return { state, api: backend ?? settings.engine.angle, led: LED[state], source: "game" };
	}
	if (launcher && launcher.kind !== "unknown") {
		const state: AccelState = launcher.kind === "software" ? "software" : "on";
		return { state, api: settings.engine.angle, led: LED[state], source: "launcher" };
	}
	return { state: "unchecked", api: settings.engine.angle, led: LED.unchecked, source: null };
}
