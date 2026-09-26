import { describe, expect, it } from "vitest";
import { probeUrl, slotStatus } from "./status";
import type { ProbeResult, Server } from "./types";

const srv = (gameUrl: string | null = null) =>
	({ id: "a", name: "A", url: "https://www.sqyre.app/games/x/", profile: null, overrides: {}, gameUrl }) as Server;
const probe = (p: Partial<ProbeResult>): ProbeResult => ({
	reachable: true,
	foundry: false,
	active: false,
	version: null,
	world: null,
	system: null,
	users: null,
	...p
});

describe("slotStatus", () => {
	it("maps Foundry status onto running / idle", () => {
		expect(slotStatus(srv(), probe({ foundry: true, active: true }))).toBe("running");
		expect(slotStatus(srv(), probe({ foundry: true }))).toBe("idle");
	});

	it("hosting page without a known game address is unknown, not an error", () => {
		expect(slotStatus(srv(), probe({ foundry: false }))).toBe("unknown");
	});

	it("known game address that is not Foundry right now means the server is stopped", () => {
		expect(slotStatus(srv("https://x.sqyre.app/"), probe({ foundry: false }))).toBe("stopped");
		expect(slotStatus(srv(), probe({ reachable: false }))).toBe("stopped");
	});

	it("covers the in-between states", () => {
		expect(slotStatus(srv(), undefined)).toBe("unknown");
		expect(slotStatus(srv(), "pending")).toBe("checking");
	});
});

describe("probeUrl", () => {
	it("prefers the discovered game address", () => {
		expect(probeUrl(srv("https://x.sqyre.app/"))).toBe("https://x.sqyre.app/");
		expect(probeUrl(srv())).toBe("https://www.sqyre.app/games/x/");
	});
});
