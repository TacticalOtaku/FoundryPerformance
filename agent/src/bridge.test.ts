import { afterEach, describe, expect, it, vi } from "vitest";
import { ipcStatus, send, toggleFullscreen } from "./bridge";

const g = globalThis as Record<string, unknown>;

afterEach(() => {
	delete g.__TAURI_INTERNALS__;
});

describe("send", () => {
	it("invokes report_telemetry when IPC is available", () => {
		const invoke = vi.fn().mockResolvedValue(null);
		g.__TAURI_INTERNALS__ = { invoke };
		send({ kind: "profileChanged", profile: "potato" });
		expect(invoke).toHaveBeenCalledWith("report_telemetry", { report: { kind: "profileChanged", profile: "potato" } });
	});

	it("is a no-op without IPC and swallows rejections", async () => {
		expect(() => send({ kind: "webglLost", early: true })).not.toThrow();
		g.__TAURI_INTERNALS__ = { invoke: vi.fn().mockRejectedValue(new Error("denied")) };
		expect(() => send({ kind: "webglLost", early: true })).not.toThrow();
	});
});

describe("ipcStatus", () => {
	it("reports a missing channel and a host rejection for diagnostics", async () => {
		vi.spyOn(console, "warn").mockImplementation(() => {});
		send({ kind: "webglLost", early: false });
		expect(ipcStatus().state).toBe("missing");
		g.__TAURI_INTERNALS__ = { invoke: vi.fn().mockRejectedValue("rejected") };
		send({ kind: "webglLost", early: false });
		await Promise.resolve();
		expect(ipcStatus()).toEqual({ state: "rejected", error: "rejected" });
		g.__TAURI_INTERNALS__ = { invoke: vi.fn().mockResolvedValue(null) };
		send({ kind: "webglLost", early: false });
		await Promise.resolve();
		expect(ipcStatus().state).toBe("ok");
	});
});

describe("toggleFullscreen", () => {
	it("asks the host to toggle this window", () => {
		const invoke = vi.fn().mockResolvedValue(null);
		g.__TAURI_INTERNALS__ = { invoke };
		toggleFullscreen();
		expect(invoke).toHaveBeenCalledWith("toggle_fullscreen", {});
	});
});
