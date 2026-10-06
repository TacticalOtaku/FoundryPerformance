import type { AngleBackend, LedState, Settings } from "./types";

export type AccelState = "on" | "software" | "wrongGpu" | "unchecked";

const LED: Record<AccelState, LedState> = { on: "ok", software: "err", wrongGpu: "warn", unchecked: "off" };

/** Статус ускорения для паспорта: вердикт последней проверки, если она про текущие бэкенд и видеокарту. */
export function accelView(settings: Settings, current: boolean): { state: AccelState; api: AngleBackend; led: LedState } {
	const v = current ? settings.gpuCheck?.verdict : undefined;
	const state: AccelState = v?.kind === "hardware" ? "on" : v?.kind === "software" ? "software" : v?.kind === "wrongGpu" ? "wrongGpu" : "unchecked";
	const backend = v && "backend" in v ? v.backend : null;
	return { state, api: backend ?? settings.engine.angle, led: LED[state] };
}
