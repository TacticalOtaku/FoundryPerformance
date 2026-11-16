// @vitest-environment happy-dom
import { afterEach, describe, expect, it, vi } from "vitest";
import type { LauncherBackend } from "./api";
import ru from "./i18n/ru.json";
import { createMemoryBackend, DEFAULT_PRESETS } from "./mock";
import { createAppStore, LEVER_LIMITS } from "./store.svelte";
import type { ProbeResult, Server, StateDto } from "./types";

function makeStore(opts: {
	seed?: Partial<StateDto>;
	backend?: Partial<LauncherBackend>;
	systemLocale?: "ru" | "en";
} = {}) {
	const storageMap = new Map<string, string>();
	const locales: string[] = [];
	const intervals: { ms: number; fn: () => void }[] = [];
	const timeouts: { ms: number; fn: () => void; id: number }[] = [];
	let nextTimeoutId = 1;
	let idSeq = 1;

	const base = createMemoryBackend({ delayMs: 0, seed: opts.seed });
	const backend: LauncherBackend = { ...base, ...opts.backend };

	const store = createAppStore(backend, {
		storage: {
			getItem: (k) => storageMap.get(k) ?? null,
			setItem: (k, v) => void storageMap.set(k, v),
			removeItem: (k) => void storageMap.delete(k)
		},
		systemLocale: () => opts.systemLocale ?? "en",
		setDocumentLocale: (l) => void locales.push(l),
		newId: () => `id-${idSeq++}`,
		wait: async () => {},
		setInterval: (fn, ms) => {
			intervals.push({ ms, fn });
			return intervals.length as unknown as ReturnType<typeof setInterval>;
		},
		clearInterval: () => {},
		setTimeout: (fn, ms) => {
			const id = nextTimeoutId++;
			timeouts.push({ ms, fn, id });
			return id as unknown as ReturnType<typeof setTimeout>;
		},
		clearTimeout: (handle) => {
			const idx = timeouts.findIndex((t) => t.id === (handle as unknown as number));
			if (idx >= 0) timeouts.splice(idx, 1);
		}
	});

	return { store, storageMap, locales, intervals, timeouts, backend };
}

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

