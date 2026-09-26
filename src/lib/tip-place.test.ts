import { describe, expect, it } from "vitest";
import { placeTip } from "./tip-place";

const vp = { w: 900, h: 620 };
const tip = { w: 200, h: 80 };

describe("placeTip", () => {
	it("prefers the space above the target, centered", () => {
		const p = placeTip({ x: 400, y: 300, w: 100, h: 20 }, tip, vp);
		expect(p.side).toBe("top");
		expect(p.y).toBe(300 - 8 - 80);
		expect(p.x).toBe(350);
		expect(p.arrow).toBe(100);
	});

	it("falls back below when there is no room above", () => {
		const p = placeTip({ x: 400, y: 40, w: 100, h: 20 }, tip, vp);
		expect(p.side).toBe("bottom");
		expect(p.y).toBe(40 + 20 + 8);
	});

	it("clamps horizontally and keeps the arrow on the target", () => {
		const left = placeTip({ x: 0, y: 300, w: 40, h: 20 }, tip, vp);
		expect(left.x).toBe(8);
		expect(left.arrow).toBe(12);
		const right = placeTip({ x: 880, y: 300, w: 20, h: 20 }, tip, vp);
		expect(right.x).toBe(900 - 200 - 8);
		expect(right.arrow).toBeLessThanOrEqual(200 - 12);
	});
});
