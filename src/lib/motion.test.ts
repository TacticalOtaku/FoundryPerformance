import { describe, expect, it } from "vitest";
import { SETTLE, settlePoints } from "./motion";

describe("SETTLE", () => {
	it("starts at 0, ends at 1 and settles from a small overshoot", () => {
		const pts = settlePoints();
		expect(pts[0]).toEqual([0, 0]);
		const [x1, y1] = pts[pts.length - 1];
		expect(x1).toBeCloseTo(1);
		expect(y1).toBeCloseTo(1);
		const peak = Math.max(...pts.map(([, y]) => y));
		expect(peak).toBeCloseTo(1.02, 2);
	});

	it("time only moves forward", () => {
		const xs = settlePoints().map(([x]) => x);
		for (let i = 1; i < xs.length; i++) expect(xs[i]).toBeGreaterThan(xs[i - 1]);
	});

	it("is a CSS linear() easing", () => {
		expect(SETTLE).toMatch(/^linear\(0 0%, .+, 1 100%\)$/);
	});
});
