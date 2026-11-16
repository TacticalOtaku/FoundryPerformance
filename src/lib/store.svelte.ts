import { backend as defaultBackend, type LauncherBackend } from "./api";
import type { GpuProbe } from "./gpu";
import { setLocale, t } from "./i18n.svelte";
import type { Levers, Overrides, ProbeResult, ProfileId, Server, Settings, StateDto, UpdateInfo } from "./types";

type Screen = "main" | "tuning" | "slot";
type UpdateCheck = "idle" | "checking" | "current" | "available" | "failed";

export type SlotStatus = "running" | "idle" | "stopped" | "unknown" | "checking";
export type Tone = "good" | "warn" | "danger" | "plain";
export type IntentReason = "last" | "manual";
export type KnobPosition = ProfileId | "manual";
export type Scope = { kind: "global" } | { kind: "server"; id: string };

export interface LastRun {
	fps: number;
	profile: ProfileId;
	kind: "bench" | "session";
}

export interface SlotView {
	status: SlotStatus;
	tone: Tone;
	detailKey: string;
	world: string;
	host: string;
	run: LastRun | null;
	reason: IntentReason | null;
}

export interface TuningView {
	scope: Scope;
	server: Server | null;
	scopeServer: Server | null;
	resolved: Levers;
	overrides: Overrides;
	overrideCount: number;
	profileValue: ProfileId | "inherit";
}

/** Single source of truth for UI slider bounds and Rust-matching sanitization limits. */
export const LEVER_LIMITS = {
	perfMode: { max: 3 },
	res: { clampMin: 0.25, clampMax: 1, uiMin: 0.4, uiMax: 1, step: 0.05 },
	maxFps: { clampMin: 10, clampMax: 240, uiMin: 20, uiMax: 240, step: 1, ticks: [60, 144, 240], presets: [60, 144, 240] },
	unfocusedFps: { clampMin: 5, clampMax: 240, uiMin: 5, uiMax: 60, step: 5 }
} as const;

export const PROFILE_POSITIONS: readonly KnobPosition[] = ["quality", "balance", "potato", "manual"];

export const profileTipKey = (p: KnobPosition): string => `tip.${p}`;

const STATUS_TONE: Record<SlotStatus, Tone> = {
	running: "good",
	idle: "warn",
	stopped: "danger",
	unknown: "plain",
	checking: "plain"
};

const STATUS_DETAIL: Record<SlotStatus, string> = {
	running: "led.ok",
	idle: "led.idle",
	stopped: "led.err",
	unknown: "led.unknown",
	checking: "led.pending"
};

const UPDATE_EVERY_MS = 30 * 60_000;
const PROBE_EVERY_MS = 60_000;

const snap = <T>(v: T): T => $state.snapshot(v) as T;
const clamp = (v: number, lo: number, hi: number) => Math.min(Math.max(v, lo), hi);
const sameValue = (a: unknown, b: unknown) =>
	typeof a === "number" && typeof b === "number" ? Math.abs(a - b) < 1e-6 : a === b;

/** Зеркало Rust `profile::sanitize`: иначе экран настройки показал бы не то, что получит игра. */
function sanitizeLevers(l: Levers): Levers {
	const a = clamp(l.resMin, LEVER_LIMITS.res.clampMin, LEVER_LIMITS.res.clampMax);
	const b = clamp(l.resMax, LEVER_LIMITS.res.clampMin, LEVER_LIMITS.res.clampMax);
	return {
		...l,
		perfMode: Math.min(l.perfMode, LEVER_LIMITS.perfMode.max) as Levers["perfMode"],
		maxFps: clamp(l.maxFps, LEVER_LIMITS.maxFps.clampMin, LEVER_LIMITS.maxFps.clampMax),
		unfocusedFps: clamp(l.unfocusedFps, LEVER_LIMITS.unfocusedFps.clampMin, LEVER_LIMITS.unfocusedFps.clampMax),
		resMin: Math.min(a, b),
		resMax: Math.max(a, b)
	};
}

