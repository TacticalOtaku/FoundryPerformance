export type ProfileId = "quality" | "balance" | "potato";
export type VideoMode = "play" | "pauseUnfocused" | "static";
export type PrimeLevel = "soft" | "medium" | "aggressive";

/** Зеркало Rust `model::Levers` (camelCase). */
export interface Levers {
	perfMode: 0 | 1 | 2 | 3;
	maxFps: number;
	resMin: number;
	resMax: number;
	adaptive: boolean;
	pixelRatioScaling: boolean;
	lightAnimation: boolean;
	visionAnimation: boolean;
	mipmap: boolean;
	video: VideoMode;
	uiBlur: boolean;
	sequencer: boolean;
	fxmaster: boolean;
	unfocusedFps: number;
	prime: PrimeLevel;
}

/** Зеркало Rust `agent::Boot`; кладётся в `window.__FP_BOOT__`. */
export interface Boot {
	serverId: string;
	profileId: ProfileId;
	presets: Record<ProfileId, Levers>;
	locale: "ru" | "en";
	/** Базовый замер: рычаги не применяются, core-настройки сбрасываются к умолчаниям Foundry. */
	measureOnly: boolean;
}

/** Зеркало Rust `model::Verdict` — ответ лаунчера на отчёт `gpu`. */
export type Verdict =
	| { kind: "hardware"; backend: string | null }
	| { kind: "software" }
	| { kind: "wrongGpu"; backend: string | null }
	| { kind: "unknown" };
