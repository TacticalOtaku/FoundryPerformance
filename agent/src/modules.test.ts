import { describe, expect, it } from "vitest";
import { moduleValues } from "./modules";
import type { Levers } from "./types";

const base = { sequencer: false, fxmaster: false, prime: "aggressive" } as Levers;

describe("moduleValues", () => {
	it("includes settings only for active modules", () => {
		const v = moduleValues((id) => id === "sequencer", base);
		expect(v).toEqual({ "sequencer.effectsEnabled": false });
	});

	it("maps fxmaster onto its client enable flag (v8)", () => {
		expect(moduleValues((id) => id === "fxmaster", { ...base, fxmaster: true })).toEqual({ "fxmaster.enable": true });
		expect(moduleValues((id) => id === "fxmaster", base)).toEqual({ "fxmaster.enable": false });
	});

	it("returns nothing when no modules are active", () => {
		expect(moduleValues(() => false, base)).toEqual({});
	});
});
