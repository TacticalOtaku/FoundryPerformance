import en from "./i18n/en.json";
import ru from "./i18n/ru.json";

const dicts: Record<"ru" | "en", Record<string, string>> = { ru, en };
let current = $state<"ru" | "en">("ru");

export function setLocale(l: "ru" | "en"): void {
	current = l;
	document.documentElement.lang = l;
}

export function locale(): "ru" | "en" {
	return current;
}

export function t(key: string, vars?: Record<string, string | number>): string {
	const s = dicts[current][key] ?? dicts.ru[key] ?? key;
	return vars ? s.replace(/\{(\w+)\}/g, (_, k: string) => String(vars[k] ?? "")) : s;
}
