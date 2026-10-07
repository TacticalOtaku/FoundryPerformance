/**
 * «Цена» каждой настройки для индикаторов в подсказке (0–5):
 * fps — сколько кадров даёт перевод рычага в производительную сторону,
 * look — сколько при этом теряет картинка. Текст подсказки — i18n-ключ `tip.<key>`.
 */
export const TIP_IMPACT = {
	resolution: { fps: 5, look: 3 },
	maxFps: { fps: 2, look: 2 },
	unfocused: { fps: 1, look: 0 },
	perfMode: { fps: 4, look: 3 },
	adaptive: { fps: 4, look: 2 },
	lightAnimation: { fps: 3, look: 2 },
	visionAnimation: { fps: 3, look: 1 },
	mipmap: { fps: 1, look: 2 },
	pixelRatio: { fps: 4, look: 2 },
	uiBlur: { fps: 2, look: 1 },
	sequencer: { fps: 3, look: 3 },
	fxmaster: { fps: 3, look: 3 },
	video: { fps: 3, look: 2 },
	prime: { fps: 3, look: 1 },
	angle: {},
	cache: {},
	cacheClear: {},
	extraArgs: {},
	scope: {},
	profile: {},
	quality: { fps: 0, look: 0 },
	balance: { fps: 3, look: 1 },
	potato: { fps: 5, look: 4 },
	manual: {}
} satisfies Record<string, Impact>;

export interface Impact {
	fps?: number;
	look?: number;
}

export type TipKey = keyof typeof TIP_IMPACT;

export interface TipData {
	title: string;
	body: string;
	/** Пункты списка под текстом — например, изменения версии. */
	lines?: string[];
	/** Проверки с точкой статуса: зелёная — да, красная — нет (значок видеокарты). */
	checks?: { ok: boolean; text: string }[];
	fps?: number;
	look?: number;
}
