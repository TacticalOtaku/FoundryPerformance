import { describe, expect, it } from "vitest";
import { accelView } from "./accel";
import type { Settings, Verdict } from "./types";

const settings = (verdict: Verdict | null): Settings => ({
	schema: 1,
	profile: "balance",
	overrides: {},
	engine: { angle: "d3d11", diskCacheMb: 2048, extraArgs: "" },
	locale: "auto",
	theme: "auto",
	gpuCheck: verdict && { verdict, renderer: "r", angle: "d3d11", adapter: "RTX" }
});

describe("accelView", () => {
	it("shows the real backend when the GPU renders", () => {
		expect(accelView(settings({ kind: "hardware", backend: "gl" }), true)).toEqual({ state: "on", api: "gl", led: "ok" });
	});

	it("flags software render and the wrong GPU", () => {
		expect(accelView(settings({ kind: "software" }), true)).toEqual({ state: "software", api: "d3d11", led: "err" });
		expect(accelView(settings({ kind: "wrongGpu", backend: "d3d11on12" }), true)).toEqual({ state: "wrongGpu", api: "d3d11on12", led: "warn" });
	});

	it("is unchecked without a check, with a stale one or an unknown verdict", () => {
		const unchecked = { state: "unchecked", api: "d3d11", led: "off" };
		expect(accelView(settings(null), true)).toEqual(unchecked);
		expect(accelView(settings({ kind: "hardware", backend: "gl" }), false)).toEqual(unchecked);
		expect(accelView(settings({ kind: "unknown" }), true)).toEqual(unchecked);
	});

	it("falls back to the configured backend when the renderer did not name one", () => {
		expect(accelView(settings({ kind: "hardware", backend: null }), true).api).toBe("d3d11");
	});
});
