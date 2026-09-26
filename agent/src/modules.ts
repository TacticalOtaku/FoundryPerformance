import type { Levers, PrimeLevel } from "./types";

export interface ModuleSetting {
	module: string;
	key: string;
	value: (l: Levers) => unknown;
}

export const MODULE_SETTINGS: ModuleSetting[] = [
	{ module: "sequencer", key: "sequencer.effectsEnabled", value: (l) => l.sequencer },
	{ module: "fxmaster", key: "fxmaster.disableAll", value: (l) => !l.fxmaster }
];

export const PRIME_MODULE = "fvtt-perf-optim";

/** Значения клиентских настроек Prime Performance по уровню. Заполняется в Task 11 по дампу. */
export const PRIME_SETTINGS: Record<PrimeLevel, Record<string, unknown>> = {
	soft: {},
	medium: {},
	aggressive: {}
};

export function moduleValues(isActive: (id: string) => boolean, l: Levers): Record<string, unknown> {
	const out: Record<string, unknown> = {};
	for (const m of MODULE_SETTINGS) if (isActive(m.module)) out[m.key] = m.value(l);
	if (isActive(PRIME_MODULE)) Object.assign(out, PRIME_SETTINGS[l.prime]);
	return out;
}
