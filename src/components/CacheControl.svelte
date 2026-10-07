<script lang="ts">
	import { onMount } from "svelte";
	import { api } from "../lib/api";
	import { formatBytes } from "../lib/bytes";
	import { locale, t } from "../lib/i18n.svelte";
	import { app } from "../lib/store.svelte";
	import { tip, tipFor } from "../lib/tooltip.svelte";
	import Band from "./tactile/Band.svelte";
	import Button from "./tactile/Button.svelte";
	import Label from "./tactile/Label.svelte";

	/** idle → armed (ждём второго нажатия) → busy → итог на клавише, затем снова idle. */
	type Phase = { kind: "idle" } | { kind: "armed" } | { kind: "busy" } | { kind: "done"; text: string; warn: boolean };

	const ARM_MS = 4000;
	const RESULT_MS = 5000;

	let bytes = $state<number | null>(null);
	let phase = $state<Phase>({ kind: "idle" });
	let timer: ReturnType<typeof setTimeout> | undefined;

	const limit = $derived((app.dto?.settings.engine.diskCacheMb ?? 2048) * 1024 * 1024);
	const fmt = (n: number) => formatBytes(n, { mb: t("unit.mb"), gb: t("unit.gb") }, locale());

	onMount(() => {
		void api.cacheSize().then((n) => (bytes = n)).catch(() => {});
		return () => clearTimeout(timer);
	});

	function later(ms: number) {
		clearTimeout(timer);
		timer = setTimeout(() => (phase = { kind: "idle" }), ms);
	}

	async function onPress() {
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
	<div class="head">
		<Label>{t("cache.title")}</Label>
		<span class="used">{bytes === null ? "…" : `${fmt(bytes)} / ${fmt(limit)}`}</span>
	</div>
	<Band value={bytes === null ? 0 : bytes / limit} label={t("cache.title")} />
	<div>
		<Button
			danger
			armed={phase.kind === "armed" || (phase.kind === "done" && phase.warn)}
			aria-live="polite"
			disabled={phase.kind === "busy"}
			onclick={onPress}>{label}</Button
		>
	</div>
</div>

<style>
	.cache {
		display: grid;
		gap: 8px;
	}
	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.used {
		font: 600 12.5px/1 var(--tc-font-mono);
	}
</style>
