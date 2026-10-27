export type ProfileId = "quality" | "balance" | "potato";
export type VideoMode = "play" | "pauseUnfocused" | "static";

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
}

/** Зеркало Rust `agent::Boot`; кладётся в `window.__FP_BOOT__`. */
export interface Boot {
	serverId: string;
	profileId: ProfileId;
	presets: Record<ProfileId, Levers>;
	locale: "ru" | "en";
	/** Базовый замер: рычаги не применяются, core-настройки сбрасываются к умолчаниям Foundry. */
	measureOnly: boolean;
	/** Безопасный режим: агент только убирает свои core-настройки из localStorage и больше ничего не делает. */
	safe: boolean;
}