function applyOverrides(base: Levers, o: Overrides): Levers {
	const defined = Object.entries(o).filter(([, v]) => v !== undefined && v !== null);
	return sanitizeLevers({ ...base, ...Object.fromEntries(defined) } as Levers);
}

function countOverrides(o: Overrides): number {
	return Object.values(o).filter((v) => v !== undefined && v !== null).length;
}

function withOverrides(o: Overrides, base: Levers, patch: Partial<Levers>): Overrides {
	const next: Record<string, unknown> = { ...o };
	for (const [k, v] of Object.entries(patch)) {
		if (sameValue(base[k as keyof Levers], v)) delete next[k];
		else next[k] = v;
	}
	return next as Overrides;
}

function slotStatusOf(server: Server, probe: ProbeResult | "pending" | undefined): SlotStatus {
	if (probe === undefined) return "unknown";
	if (probe === "pending") return "checking";
	if (probe.foundry) return probe.active ? "running" : "idle";
	// Страница хостинга (Sqyre и т.п.) — не Foundry, и это нормально, пока адрес игры не узнан
	if (probe.reachable && !server.gameUrl) return "unknown";
	return "stopped";
}

function hostOf(url: string): string {
	try {
		return new URL(/^https?:\/\//i.test(url) ? url : `http://${url}`).host || url;
	} catch {
		return url;
	}
}

function lastRunOf(stats: StateDto["stats"][string] | undefined): LastRun | null {
	const b = stats?.lastBench;
	if (b) return { fps: Math.round(b.avg), profile: b.profile, kind: "bench" };
	const s = stats?.lastSession;
	return s ? { fps: Math.round(s.avg), profile: s.profile, kind: "session" } : null;
}

export interface StorageLike {
	getItem(k: string): string | null;
	setItem(k: string, v: string): void;
	removeItem(k: string): void;
}

export interface StoreEnv {
	storage?: StorageLike;
	systemLocale?: () => "ru" | "en";
	setDocumentLocale?: (l: "ru" | "en") => void;
	newId?: () => string;
	wait?: (ms: number) => Promise<void>;
	setInterval?: (fn: () => void, ms: number) => ReturnType<typeof setInterval>;
	clearInterval?: (id: ReturnType<typeof setInterval>) => void;
	setTimeout?: (fn: () => void, ms: number) => ReturnType<typeof setTimeout>;
	clearTimeout?: (id: ReturnType<typeof setTimeout>) => void;
}

const savedKey = (id: string) => `fp.lastOverrides.${id}`;

export class AppStore {
	dto = $state<StateDto | null>(null);
	selectedId = $state<string | null>(null);
	screen = $state<Screen>("main");
	editing = $state<Server | null>(null);
	scope = $state<Scope>({ kind: "global" });
	probes = $state<Record<string, ProbeResult | "pending">>({});
	/** WebGL2 и аппаратное ускорение в окне лаунчера (null — проба ещё не прошла). */
	gpu = $state<GpuProbe | null>(null);
	error = $state<string | null>(null);
	launching = $state(false);
	/** Доступное обновление (null — нет или проверка не удалась). */
	update = $state<UpdateInfo | null>(null);
	/** Итог ручной проверки обновлений; через несколько секунд снова idle. */
	updateCheck = $state<UpdateCheck>("idle");

	#backend: LauncherBackend;
	#storage: StorageLike | undefined;
	#systemLocale: () => "ru" | "en";
	#setDocumentLocale: (l: "ru" | "en") => void;
	#newId: () => string;
	#wait: (ms: number) => Promise<void>;
	#setInterval: (fn: () => void, ms: number) => ReturnType<typeof setInterval>;
	#clearInterval: (id: ReturnType<typeof setInterval>) => void;
	#setTimeout: (fn: () => void, ms: number) => ReturnType<typeof setTimeout>;
	#clearTimeout: (id: ReturnType<typeof setTimeout>) => void;
	#checkReset: ReturnType<typeof setTimeout> | undefined;
	#intervals: ReturnType<typeof setInterval>[] = [];

	constructor(backend: LauncherBackend, env: StoreEnv = {}) {
		this.#backend = backend;
		this.#storage = env.storage ?? (typeof localStorage !== "undefined" ? localStorage : undefined);
		this.#systemLocale = env.systemLocale ?? (() => (typeof navigator !== "undefined" && navigator.language.toLowerCase().startsWith("ru") ? "ru" : "en"));
		this.#setDocumentLocale = env.setDocumentLocale ?? setLocale;
		this.#newId = env.newId ?? (() => crypto.randomUUID());
		this.#wait = env.wait ?? ((ms) => new Promise((r) => setTimeout(r, ms)));
		this.#setInterval = env.setInterval ?? ((fn, ms) => globalThis.setInterval(fn, ms));
		this.#clearInterval = env.clearInterval ?? ((id) => globalThis.clearInterval(id));
		this.#setTimeout = env.setTimeout ?? ((fn, ms) => globalThis.setTimeout(fn, ms));
		this.#clearTimeout = env.clearTimeout ?? ((id) => globalThis.clearTimeout(id));
	}

	get selected(): Server | null {
		return this.dto?.servers.find((s) => s.id === this.selectedId) ?? null;
	}

	get message(): string | null {
		if (this.error) return t(this.error);
		const n = this.dto?.notice;
		return n ? t(n.key, n.params) : null;
	}

	get knobPosition(): KnobPosition {
		const dto = this.dto!;
		const s = this.selected;
		const manual = countOverrides(dto.settings.overrides) + countOverrides(s?.overrides ?? {}) > 0;
		return manual ? "manual" : (s?.profile ?? dto.settings.profile);
	}

	#scopeContext() {
		const dto = this.dto!;
		const scope = this.scope;
		const server = scope.kind === "server" ? (dto.servers.find((s) => s.id === scope.id) ?? null) : null;
		const preset = dto.presets[server?.profile ?? dto.settings.profile];
		const base = scope.kind === "global" ? preset : applyOverrides(preset, dto.settings.overrides);
		const overrides = scope.kind === "global" ? dto.settings.overrides : (server?.overrides ?? {});
		return { dto, scope, server, base, overrides };
	}

	get tuning(): TuningView {
		const { dto, scope, server, base, overrides } = this.#scopeContext();
		const selected = dto.servers.find((s) => s.id === this.selectedId) ?? null;
		return {
			scope,
			server,
			scopeServer: server ?? selected,
			resolved: applyOverrides(base, overrides),
			overrides,
			overrideCount: countOverrides(overrides),
			profileValue: server ? (server.profile ?? "inherit") : dto.settings.profile
		};
	}

	slotView(s: Server): SlotView {
		const dto = this.dto!;
		const probe = this.probes[s.id];
		const status = slotStatusOf(s, probe);
		const lastServer = dto.settings.lastServer;
		const reason: IntentReason | null = !lastServer ? null : s.id === lastServer ? "last" : "manual";
		return {
			status,
			tone: STATUS_TONE[status],
			detailKey: STATUS_DETAIL[status],
			world: probe && probe !== "pending" ? (probe.world ?? "") : "",
			host: hostOf(s.url),
			run: lastRunOf(dto.stats[s.id]),
			reason
		};
	}

	async load(): Promise<void> {
		const dto = await this.#backend.getState();
		this.dto = dto;
		this.applyLocale();
		void this.#backend.gpuProbe().then((p) => (this.gpu = p)).catch(() => {});
		this.selectedId = dto.servers.find((s) => s.id === dto.settings.lastServer)?.id ?? dto.servers[0]?.id ?? null;
		for (const s of dto.servers) void this.probe(s);
		void this.checkUpdate();
		this.#intervals.push(this.#setInterval(() => void this.checkUpdate(), UPDATE_EVERY_MS));
		// Пока лаунчер открыт, кружки сами обновляются — видно, когда ГМ запустил сервер
		this.#intervals.push(
			this.#setInterval(() => {
				for (const s of this.dto?.servers ?? []) void this.probe(s, true);
			}, PROBE_EVERY_MS)
		);
	}

	/** Сплэш: только тема, акцент и язык — без проб серверов и фоновой проверки обновлений. */
	async loadForSplash(): Promise<void> {
		this.dto = await this.#backend.getState();
		this.applyLocale();
	}

	dispose(): void {
		for (const id of this.#intervals) this.#clearInterval(id);
		this.#intervals = [];
		if (this.#checkReset !== undefined) this.#clearTimeout(this.#checkReset);
		this.#checkReset = undefined;
	}

	private applyLocale(): void {
		const pref = this.dto?.settings.locale ?? "auto";
		this.#setDocumentLocale(pref === "auto" ? this.#systemLocale() : pref);
	}

	/** `quiet` — фоновая перепроверка без мигания «проверка…». */
	async probe(s: Server, quiet = false): Promise<void> {
		if (!quiet || !this.probes[s.id]) this.probes[s.id] = "pending";
		this.probes[s.id] = await this.#backend.probe(s.gameUrl ?? s.url);
	}

	async dismiss(): Promise<void> {
		if (this.error) {
			this.error = null;
			return;
		}
		if (this.dto?.notice) {
			await this.#backend.clearNotice();
			this.dto.notice = null;
		}
	}

	newSlot(): void {
		this.editing = { id: this.#newId(), name: "", url: "", profile: null, overrides: {} };
		this.screen = "slot";
	}

	editSlot(s: Server): void {
		this.editing = snap(s);
		this.screen = "slot";
	}

	openTuning(scope: Scope): void {
		this.scope = scope;
		this.screen = "tuning";
	}

	closeScreen(): void {
		this.screen = "main";
		this.editing = null;
	}

	/** @returns i18n-ключ ошибки или null */
	async saveServer(s: Server): Promise<string | null> {
		try {
			const servers = await this.#backend.saveServer(snap(s));
			this.dto!.servers = servers;
			this.selectedId = s.id;
			this.closeScreen();
			const saved = servers.find((x) => x.id === s.id);
			if (saved) void this.probe(saved);
			return null;
		} catch (e) {
			return String(e);
		}
	}

	async deleteServer(id: string): Promise<void> {
		try {
			this.dto!.servers = await this.#backend.deleteServer(id);
			delete this.dto!.stats[id];
			if (this.selectedId === id) this.selectedId = this.dto!.servers[0]?.id ?? null;
			this.closeScreen();
		} catch (e) {
			this.error = String(e);
		}
	}

	async saveSettings(patch: Partial<Settings>): Promise<void> {
		try {
			this.dto!.settings = await this.#backend.saveSettings({ ...snap(this.dto!.settings), ...patch });
			this.applyLocale();
		} catch (e) {
			this.error = String(e);
		}
	}

	private async patchServer(id: string, patch: Partial<Server>): Promise<void> {
		const s = this.dto!.servers.find((x) => x.id === id);
		if (!s) return;
		try {
			this.dto!.servers = await this.#backend.saveServer({ ...snap(s), ...patch });
		} catch (e) {
			this.error = String(e);
		}
	}

	private stashOverrides(id: string, o: Overrides): void {
		try {
			this.#storage?.setItem(savedKey(id), JSON.stringify(o));
		} catch {
			/* нет хранилища — «РУЧ» просто откроет настройку */
		}
	}

	private takeStashedOverrides(id: string): Overrides | null {
		try {
			const raw = this.#storage?.getItem(savedKey(id));
			this.#storage?.removeItem(savedKey(id));
			return raw ? (JSON.parse(raw) as Overrides) : null;
		} catch {
			return null;
		}
	}

	/**
	 * Ручка на главном экране: профиль выбранного сервера (или глобальный, если серверов нет).
	 * Пресет — чистый профиль: правки этого уровня сбрасываются (и запоминаются).
	 * «РУЧ» — вернуть запомненные правки, а если их нет — открыть настройку.
	 */
	async setKnob(p: KnobPosition): Promise<void> {
		const s = this.selected;
		const id = s?.id ?? "global";
		if (p === "manual") {
			const saved = this.takeStashedOverrides(id);
			if (!saved || countOverrides(saved) === 0) {
				this.openTuning(s ? { kind: "server", id: s.id } : { kind: "global" });
			} else if (s) await this.patchServer(s.id, { overrides: saved });
			else await this.saveSettings({ overrides: saved });
			return;
		}
		if (!s) {
			const own = this.dto!.settings.overrides;
			if (countOverrides(own) > 0) this.stashOverrides(id, snap(own));
			await this.saveSettings({ profile: p, overrides: {} });
			return;
		}
		if (countOverrides(s.overrides) > 0) this.stashOverrides(id, snap(s.overrides));
		await this.patchServer(s.id, { profile: p, overrides: {} });
		// Общие правки ручка сервера не трогает — подсказываем, где их сбросить
		if (countOverrides(this.dto!.settings.overrides) > 0) this.error = "knob.globalOverrides";
	}

	async setScopeProfile(p: ProfileId | null): Promise<void> {
		if (this.scope.kind === "global") {
			if (p) await this.saveSettings({ profile: p });
		} else await this.patchServer(this.scope.id, { profile: p });
	}

	private async saveOverrides(next: Overrides): Promise<void> {
		if (this.scope.kind === "global") await this.saveSettings({ overrides: next });
		else await this.patchServer(this.scope.id, { overrides: next });
	}

	async setLevers(patch: Partial<Levers>): Promise<void> {
		const { base, overrides } = this.#scopeContext();
		await this.saveOverrides(withOverrides(snap(overrides), base, patch));
	}

	async setResolution(value: number): Promise<void> {
		const l = this.tuning.resolved;
		await this.setLevers(l.adaptive ? { resMin: Math.min(value, l.resMax) } : { resMin: value, resMax: value });
	}

	async resetOverrides(): Promise<void> {
		await this.saveOverrides({});
	}

	/** `manual` — нажатие кнопки: показываем ход и итог проверки. Фоновая — молча. */
	async checkUpdate(manual = false): Promise<void> {
		if (this.updateCheck === "checking") return;
		if (manual) {
			if (this.#checkReset !== undefined) this.#clearTimeout(this.#checkReset);
			this.updateCheck = "checking";
		}
		let result: UpdateCheck;
		try {
			// короткий ответ из кэша иначе мелькнул бы незаметно
			const [u] = await Promise.all([this.#backend.checkUpdate(), manual ? this.#wait(600) : null]);
			this.update = u;
			result = u ? "available" : "current";
		} catch {
			// без сети прежнее найденное обновление остаётся на экране
			result = "failed";
		}
		if (!manual) return;
		this.updateCheck = result;
		this.#checkReset = this.#setTimeout(() => (this.updateCheck = "idle"), 5000);
	}

	/** Лаунчер закрывается, обновление ставит сплэш и потом открывает лаунчер заново. */
	async restartToUpdate(): Promise<void> {
		try {
			await this.#backend.restartToUpdate();
		} catch (e) {
			this.error = String(e);
		}
	}

	async launch(safe: boolean): Promise<void> {
		const s = this.selected;
		if (!s || this.launching) return;
		this.launching = true;
		try {
			await this.#backend.launch(s.id, safe);
		} catch (e) {
			this.error = String(e);
		} finally {
			this.launching = false;
		}
	}
}

export function createAppStore(backend: LauncherBackend, env?: StoreEnv): AppStore {
	return new AppStore(backend, env);
}

export const app = createAppStore(defaultBackend);
