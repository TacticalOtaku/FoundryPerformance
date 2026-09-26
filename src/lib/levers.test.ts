import { describe, expect, it } from "vitest";
import { baseFor, countOverrides, resolved, withOverrides } from "./levers";
import type { Levers, StateDto } from "./types";

const preset = (maxFps: number, mipmap: boolean) => ({ maxFps, mipmap, resMin: 0.7 }) as Levers;

const dto = {
	settings: { profile: "balance", overrides: { maxFps: 30 } },
	servers: [
		{ id: "a", profile: "potato", overrides: { mipmap: true } },
		{ id: "b", profile: null, overrides: {} }
	],
	presets: { quality: preset(60, true), balance: preset(60, true), potato: preset(45, false) }
} as unknown as StateDto;

describe("levers resolution", () => {
	it("global scope = preset + global overrides", () => {
		expect(resolved(dto, { kind: "global" }).maxFps).toBe(30);
		expect(baseFor(dto, { kind: "global" }).maxFps).toBe(60);
	});

	it("server scope = own profile + global overrides + server overrides", () => {
		const r = resolved(dto, { kind: "server", id: "a" });
		expect(r.maxFps).toBe(30);
		expect(r.mipmap).toBe(true);
		expect(baseFor(dto, { kind: "server", id: "a" }).mipmap).toBe(false);
	});

	it("server without profile inherits global profile", () => {
		expect(resolved(dto, { kind: "server", id: "b" }).mipmap).toBe(true);
	});
});

describe("withOverrides", () => {
	it("drops an override equal to the base value (float-safe)", () => {
		const base = preset(60, true);
		expect(withOverrides({ resMin: 0.5 }, base, { resMin: 0.7000000001 })).toEqual({});
	});

	it("adds differing values and keeps others", () => {
		const base = preset(60, true);
		expect(withOverrides({ mipmap: false }, base, { maxFps: 45 })).toEqual({ mipmap: false, maxFps: 45 });
	});

	it("counts overrides", () => {
		expect(countOverrides({ maxFps: 45, mipmap: false })).toBe(2);
		expect(countOverrides({})).toBe(0);
	});
});
