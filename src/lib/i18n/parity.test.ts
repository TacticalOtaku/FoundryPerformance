import { describe, expect, it } from "vitest";
import en from "./en.json";
import ru from "./ru.json";

const vars = (s: string) => [...s.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();

describe("i18n dictionaries", () => {
	it("have identical keys", () => {
		expect(Object.keys(en).sort()).toEqual(Object.keys(ru).sort());
	});

	it("use the same placeholders", () => {
		for (const k of Object.keys(ru) as (keyof typeof ru)[]) expect(vars(en[k]), k).toEqual(vars(ru[k]));
	});
});
