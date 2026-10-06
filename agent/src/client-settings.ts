import type { Levers } from "./types";

export const CORE_KEYS = {
	perfMode: "core.performanceMode",
	maxFps: "core.maxFPS",
	pixelRatioScaling: "core.pixelRatioResolutionScaling",
	lightAnimation: "core.lightAnimation",
	visionAnimation: "core.visionAnimation",
	mipmap: "core.mipmap"
} as const;

const FOUNDRY_MAX_FPS = 60;

export function coreValues(l: Levers): Record<string, unknown> {
	return {
		[CORE_KEYS.perfMode]: l.perfMode,
		// Настройка Foundry ограничена 60; потолок выше ставим прямо на тикер (FocusThrottle)
		[CORE_KEYS.maxFps]: Math.min(l.maxFps, FOUNDRY_MAX_FPS),
		[CORE_KEYS.pixelRatioScaling]: l.pixelRatioScaling,
		[CORE_KEYS.lightAnimation]: l.lightAnimation,
		[CORE_KEYS.visionAnimation]: l.visionAnimation,
		[CORE_KEYS.mipmap]: l.mipmap
	};
}

/** Foundry хранит client-настройки в localStorage как JSON под ключом `namespace.key`. */
export function writePreboot(storage: { setItem(k: string, v: string): void }, values: Record<string, unknown>): void {
	for (const [k, v] of Object.entries(values)) storage.setItem(k, JSON.stringify(v));
}

export interface SettingsHost {
	settings: {
		settings: Map<string, { default?: unknown; scope?: string }>;
		get(ns: string, key: string): unknown;
		set(ns: string, key: string, value: unknown): Promise<unknown>;
	};
}

export interface ReconcileResult {
	applied: string[];
	missing: string[];
	failed: string[];
}

export async function reconcile(game: SettingsHost, values: Record<string, unknown>): Promise<ReconcileResult> {
	const result: ReconcileResult = { applied: [], missing: [], failed: [] };
	for (const [full, value] of Object.entries(values)) {
		// Мировые настройки не трогаем никогда: у ГМа это изменило бы игру всем игрокам
		const scope = game.settings.settings.get(full)?.scope;
		if (scope !== "client" && scope !== "user") {
			result.missing.push(full);
			continue;
		}
		const dot = full.indexOf(".");
		const ns = full.slice(0, dot);
		const key = full.slice(dot + 1);
		try {
			if (game.settings.get(ns, key) !== value) await game.settings.set(ns, key, value);
			result.applied.push(full);
		} catch {
			result.failed.push(full);
		}
	}
	return result;
}

export function defaultValues(game: SettingsHost, keys: string[]): Record<string, unknown> {
	const out: Record<string, unknown> = {};
	for (const k of keys) {
		const cfg = game.settings.settings.get(k);
		if (cfg) out[k] = cfg.default;
	}
	return out;
}
