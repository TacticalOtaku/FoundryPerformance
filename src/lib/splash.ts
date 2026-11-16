import type { LauncherBackend } from "./api";
import type { UpdateInfo } from "./types";

/** Сплэш ждёт ленту меньше фоновой проверки — игрок не должен сидеть перед кольцом. */
export const CHECK_TIMEOUT_S = 4;
export const OFFLINE_MS = 1500;
export const UPDATED_MS = 600;
/** Ответ Rust на «Пропустить»; на экран не выводится. */
export const CANCELLED = "update.err.cancelled";

export type Phase =
	| { kind: "checking" }
	/** `pct: null` — процентов ещё нет (или сервер не прислал размер файла). */
	| { kind: "downloading"; version: string; pct: number | null }
	| { kind: "verifying"; version: string }
	| { kind: "restarting"; version: string }
	| { kind: "offline" }
	| { kind: "error"; key: string }
	| { kind: "updated"; version: string };

export type RingMode = "spin" | "progress" | "breathe" | "muted" | "error" | "full";

export type SplashBackend = Pick<
	LauncherBackend,
	"splashFlags" | "checkUpdate" | "applyUpdate" | "onUpdateProgress" | "cancelUpdate" | "splashDone"
>;

export function ringMode(p: Phase): RingMode {
	switch (p.kind) {
		case "checking":
			return "spin";
		case "downloading":
			return p.pct === null ? "spin" : "progress";
		case "verifying":
		case "restarting":
			return "breathe";
		case "offline":
			return "muted";
		case "error":
			return "error";
		case "updated":
			return "full";
	}
}

/** После 100 % идут проверка и замена exe — их не прерываем. */
export const canSkip = (p: Phase): boolean => p.kind === "checking" || p.kind === "downloading";

/** Поток сплэша: проверка → загрузка → проверка подписи → перезапуск; обновление ставится без вопроса. */
export class SplashFlow {
	#backend: SplashBackend;
	#show: (p: Phase) => void;
	#wait: (ms: number) => Promise<void>;
	/** Пропустили или закрыли — поздние ответы Rust больше ничего не показывают и не ставят. */
	#stopped = false;
	#finished = false;

	constructor(
		backend: SplashBackend,
		show: (p: Phase) => void,
		wait: (ms: number) => Promise<void> = (ms) => new Promise((r) => setTimeout(r, ms))
	) {
		this.#backend = backend;
		this.#show = show;
		this.#wait = wait;
	}

	async run(): Promise<void> {
		const { updated } = await this.#backend.splashFlags();
		if (updated) {
			this.#show({ kind: "updated", version: updated });
			await this.#wait(UPDATED_MS);
			return this.close();
		}
		this.#show({ kind: "checking" });
		let info: UpdateInfo | null;
		try {
			info = await this.#backend.checkUpdate(CHECK_TIMEOUT_S);
		} catch {
			if (this.#stopped) return;
			this.#show({ kind: "offline" });
			await this.#wait(OFFLINE_MS);
			return this.close();
		}
		if (this.#stopped) return;
		if (!info) return this.close();
		await this.#install(info.version);
	}

	async #install(version: string): Promise<void> {
		this.#show({ kind: "downloading", version, pct: null });
		const off = await this.#backend.onUpdateProgress((pct) => {
			if (this.#stopped) return;
			this.#show(pct >= 100 ? { kind: "verifying", version } : { kind: "downloading", version, pct });
		});
		try {
			// при успехе Rust запускает новую версию и закрывает эту
			await this.#backend.applyUpdate();
			if (!this.#stopped) this.#show({ kind: "restarting", version });
		} catch (e) {
			const key = String(e);
			if (!this.#stopped && key !== CANCELLED) this.#show({ kind: "error", key });
		} finally {
			off();
		}
	}

	async skip(): Promise<void> {
		if (this.#stopped) return;
		this.#stopped = true;
		await this.#backend.cancelUpdate();
		await this.close();
	}

	/** Открыть лаунчер (или игру по адресу) и закрыть сплэш; повторный вызов ничего не делает. */
	async close(): Promise<void> {
		this.#stopped = true;
		if (this.#finished) return;
		this.#finished = true;
		await this.#backend.splashDone();
	}
}
