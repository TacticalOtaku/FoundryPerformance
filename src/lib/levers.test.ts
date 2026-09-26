import { describe, expect, it } from "vitest";
import { baseFor, countOverrides, knobPosition, resolved, withOverrides } from "./levers";
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

describe("knobPosition", () => {
	const make = (global: object, server: object, profile: string | null) =>
		({
			settings: { profile: "balance", overrides: global },
			servers: [{ id: "a", profile, overrides: server }],
			presets: {}
		}) as unknown as StateDto;

	it("shows the preset when nothing is overridden", () => {
		expect(knobPosition(make({}, {}, "potato"), "a")).toBe("potato");
		expect(knobPosition(make({}, {}, null), "a")).toBe("balance");
		expect(knobPosition(make({}, {}, null), null)).toBe("balance");
	});

	it("shows manual when the server or all servers have manual changes", () => {
		expect(knobPosition(make({}, { maxFps: 144 }, "potato"), "a")).toBe("manual");
		expect(knobPosition(make({ mipmap: false }, {}, "potato"), "a")).toBe("manual");
		expect(knobPosition(make({ mipmap: false }, {}, null), null)).toBe("manual");
	});
});
