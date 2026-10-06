import type { ProfileId } from "./types";

const ru = {
	profile: { quality: "КАЧ", balance: "БАЛ", potato: "КРТ" } as Record<ProfileId, string>,
	profileLong: { quality: "Качество", balance: "Баланс", potato: "Картошка" } as Record<ProfileId, string>,
	base: "БАЗА",
	menuTitle: "ПРОФИЛЬ",
	bench: "ЗАМЕР 30 С",
	benchRunning: "ЗАМЕР… НЕ ТРОГАЙТЕ КАРТУ",
	benchDone: "AVG {avg} · 1% {low} · MIN {min}",
	benchNoScene: "НЕТ АКТИВНОЙ СЦЕНЫ",
	missing: "ПРОПУЩЕНО НАСТРОЕК: {n}",
	diagSaved: "ДИАГНОСТИКА СОХРАНЕНА В ЗАГРУЗКИ",
	gpuSoftware: "Игра рисует без видеокарты. Движок уже переключён — перезапусти игру",
	gpuWrong: "Игра рисует на встроенной видеокарте. Выбери дискретную видеокарту основной в панели NVIDIA или AMD",
	hint: "F9 HUD · SHIFT+F9 МЕНЮ · F10 ДИАГН. · F11 ЭКРАН"
};

export type Strings = typeof ru;

const en: Strings = {
	profile: { quality: "QLT", balance: "BAL", potato: "POT" },
	profileLong: { quality: "Quality", balance: "Balance", potato: "Potato" },
	base: "BASE",
	menuTitle: "PROFILE",
	bench: "BENCH 30 S",
	benchRunning: "BENCHMARK… HANDS OFF THE MAP",
	benchDone: "AVG {avg} · 1% {low} · MIN {min}",
	benchNoScene: "NO ACTIVE SCENE",
	missing: "SETTINGS SKIPPED: {n}",
	diagSaved: "DIAGNOSTICS SAVED TO DOWNLOADS",
	gpuSoftware: "The game is rendering without the GPU. The engine has been switched — restart the game",
	gpuWrong: "The game is rendering on the integrated GPU. Make the discrete GPU the preferred one in the NVIDIA or AMD control panel",
	hint: "F9 HUD · SHIFT+F9 MENU · F10 DIAG · F11 SCREEN"
};

export function strings(locale: "ru" | "en"): Strings {
	return locale === "ru" ? ru : en;
}

export function fmt(s: string, vars: Record<string, string | number>): string {
	return s.replace(/\{(\w+)\}/g, (_, k: string) => String(vars[k] ?? ""));
}
