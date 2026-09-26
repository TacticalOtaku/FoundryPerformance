import { describe, expect, it } from "vitest";
import { CORE_KEYS, coreValues, defaultValues, reconcile, type SettingsHost, writePreboot } from "./client-settings";
import type { Levers } from "./types";

const levers: Levers = {
	perfMode: 1,
	maxFps: 60,
	resMin: 0.7,
	resMax: 1,
	adaptive: true,
	pixelRatioScaling: false,
	lightAnimation: true,
	visionAnimation: false,
	mipmap: true,
	video: "pauseUnfocused",
	uiBlur: false,
	sequencer: true,
	fxmaster: true,
	unfocusedFps: 15,
	prime: "medium"
};

function fakeGame(registered: Record<string, unknown>, failing: string[] = [], worldScoped: string[] = []) {
	const values = new Map(Object.entries(registered));
	const sets: string[] = [];
	const game: SettingsHost = {
		settings: {
			settings: new Map(
				[...values.keys()].map((k) => [k, { default: `default-of-${k}`, scope: worldScoped.includes(k) ? "world" : "client" }])
			),
			get: (ns, key) => values.get(`${ns}.${key}`),
			set: async (ns, key, v) => {
				const full = `${ns}.${key}`;
				if (failing.includes(full)) throw new Error("invalid");
				sets.push(full);
				values.set(full, v);
				return v;
			}
		}
	};
	return { game, sets, values };
}

describe("coreValues", () => {
	it("maps levers onto core setting keys", () => {
		const v = coreValues(levers);
		expect(v[CORE_KEYS.perfMode]).toBe(1);
		expect(v[CORE_KEYS.maxFps]).toBe(60);
		expect(v[CORE_KEYS.visionAnimation]).toBe(false);
		expect(Object.keys(v)).toHaveLength(6);
	});

	it("keeps Foundry's own maxFPS setting within its 60 limit", () => {
		expect(coreValues({ ...levers, maxFps: 144 })[CORE_KEYS.maxFps]).toBe(60);
		expect(coreValues({ ...levers, maxFps: 45 })[CORE_KEYS.maxFps]).toBe(45);
	});
});

describe("writePreboot", () => {
	it("stores JSON-encoded values", () => {
		const store = new Map<string, string>();
		writePreboot({ setItem: (k, v) => store.set(k, v) }, { "core.maxFPS": 45, "core.mipmap": false });
		expect(store.get("core.maxFPS")).toBe("45");
		expect(store.get("core.mipmap")).toBe("false");
	});
});

describe("reconcile", () => {
	it("sets only registered keys that differ", async () => {
		const { game, sets } = fakeGame({ "core.maxFPS": 60, "core.mipmap": true });
		const r = await reconcile(game, { "core.maxFPS": 60, "core.mipmap": false, "core.unknown": 1 });
		expect(sets).toEqual(["core.mipmap"]);
		expect(r.applied.sort()).toEqual(["core.maxFPS", "core.mipmap"]);
		expect(r.missing).toEqual(["core.unknown"]);
		expect(r.failed).toEqual([]);
	});

	it("isolates failures per key", async () => {
		const { game } = fakeGame({ "core.maxFPS": 60, "core.mipmap": true }, ["core.maxFPS"]);
		const r = await reconcile(game, { "core.maxFPS": 240, "core.mipmap": false });
		expect(r.failed).toEqual(["core.maxFPS"]);
		expect(r.applied).toEqual(["core.mipmap"]);
	});

	it("never touches world-scoped settings (a GM would change them for everyone)", async () => {
		const { game, sets } = fakeGame({ "fxmaster.disableAll": false, "core.mipmap": true }, [], ["fxmaster.disableAll"]);
		const r = await reconcile(game, { "fxmaster.disableAll": true, "core.mipmap": false });
		expect(sets).toEqual(["core.mipmap"]);
		expect(r.missing).toEqual(["fxmaster.disableAll"]);
	});
});

describe("defaultValues", () => {
	it("returns Foundry defaults for registered keys only", () => {
		const { game } = fakeGame({ "core.maxFPS": 30 });
		expect(defaultValues(game, ["core.maxFPS", "core.nope"])).toEqual({ "core.maxFPS": "default-of-core.maxFPS" });
	});
});
