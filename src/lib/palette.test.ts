import { describe, expect, it } from "vitest";
import en from "./i18n/en.json";
import ru from "./i18n/ru.json";
import { ACCENTS, accentById, accentStyle, DEFAULT_ACCENT } from "./palette";

describe("palette", () => {
	it("is the Tactile palette from AIM, in picker order", () => {
		expect(ACCENTS.map((a) => a.id)).toEqual(["peach", "amber", "sage", "mint", "azure", "periwinkle", "lavender", "orchid", "rose", "steel"]);
		expect(accentById("peach")).toEqual({ id: "peach", h: 45, c: 0.13 });
		expect(accentById("steel")).toEqual({ id: "steel", h: 250, c: 0.035 });
	});

	it("defaults to peach and falls back to it", () => {
		expect(DEFAULT_ACCENT).toBe("peach");
		expect(accentById("ultraviolet" as never).id).toBe("peach");
	});

	it("renders hue and chroma as root custom properties", () => {
		expect(accentStyle("sage")).toBe("--tc-acc-h: 145; --tc-acc-c: 0.08");
	});

	it("every accent is named in both languages", () => {
		for (const a of ACCENTS) {
			expect(ru).toHaveProperty([`accent.${a.id}`]);
			expect(en).toHaveProperty([`accent.${a.id}`]);
		}
	});
});
