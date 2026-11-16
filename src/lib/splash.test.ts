import { describe, expect, it } from "vitest";
import type { LauncherBackend } from "./api";
import { createMemoryBackend } from "./mock";
import { canSkip, CHECK_TIMEOUT_S, type Phase, ringMode, SplashFlow } from "./splash";

const flush = () => new Promise((r) => setTimeout(r, 0));

function deferred<T>() {
	let resolve!: (v: T) => void;
	let reject!: (e: unknown) => void;
	const promise = new Promise<T>((a, b) => {
		resolve = a;
		reject = b;
	});
	return { promise, resolve, reject };
}

function harness(over: Partial<LauncherBackend> = {}) {
	const shown: Phase[] = [];
	const calls: string[] = [];
	let progress: (pct: number) => void = () => {};
	const base = createMemoryBackend({ delayMs: 0 });
	const backend: LauncherBackend = {
		...base,
		onUpdateProgress: async (cb) => {
			progress = cb;
			return () => void calls.push("off");
		},
		cancelUpdate: async () => void calls.push("cancel"),
		splashDone: async () => void calls.push("done"),
		...over
	};
	const flow = new SplashFlow(backend, (p) => shown.push(p), async (ms) => void calls.push(`wait ${ms}`));
	return { flow, shown, calls, progress: (pct: number) => progress(pct), last: () => shown.at(-1) };
}

const found = { version: "0.3.1", notes: "" };

describe("SplashFlow", () => {
	it("no update: straight to the launcher with splash timeout", async () => {
		const seenTimeouts: (number | undefined)[] = [];
		const h = harness({
			checkUpdate: async (t) => {
				seenTimeouts.push(t);
				return null;
			}
		});
		await h.flow.run();
		expect(seenTimeouts).toEqual([CHECK_TIMEOUT_S]);
		expect(h.shown).toEqual([{ kind: "checking" }]);
		expect(h.calls).toEqual(["done"]);
	});

	it("offline: says so, waits, then opens launcher", async () => {
		const h = harness({ checkUpdate: async () => Promise.reject("update.err.check") });
		await h.flow.run();
		expect(h.last()).toEqual({ kind: "offline" });
		expect(h.calls).toEqual(["wait 1500", "done"]);
	});

	it("after an update: shows the new version without checking again", async () => {
		let checked = false;
		const h = harness({
			splashFlags: async () => ({ updated: "0.3.1" }),
			checkUpdate: async () => {
				checked = true;
				return null;
			}
		});
		await h.flow.run();
		expect(h.shown).toEqual([{ kind: "updated", version: "0.3.1" }]);
		expect(h.calls).toEqual(["wait 600", "done"]);
		expect(checked).toBe(false);
	});

	it("found: download -> verify at 100 % -> restart, never closes itself", async () => {
		const apply = deferred<void>();
		const h = harness({ checkUpdate: async () => found, applyUpdate: () => apply.promise });
		const running = h.flow.run();
		await flush();
		expect(h.last()).toEqual({ kind: "downloading", version: "0.3.1", pct: null });
		h.progress(40);
		expect(h.last()).toEqual({ kind: "downloading", version: "0.3.1", pct: 40 });
		h.progress(100);
		expect(h.last()).toEqual({ kind: "verifying", version: "0.3.1" });
		apply.resolve();
		await running;
		expect(h.last()).toEqual({ kind: "restarting", version: "0.3.1" });
		expect(h.calls).toEqual(["off"]);
	});

	it("failed verification: error screen waits for a click", async () => {
		const h = harness({ checkUpdate: async () => found, applyUpdate: async () => Promise.reject("update.err.signature") });
		await h.flow.run();
		expect(h.last()).toEqual({ kind: "error", key: "update.err.signature" });
		expect(h.calls).not.toContain("done");
		await h.flow.close();
		expect(h.calls.at(-1)).toBe("done");
	});

	it("skip while downloading: cancels, closes once, shows no error", async () => {
		const apply = deferred<void>();
		const h = harness({ checkUpdate: async () => found, applyUpdate: () => apply.promise });
		const running = h.flow.run();
		await flush();
		h.progress(30);
		await h.flow.skip();
		await h.flow.skip();
		apply.reject("update.err.cancelled");
		await running;
		expect(h.calls.filter((c) => c === "cancel" || c === "done")).toEqual(["cancel", "done"]);
		expect(h.last()).toEqual({ kind: "downloading", version: "0.3.1", pct: 30 });
	});

	it("skip while checking: a late update is not installed", async () => {
		const check = deferred<typeof found | null>();
		let applied = false;
		const h = harness({
			checkUpdate: () => check.promise,
			applyUpdate: async () => {
				applied = true;
			}
		});
		const running = h.flow.run();
		await flush();
		await h.flow.skip();
		check.resolve(found);
		await running;
		expect(applied).toBe(false);
		expect(h.calls).toEqual(["cancel", "done"]);
	});
});

describe("ringMode / canSkip", () => {
	it("maps phases to ring modes", () => {
		expect(ringMode({ kind: "checking" })).toBe("spin");
		expect(ringMode({ kind: "downloading", version: "1", pct: null })).toBe("spin");
		expect(ringMode({ kind: "downloading", version: "1", pct: 40 })).toBe("progress");
		expect(ringMode({ kind: "verifying", version: "1" })).toBe("breathe");
		expect(ringMode({ kind: "restarting", version: "1" })).toBe("breathe");
		expect(ringMode({ kind: "offline" })).toBe("muted");
		expect(ringMode({ kind: "error", key: "x" })).toBe("error");
		expect(ringMode({ kind: "updated", version: "1" })).toBe("full");
	});

	it("skip only before verification", () => {
		expect(canSkip({ kind: "checking" })).toBe(true);
		expect(canSkip({ kind: "downloading", version: "1", pct: 99 })).toBe(true);
		expect(canSkip({ kind: "verifying", version: "1" })).toBe(false);
		expect(canSkip({ kind: "error", key: "x" })).toBe(false);
	});
});
