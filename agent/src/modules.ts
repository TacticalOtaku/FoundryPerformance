import type { Levers } from "./types";

interface ModuleSetting {
	module: string;
	key: string;
	value: (l: Levers) => unknown;
}

const MODULE_SETTINGS: ModuleSetting[] = [
	{ module: "sequencer", key: "sequencer.effectsEnabled", value: (l) => l.sequencer },
	// FXMaster 8.x: клиентская `enable`; `disableAll` — мировая, её не трогаем
	{ module: "fxmaster", key: "fxmaster.enable", value: (l) => l.fxmaster }
];

export function moduleValues(isActive: (id: string) => boolean, l: Levers): Record<string, unknown> {
	const out: Record<string, unknown> = {};
	for (const m of MODULE_SETTINGS) if (isActive(m.module)) out[m.key] = m.value(l);
	return out;
}
