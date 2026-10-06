import { describe, expect, it } from "vitest";
import { hostOf, initialSelection, intentReason, lastRun, STATUS_DETAIL, STATUS_TONE } from "./intent";
import ru from "./i18n/ru.json";

const servers = [{ id: "a1" }, { id: "a2" }];

describe("intentReason", () => {
	it("last launched vs picked by hand", () => {
		expect(intentReason("a1", "a1")).toBe("last");
		expect(intentReason("a2", "a1")).toBe("manual");
	});

	it("no reason without a selection or without launches", () => {
		expect(intentReason(null, "a1")).toBeNull();
		expect(intentReason("a1", null)).toBeNull();
	});
});

describe("initialSelection", () => {
	it("prefers the last launched server", () => {
		expect(initialSelection(servers, "a2")).toBe("a2");
	});

	it("falls back to the first one", () => {
		expect(initialSelection(servers, "gone")).toBe("a1");
		expect(initialSelection(servers, null)).toBe("a1");
		expect(initialSelection([], "a1")).toBeNull();
	});
});

describe("lastRun", () => {
	const session = { avg: 52.4, low1: 31, profile: "balance" as const, at: 0 };

	it("benchmark wins over a session", () => {
		expect(lastRun({ lastSession: session, lastBench: { ...session, avg: 60.6, min: 20, profile: "potato" } })).toEqual({ fps: 61, profile: "potato", kind: "bench" });
	});

	it("session, rounded", () => {
		expect(lastRun({ lastSession: session, lastBench: null })).toEqual({ fps: 52, profile: "balance", kind: "session" });
	});

	it("nothing measured", () => {
		expect(lastRun(undefined)).toBeNull();
		expect(lastRun({ lastSession: null, lastBench: null })).toBeNull();
	});
});

describe("status", () => {
	it("every status has a tone and a detail string", () => {
		expect(STATUS_TONE).toEqual({ running: "good", idle: "warn", stopped: "danger", unknown: "plain", checking: "plain" });
		for (const k of Object.values(STATUS_DETAIL)) expect(ru).toHaveProperty([k]);
	});
});

describe("hostOf", () => {
	it("keeps host and port only", () => {
		expect(hostOf("https://vtt.example.com:30000/game")).toBe("vtt.example.com:30000");
		expect(hostOf("192.168.1.40:30000")).toBe("192.168.1.40:30000");
		expect(hostOf("https://www.sqyre.app/games/x/")).toBe("www.sqyre.app");
	});

	it("returns garbage as is", () => {
		expect(hostOf("::")).toBe("::");
	});
});
