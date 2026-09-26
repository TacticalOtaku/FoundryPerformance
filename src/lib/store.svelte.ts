import { api, updateApi, windowControls } from "./api";
import { setLocale, t } from "./i18n.svelte";
import { baseFor, overridesFor, type Scope, withOverrides } from "./levers";
import { probeUrl } from "./status";
import type { Levers, Overrides, ProbeResult, ProfileId, Server, Settings, StateDto, UpdateInfo } from "./types";

export type Screen = "main" | "tuning" | "slot";

const snap = <T>(v: T): T => $state.snapshot(v) as T;
const systemLocale = (): "ru" | "en" => (navigator.language.toLowerCase().startsWith("ru") ? "ru" : "en");

class AppStore {
	dto = $state<StateDto | null>(null);
	selectedId = $state<string | null>(null);
	screen = $state<Screen>("main");
	editing = $state<Server | null>(null);
	scope = $state<Scope>({ kind: "global" });
	probes = $state<Record<string, ProbeResult | "pending">>({});
	error = $state<string | null>(null);
	launching = $state(false);
	/** Доступное обновление (null — нет или проверка не удалась). */
	update = $state<UpdateInfo | null>(null);
	/** Проценты загрузки обновления; null — не качаем. */
	updating = $state<number | null>(null);

	get selected(): Server | null {
		return this.dto?.servers.find((s) => s.id === this.selectedId) ?? null;
	}

	get message(): string | null {
		if (this.error) return t(this.error);
		const n = this.dto?.notice;
		return n ? t(n.key, n.params) : null;
	}

	async load(): Promise<void> {
		const dto = await api.getState();
		this.dto = dto;
		this.applyLocale();
		this.selectedId = dto.servers[0]?.id ?? null;
		for (const s of dto.servers) void this.probe(s);
		void updateApi.check().then((u) => (this.update = u)).catch(() => {});
		// Пока лаунчер открыт, кружки сами обновляются — видно, когда ГМ запустил сервер
		setInterval(() => {
			for (const s of this.dto?.servers ?? []) void this.probe(s, true);
		}, 60_000);
	}

	private applyLocale(): void {
		const pref = this.dto?.settings.locale ?? "auto";
		setLocale(pref === "auto" ? systemLocale() : pref);
	}

	/** `quiet` — фоновая перепроверка без мигания «проверка…». */
	async probe(s: Server, quiet = false): Promise<void> {
		if (!quiet || !this.probes[s.id]) this.probes[s.id] = "pending";
		this.probes[s.id] = await api.probe(probeUrl(s));
	}

	async dismiss(): Promise<void> {
		if (this.error) {
			this.error = null;
			return;
		}
		if (this.dto?.notice) {
			await api.clearNotice();
			this.dto.notice = null;
		}
	}

	newSlot(): void {
		this.editing = { id: crypto.randomUUID(), name: "", url: "", profile: null, overrides: {} };
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
			const servers = await api.saveServer(snap(s));
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
			this.dto!.servers = await api.deleteServer(id);
			delete this.dto!.stats[id];
			if (this.selectedId === id) this.selectedId = this.dto!.servers[0]?.id ?? null;
			this.closeScreen();
		} catch (e) {
			this.error = String(e);
		}
	}

	async saveSettings(patch: Partial<Settings>): Promise<void> {
		try {
			this.dto!.settings = await api.saveSettings({ ...snap(this.dto!.settings), ...patch });
			this.applyLocale();
		} catch (e) {
			this.error = String(e);
		}
	}

	private async patchServer(id: string, patch: Partial<Server>): Promise<void> {
		const s = this.dto!.servers.find((x) => x.id === id);
		if (!s) return;
		try {
			this.dto!.servers = await api.saveServer({ ...snap(s), ...patch });
		} catch (e) {
			this.error = String(e);
		}
	}

	/** Ручка на главном экране: профиль выбранного сервера (или глобальный, если серверов нет). */
	async setKnob(p: ProfileId): Promise<void> {
		const s = this.selected;
		if (s) await this.patchServer(s.id, { profile: p });
		else await this.saveSettings({ profile: p });
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
		const dto = this.dto!;
		await this.saveOverrides(withOverrides(snap(overridesFor(dto, this.scope)), baseFor(dto, this.scope), patch));
	}

	async resetOverrides(): Promise<void> {
		await this.saveOverrides({});
	}

	async applyUpdate(): Promise<void> {
		if (this.updating !== null) return;
		this.updating = 0;
		this.error = null;
		const off = await updateApi.onProgress((pct) => (this.updating = pct));
		try {
			// при успехе Rust запускает новую версию и закрывает эту
			await updateApi.apply();
		} catch (e) {
			this.error = String(e);
			this.updating = null;
		} finally {
			off();
		}
	}

	async launch(safe: boolean): Promise<void> {
		const s = this.selected;
		if (!s || this.launching) return;
		this.launching = true;
		try {
			await api.launch(s.id, safe);
			await windowControls.destroy();
		} catch (e) {
			this.error = String(e);
		} finally {
			this.launching = false;
		}
	}
}

export const app = new AppStore();