describe("AppStore", () => {
	afterEach(() => {
		vi.restoreAllMocks();
	});

	it("load selects lastServer or falls back to first, applies locale, and probes servers", async () => {
		const { store, locales } = makeStore({
			systemLocale: "en",
			seed: {
				settings: {
					schema: 1,
					profile: "balance",
					overrides: {},
					engine: { angle: "d3d11", diskCacheMb: 2048, extraArgs: "" },
					locale: "auto",
					theme: "auto",
					accent: "peach",
					lastServer: "missing"
				},
				servers: [
					{ id: "s1", name: "First", url: "https://one.example.com/", profile: null, overrides: {} },
					{ id: "s2", name: "Second", url: "https://two.example.com/", profile: null, overrides: {} }
				]
			}
		});
		await store.load();
		expect(store.selectedId).toBe("s1");
		expect(locales).toEqual(["en"]);
		await new Promise((r) => setTimeout(r, 0));
		expect(store.probes["s1"]).toMatchObject({ foundry: true, active: true });
	});

	it("loadForSplash loads state and locale without probing servers or scheduling intervals", async () => {
		let probed = false;
		const { store, intervals } = makeStore({
			backend: {
				probe: async () => {
					probed = true;
					return probe({});
				}
			}
		});
		await store.loadForSplash();
		expect(store.dto).not.toBeNull();
		expect(probed).toBe(false);
		expect(intervals).toHaveLength(0);
	});

	it("slotView computes status, tone, detailKey, host, lastRun priority, and intentReason", async () => {
		const sqyre: Server = { id: "s1", name: "Sqyre", url: "https://www.sqyre.app/games/x/", profile: null, overrides: {} };
		const direct: Server = { id: "s2", name: "Direct", url: "https://vtt.example.com:30000/game", profile: "potato", overrides: {} };
		const { store } = makeStore({
			seed: {
				settings: {
					schema: 1,
					profile: "balance",
					overrides: {},
					engine: { angle: "d3d11", diskCacheMb: 2048, extraArgs: "" },
					locale: "ru",
					theme: "auto",
					accent: "peach",
					lastServer: "s1"
				},
				servers: [sqyre, direct],
				stats: {
					s1: {
						lastSession: { avg: 52.4, low1: 31, profile: "balance", at: 1 },
						lastBench: { avg: 60.6, low1: 40, min: 25, profile: "potato", at: 2 }
					}
				}
			},
			backend: {
				probe: async (url) =>
					url.includes("sqyre")
						? probe({ foundry: false, reachable: true })
						: probe({ foundry: true, active: true, world: "strahd" })
			}
		});
		await store.load();
		await Promise.resolve();

		const v1 = store.slotView(sqyre);
		expect(v1.status).toBe("unknown");
		expect(v1.tone).toBe("plain");
		expect(v1.detailKey).toBe("led.unknown");
		expect(v1.host).toBe("www.sqyre.app");
		expect(v1.run).toEqual({ fps: 61, profile: "potato", kind: "bench" });
		expect(v1.reason).toBe("last");

		store.selectedId = "s2";
		const v2 = store.slotView(direct);
		expect(v2.status).toBe("running");
		expect(v2.tone).toBe("good");
		expect(v2.detailKey).toBe("led.ok");
		expect(v2.world).toBe("strahd");
		expect(v2.host).toBe("vtt.example.com:30000");
		expect(v2.reason).toBe("manual");
		expect(ru).toHaveProperty([v1.detailKey]);
		expect(ru).toHaveProperty([v2.detailKey]);

		// Known gameUrl failing Foundry probe is stopped; pending/unreachable/idle/malformed URLs behave cleanly
		store.probes["s1"] = "pending";
		expect(store.slotView(sqyre).status).toBe("checking");
		store.probes["s1"] = probe({ foundry: false, reachable: true });
		expect(store.slotView({ ...sqyre, gameUrl: "https://x.sqyre.app/" }).status).toBe("stopped");
		store.probes["s1"] = probe({ foundry: true, active: false });
		expect(store.slotView(sqyre).status).toBe("idle");
		expect(store.slotView({ ...direct, url: "192.168.1.40:30000" }).host).toBe("192.168.1.40:30000");
		expect(store.slotView({ ...direct, url: "::" }).host).toBe("::");
	});

	it("setKnob stashes and restores manual overrides, opens tuning when empty, and warns on global overrides", async () => {
		const { store } = makeStore({
			seed: {
				settings: {
					schema: 1,
					profile: "balance",
					overrides: { mipmap: false },
					engine: { angle: "d3d11", diskCacheMb: 2048, extraArgs: "" },
					locale: "en",
					theme: "auto",
					accent: "peach",
					lastServer: "s1"
				},
				servers: [{ id: "s1", name: "S1", url: "https://s1.example.com/", profile: null, overrides: { maxFps: 144 } }],
				presets: DEFAULT_PRESETS
			}
		});
		await store.load();
		expect(store.knobPosition).toBe("manual");

		// Switching to preset stashes server overrides and warns about global overrides
		await store.setKnob("potato");
		expect(store.selected?.profile).toBe("potato");
		expect(store.selected?.overrides).toEqual({});
		expect(store.error).toBe("knob.globalOverrides");

		// Switching back to manual restores stashed server overrides
		await store.setKnob("manual");
		expect(store.selected?.overrides).toEqual({ maxFps: 144 });

		// With no stashed overrides, manual opens tuning screen for current scope
		await store.setKnob("manual");
		expect(store.screen).toBe("tuning");
		expect(store.scope).toEqual({ kind: "server", id: "s1" });
	});

	it("unifies lever bounds, float-safe override dropping, adaptive resolution editing, and Rust-matching sanitization", async () => {
		const { store } = makeStore({
			seed: {
				settings: {
					schema: 1,
					profile: "potato",
					overrides: { resMin: 0.9, maxFps: 500 },
					engine: { angle: "d3d11", diskCacheMb: 2048, extraArgs: "" },
					locale: "en",
					theme: "auto",
					accent: "peach",
					lastServer: null
				},
				servers: [{ id: "s1", name: "S1", url: "https://s1.example.com/", profile: null, overrides: {} }],
				presets: DEFAULT_PRESETS
			}
		});
		await store.load();
		expect(LEVER_LIMITS.res.uiMin).toBe(0.4);
		expect(LEVER_LIMITS.res.clampMin).toBe(0.25);

		// Global scope clamps out-of-range JSON values like Rust profile::sanitize
		store.openTuning({ kind: "global" });
		expect(store.tuning.resolved.resMin).toBe(0.85);
		expect(store.tuning.resolved.resMax).toBe(0.9);
		expect(store.tuning.resolved.maxFps).toBe(240);

		// Server scope layers global overrides beneath server overrides and inherits global profile when null
		store.openTuning({ kind: "server", id: "s1" });
		expect(store.tuning.profileValue).toBe("inherit");
		expect(store.tuning.resolved.maxFps).toBe(240);
		await store.setScopeProfile("balance");
		expect(store.tuning.profileValue).toBe("balance");

		// Setting a value equal to base (within float tolerance) removes the override
		store.openTuning({ kind: "global" });
		await store.setLevers({ maxFps: 45 });
		expect(store.dto!.settings.overrides.maxFps).toBeUndefined();

		// Adaptive resolution updates only resMin (clamped <= resMax), static sets both
		await store.setResolution(0.6);
		expect(store.dto!.settings.overrides.resMin).toBe(0.6);
		await store.setLevers({ adaptive: false });
		await store.setResolution(0.75);
		expect(store.dto!.settings.overrides.resMin).toBe(0.75);
		expect(store.dto!.settings.overrides.resMax).toBe(0.75);
	});

	it("deleteServer cleans stats and falls back selection, dismiss clears error before notice, and launch prevents double launch", async () => {
		let destroyed = 0;
		const { store } = makeStore({
			seed: {
				notice: { key: "notice.launchFailed", params: {} },
				servers: [
					{ id: "s1", name: "S1", url: "https://s1.example.com/", profile: null, overrides: {} },
					{ id: "s2", name: "S2", url: "https://s2.example.com/", profile: null, overrides: {} }
				],
				stats: { s1: { lastSession: { avg: 50, low1: 30, profile: "balance", at: 1 }, lastBench: null } }
			},
			backend: {
				destroy: async () => void destroyed++
			}
		});
		await store.load();
		store.error = "err.saveFailed";
		await store.dismiss();
		expect(store.error).toBeNull();
		expect(store.dto?.notice).not.toBeNull();
		await store.dismiss();
		expect(store.dto?.notice).toBeNull();

		await store.deleteServer("s1");
		expect(store.dto?.stats["s1"]).toBeUndefined();
		expect(store.selectedId).toBe("s2");

		await store.launch(false);
		expect(destroyed).toBe(1);
	});

	it("manual checkUpdate deduplicates, retains prior update on network error, and resets indicator after timeout", async () => {
		let shouldFail = false;
		let checkCount = 0;
		let holdResolve: (() => void) | null = null;
		const { store, timeouts } = makeStore({
			backend: {
				checkUpdate: async () => {
					checkCount++;
					if (holdResolve) await new Promise<void>((r) => (holdResolve = r));
					if (shouldFail) throw new Error("offline");
					return { version: "0.3.1", notes: "- Fix" };
				}
			}
		});
		await store.load();
		await new Promise((r) => setTimeout(r, 0));
		expect(store.update).toEqual({ version: "0.3.1", notes: "- Fix" });

		// Concurrent manual check while checking is deduplicated
		const baseCalls = checkCount;
		holdResolve = () => {};
		const first = store.checkUpdate(true);
		const second = store.checkUpdate(true);
		expect(store.updateCheck).toBe("checking");
		expect(checkCount).toBe(baseCalls + 1);
		holdResolve();
		holdResolve = null;
		await Promise.all([first, second]);
		expect(store.updateCheck).toBe("available");

		shouldFail = true;
		await store.checkUpdate(true);
		expect(store.updateCheck).toBe("failed");
		expect(store.update).toEqual({ version: "0.3.1", notes: "- Fix" });
		expect(timeouts.at(-1)?.ms).toBe(5000);
		timeouts.at(-1)?.fn();
		expect(store.updateCheck).toBe("idle");
	});
});
