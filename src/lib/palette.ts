import type { AccentId } from "./types";

export interface Accent {
	id: AccentId;
	h: number;
	c: number;
}

/** Палитра Tactile — копия `scripts/tactile/palette.js` из AIM 1.8.0; порядок — как в выборе акцента. */
export const ACCENTS: readonly Accent[] = [
	{ id: "peach", h: 45, c: 0.13 },
	{ id: "amber", h: 78, c: 0.12 },
	{ id: "sage", h: 145, c: 0.08 },
	{ id: "mint", h: 178, c: 0.09 },
	{ id: "azure", h: 235, c: 0.1 },
	{ id: "periwinkle", h: 275, c: 0.1 },
	{ id: "lavender", h: 300, c: 0.1 },
	{ id: "orchid", h: 330, c: 0.11 },
	{ id: "rose", h: 10, c: 0.11 },
	{ id: "steel", h: 250, c: 0.035 }
];

export const DEFAULT_ACCENT: AccentId = "peach";

export const accentById = (id: AccentId): Accent => ACCENTS.find((a) => a.id === id) ?? ACCENTS[0];

/** Оттенок и насыщенность акцента для `style` корня; светлоту даёт тема. */
export function accentStyle(id: AccentId): string {
	const a = accentById(id);
	return `--tc-acc-h: ${a.h}; --tc-acc-c: ${a.c}`;
}
