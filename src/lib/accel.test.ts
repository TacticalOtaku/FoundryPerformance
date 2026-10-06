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
		expect(accelView(settings({ kind: "hardware", backend: "gl" }), true, null)).toEqual({ state: "on", api: "gl", led: "ok", source: "game" });
	});

	it("flags software render and the wrong GPU", () => {
		expect(accelView(settings({ kind: "software" }), true, null)).toEqual({ state: "software", api: "d3d11", led: "err", source: "game" });
		expect(accelView(settings({ kind: "wrongGpu", backend: "d3d11on12" }), true, null)).toEqual({
			state: "wrongGpu",
			api: "d3d11on12",
			led: "warn",
			source: "game"
		});
	});

	it("is unchecked without a check, with a stale one or an unknown verdict", () => {
		const unchecked = { state: "unchecked", api: "d3d11", led: "off", source: null };
		expect(accelView(settings(null), true, null)).toEqual(unchecked);
		expect(accelView(settings({ kind: "hardware", backend: "gl" }), false, null)).toEqual(unchecked);
		expect(accelView(settings({ kind: "unknown" }), true, null)).toEqual(unchecked);
	});

	it("falls back to the configured backend when the renderer did not name one", () => {
		expect(accelView(settings({ kind: "hardware", backend: null }), true, null).api).toBe("d3d11");
	});
});

describe("accelView before launch", () => {
	it("uses the launcher check when the game has none", () => {
		expect(accelView(settings(null), true, { kind: "hardware", backend: "d3d11" })).toEqual({ state: "on", api: "d3d11", led: "ok", source: "launcher" });
		expect(accelView(settings(null), true, { kind: "software" })).toEqual({ state: "software", api: "d3d11", led: "err", source: "launcher" });
	});

	it("treats the launcher on an integrated GPU as available", () => {
		expect(accelView(settings(null), true, { kind: "wrongGpu", backend: "d3d11" }).state).toBe("on");
	});

	it("prefers a current game check and ignores unknown launcher results", () => {
		expect(accelView(settings({ kind: "hardware", backend: "gl" }), true, { kind: "software" }).source).toBe("game");
		expect(accelView(settings({ kind: "hardware", backend: "gl" }), false, { kind: "hardware", backend: "gl" })).toEqual({
			state: "on",
			api: "d3d11",
			led: "ok",
			source: "launcher"
		});
		expect(accelView(settings(null), true, { kind: "unknown" }).state).toBe("unchecked");
	});
});
