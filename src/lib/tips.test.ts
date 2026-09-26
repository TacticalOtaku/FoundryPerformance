import { describe, expect, it } from "vitest";
import en from "./i18n/en.json";
import ru from "./i18n/ru.json";
import { type Impact, TIP_IMPACT } from "./tips";

describe("tips", () => {
	it("every tip has RU and EN text", () => {
		for (const key of Object.keys(TIP_IMPACT)) {
			expect(ru, key).toHaveProperty([`tip.${key}`]);
			expect(en, key).toHaveProperty([`tip.${key}`]);
		}
	});

	it("impact meters stay within 0..5", () => {
		for (const [key, v] of Object.entries(TIP_IMPACT) as [string, Impact][]) {
			for (const n of [v.fps, v.look]) if (n !== undefined) expect(n >= 0 && n <= 5, key).toBe(true);
		}
	});
});
