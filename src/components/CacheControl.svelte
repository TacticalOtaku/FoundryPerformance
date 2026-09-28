<script lang="ts">
	import { onMount } from "svelte";
	import { api } from "../lib/api";
	import { formatBytes } from "../lib/bytes";
	import { locale, t } from "../lib/i18n.svelte";
	import { tip, tipFor } from "../lib/tooltip.svelte";

	/** idle → armed (ждём второго нажатия) → busy → итог на клавише, затем снова idle. */
	type Phase = { kind: "idle" } | { kind: "armed" } | { kind: "busy" } | { kind: "done"; text: string; warn: boolean };

	const ARM_MS = 4000;
	const RESULT_MS = 5000;

	let bytes = $state<number | null>(null);
	let phase = $state<Phase>({ kind: "idle" });
	let timer: ReturnType<typeof setTimeout> | undefined;

	const fmt = (n: number) => formatBytes(n, { mb: t("unit.mb"), gb: t("unit.gb") }, locale());

	onMount(() => {
		void api.cacheSize().then((n) => (bytes = n)).catch(() => {});
		return () => clearTimeout(timer);
	});

	function later(ms: number) {
		clearTimeout(timer);
		timer = setTimeout(() => (phase = { kind: "idle" }), ms);
	}

	async function press() {
		if (phase.kind === "busy") return;
		if (phase.kind !== "armed") {
			phase = { kind: "armed" };
			later(ARM_MS);
			return;
		}
		clearTimeout(timer);
		phase = { kind: "busy" };
		try {
			const r = await api.clearCache();
			bytes = await api.cacheSize().catch(() => 0);
			phase = r.pending
				? { kind: "done", text: t("cache.pending"), warn: true }
				: { kind: "done", text: t("cache.freed", { size: fmt(r.freed) }), warn: false };
		} catch (e) {
			phase = { kind: "done", text: t(String(e)), warn: true };
		}
		later(RESULT_MS);
	}

	const label = $derived(
		phase.kind === "armed" ? t("cache.confirm") : phase.kind === "busy" ? t("cache.clearing") : phase.kind === "done" ? phase.text : t("cache.clear")
	);
</script>

<div class="cache" {@attach tip(() => tipFor("cacheClear", t("cache.title")))}>
	<span class="lbl silk">{t("cache.title")}</span>
	<div class="row">
		<span class="used mono">{bytes === null ? "…" : fmt(bytes)}</span>
		<button
			class="key mono"
			class:armed={phase.kind === "armed"}
			class:warn={phase.kind === "done" && phase.warn}
			aria-live="polite"
			disabled={phase.kind === "busy"}
			onclick={press}>{label}</button
		>
	</div>
</div>

<style>
	.cache {
		display: grid;
		gap: 7px;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	/* занятое место — на мини-индикаторе, как число у фейдера FPS */
	.used {
		min-width: 64px;
		padding: 4px 8px;
		text-align: right;
		color: var(--lcd-fg);
		background: var(--lcd-bg);
		border: 1px solid rgb(0 0 0 / 0.6);
		box-shadow: var(--recess);
		text-shadow: var(--glow-lcd);
	}
	.key {
		padding: 5px 10px;
		color: var(--ink-2);
		background: var(--face-2);
		box-shadow: var(--lift);
		transition:
			color 0.12s,
			background 0.12s;
	}
	.key:hover {
		color: var(--ink);
	}
	.key:active {
		box-shadow: var(--recess);
	}
	/* второе нажатие удалит файлы — клавиша загорается сигнальным */
	.key.armed {
		color: var(--signal-ink);
		background: linear-gradient(var(--signal-hi), var(--signal));
	}
	.key.warn {
		color: var(--signal-text);
	}
</style>
