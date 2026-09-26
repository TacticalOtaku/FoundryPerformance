import { describe, expect, it } from "vitest";
import { CanvasResolution } from "./resolution";

describe("CanvasResolution", () => {
	it("sets renderer resolution relative to base and triggers resize", () => {
		const renderer = { resolution: 1.5 };
		const events: string[] = [];
		const r = new CanvasResolution(() => renderer, () => 1.5, { dispatchEvent: (e) => (events.push(e.type), true) });
		expect(r.apply(0.8)).toBe(true);
		expect(renderer.resolution).toBe(1.2);
		expect(events).toEqual(["resize"]);
		expect(r.scale).toBe(0.8);
	});

	it("skips work when already at target and clamps to 0.25", () => {
		const renderer = { resolution: 1 };
		const events: string[] = [];
		const r = new CanvasResolution(() => renderer, () => 1, { dispatchEvent: (e) => (events.push(e.type), true) });
		r.apply(1);
		expect(events).toEqual([]);
		expect(r.targetFor(0.1)).toBe(0.25);
	});

	it("returns false without a renderer", () => {
		const r = new CanvasResolution(() => undefined, () => 1, { dispatchEvent: () => true });
		expect(r.apply(0.5)).toBe(false);
	});
});
