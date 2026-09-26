import { describe, expect, it } from "vitest";
import { type AdaptiveConfig, configFor, initAdaptive, refreshFromDeltas, resetTimers, stepAdaptive } from "./adaptive";
import type { Levers } from "./types";

const cfg: AdaptiveConfig = {
	min: 0.7,
	max: 1,
	step: 0.05,
	targetMs: 1000 / 60,
	overRatio: 1.15,
	stableRatio: 1.05,
	downAfterMs: 2000,
	upAfterMs: 5000,
	cooldownMs: 1000,
	probeBackoffMs: 30000
};
const SLOW = 30;
const OK = 16.7;

function run(state = initAdaptive(cfg), from: number, to: number, ms: number) {
	let s = state;
	for (let t = from; t <= to; t += 250) s = stepAdaptive(s, cfg, ms, t);
	return s;
}

describe("adaptive resolution", () => {
	it("starts at max", () => {
		expect(initAdaptive(cfg).scale).toBe(1);
	});

	it("ignores short spikes", () => {
		expect(run(undefined, 0, 1500, SLOW).scale).toBe(1);
	});

	it("steps down after sustained slow frames", () => {
		expect(run(undefined, 0, 2000, SLOW).scale).toBe(0.95);
	});

	it("never goes below min", () => {
		expect(run(undefined, 0, 60000, SLOW).scale).toBe(0.7);
	});

	it("steps up after stable frames and never above max", () => {
		const low = run(undefined, 0, 60000, SLOW);
		const s = run(low, 60250, 65250, OK);
		expect(s.scale).toBe(0.75);
		expect(run(low, 60250, 400000, OK).scale).toBe(1);
	});

	it("blocks a scale that failed right after a probe", () => {
		let s = run(undefined, 0, 2000, SLOW); // 0.95 at t=2000
		s = run(s, 2250, 7250, OK); // probe up to 1.0 at t=7250
		expect(s.scale).toBe(1);
		s = run(s, 7500, 9500, SLOW); // fails, back to 0.95
		expect(s.scale).toBe(0.95);
		s = run(s, 9750, 30000, OK); // stable but 1.0 is blocked
		expect(s.scale).toBe(0.95);
		s = run(s, 30250, 45000, OK); // backoff expired
		expect(s.scale).toBe(1);
	});

	it("resetTimers clears pending windows", () => {
		const s = resetTimers(run(undefined, 0, 1500, SLOW));
		expect(s.overSince).toBeNull();
		expect(s.stableSince).toBeNull();
	});
});

describe("frame target respects the monitor", () => {
	const l = { maxFps: 144, resMin: 0.7, resMax: 1 } as Levers;

	it("never targets more frames than the display can show", () => {
		expect(configFor(l, 60).targetMs).toBeCloseTo(1000 / 60, 5);
		expect(configFor(l, 165).targetMs).toBeCloseTo(1000 / 144, 5);
		expect(configFor({ ...l, maxFps: 45 }, 144).targetMs).toBeCloseTo(1000 / 45, 5);
	});

	it("estimates refresh rate from animation frame intervals", () => {
		expect(refreshFromDeltas([16.6, 16.7, 16.8, 33.4, 16.7, 16.6, 16.7])).toBe(60);
		expect(refreshFromDeltas([6.9, 7.0, 6.9, 6.95, 13.9, 6.9])).toBe(144);
		expect(refreshFromDeltas([])).toBe(60);
	});
});
