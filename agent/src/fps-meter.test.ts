import { describe, expect, it } from "vitest";
import { FrameStats, low1, summarize } from "./fps-meter";

describe("FrameStats", () => {
	it("keeps the most recent samples in order", () => {
		const s = new FrameStats(3);
		[10, 20, 30, 40].forEach((x) => s.push(x));
		expect(s.recent(3)).toEqual([20, 30, 40]);
		expect(s.recent(2)).toEqual([30, 40]);
		expect(s.size).toBe(3);
	});

	it("averages the last N samples", () => {
		const s = new FrameStats(10);
		[10, 20, 30].forEach((x) => s.push(x));
		expect(s.avgMs(2)).toBe(25);
		expect(s.avgMs(100)).toBe(20);
	});

	it("returns 0 average when empty", () => {
		expect(new FrameStats(4).avgMs(4)).toBe(0);
	});
});

describe("low1 / summarize", () => {
	it("computes 1% low from the slowest frames", () => {
		const samples = Array.from({ length: 100 }, () => 10);
		samples[0] = 50;
		expect(low1(samples)).toBe(20);
	});

	it("summarizes fps from frame times", () => {
		const r = summarize([10, 10, 20, 40]);
		expect(r.avg).toBeCloseTo(1000 / 20, 5);
		expect(r.min).toBeCloseTo(25, 5);
		expect(r.low1).toBeCloseTo(25, 5);
	});

	it("handles empty input", () => {
		expect(summarize([])).toEqual({ avg: 0, low1: 0, min: 0 });
	});
});
