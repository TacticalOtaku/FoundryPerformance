import { describe, expect, it } from "vitest";
import { moduleValues } from "./modules";
import type { Levers } from "./types";

const base = { sequencer: false, fxmaster: false, prime: "aggressive" } as Levers;

describe("moduleValues", () => {
	it("includes settings only for active modules", () => {
		const v = moduleValues((id) => id === "sequencer", base);
		expect(v).toEqual({ "sequencer.effectsEnabled": false });
	});

	it("inverts fxmaster disable flag", () => {
		const v = moduleValues((id) => id === "fxmaster", { ...base, fxmaster: true });
		expect(v["fxmaster.disableAll"]).toBe(false);
	});

	it("returns nothing when no modules are active", () => {
		expect(moduleValues(() => false, base)).toEqual({});
	});
});
