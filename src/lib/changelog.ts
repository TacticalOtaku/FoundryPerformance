// CHANGELOG.md — единственный источник «что нового»: его раздел встраивается в лаунчер
// (подсказка у версии) и уходит в ленту обновлений (scripts/make-manifest.mjs).
// Только стираемый синтаксис TS: файл импортирует и Node без сборки.

const HEADING = /^##\s+v?(\d+\.\d+\.\d+)(?!\.?\d)/;
const BULLET = /^[-*]\s+/;

/** Пункты раздела `## <version>`; null, если раздела нет или он пуст. */
export function notesFor(md: string, version: string): string[] | null {
	const items: string[] = [];
	let inside = false;
	for (const raw of md.split(/\r?\n/)) {
		const heading = raw.match(HEADING) ?? (raw.startsWith("## ") ? [] : null);
		if (heading) {
			if (inside) break;
			inside = heading[1] === version;
			continue;
		}
		if (!inside || !raw.trim()) continue;
		if (BULLET.test(raw) || items.length === 0) items.push(raw.replace(BULLET, "").trim());
		else items[items.length - 1] += ` ${raw.trim()}`;
	}
	return items.length ? items : null;
}

/** Текст заметок из ленты обновлений → пункты для подсказки. */
export function noteLines(text: string): string[] {
	return text
		.split(/\r?\n/)
		.map((l) => l.replace(BULLET, "").trim())
		.filter(Boolean);
}
