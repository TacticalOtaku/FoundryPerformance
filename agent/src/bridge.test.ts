import { afterEach, describe, expect, it, vi } from "vitest";
import { send } from "./bridge";

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
