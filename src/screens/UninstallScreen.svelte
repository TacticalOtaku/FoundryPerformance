<script lang="ts">
	import Toggle from "../components/Toggle.svelte";
	import { setupApi, windowControls } from "../lib/api";
	import { t } from "../lib/i18n.svelte";

	let wipe = $state(false);
	let phase = $state<"idle" | "running" | "done" | "error">("idle");
	let error = $state<string | null>(null);

	async function run() {
		if (phase === "running") return;
		phase = "running";
		error = null;
		try {
			await setupApi.uninstall(wipe);
			phase = "done";
		} catch (e) {
			phase = "error";
			error = t(String(e));
		}
	}
</script>

<section class="uninstall">
	<h1>{t("uninstall.title")}</h1>
	<p class="lead">{t("uninstall.lead")}</p>

	{#if phase === "done"}
		<p class="done" role="status">{t("uninstall.done")}</p>
	{:else}
		<div class="toggle">
			<Toggle label={t("uninstall.wipe")} checked={wipe} onchange={(v) => (wipe = v)} />
		</div>
		{#if error}<p class="err mono" role="alert">{error}</p>{/if}
		<div class="actions">
			<button class="danger" aria-busy={phase === "running"} onclick={run}>
				{phase === "running" ? t("uninstall.running") : t("uninstall.go")}
			</button>
			<button class="secondary mono" onclick={() => windowControls.close()}>{t("uninstall.cancel")}</button>
		</div>
	{/if}
</section>

<style>
	.uninstall {
		height: 100%;
		background: var(--face);
		padding: 28px 24px;
		display: grid;
		align-content: start;
		gap: 18px;
	}
	h1 {
		margin: 0;
		font-weight: 800;
		font-size: 30px;
		line-height: 1.05;
	}
	.lead {
		margin: 0;
		color: var(--ink-2);
		font-size: 14px;
		max-width: 520px;
	}
	.toggle {
		max-width: 460px;
	}
	.actions {
		display: flex;
		gap: 8px;
	}
	.danger {
		padding: 13px 22px;
		background: var(--led-err);
		color: var(--face);
		font-weight: 800;
		letter-spacing: 0.06em;
	}
	.danger[aria-busy="true"] {
		cursor: progress;
	}
	.secondary {
		padding: 0 16px;
		border: 1px solid var(--ink-3);
	}
	.err {
		margin: 0;
		color: var(--led-err);
	}
	.done {
		margin: 0;
		font-size: 16px;
	}
</style>
