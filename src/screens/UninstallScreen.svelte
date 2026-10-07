<script lang="ts">
	import Button from "../components/tactile/Button.svelte";
	import Primary from "../components/tactile/Primary.svelte";
	import Shell from "../components/tactile/Shell.svelte";
	import Toggle from "../components/tactile/Toggle.svelte";
	import { setupApi, windowControls } from "../lib/api";
	import { t } from "../lib/i18n.svelte";
	import { openWindow } from "../lib/motion";

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

<div class="wrap" use:openWindow>
	<div class="card" data-part>
		<Shell pad="22px 24px 20px">
			<h1>{t("uninstall.title")}</h1>
			<p class="lead">{t("uninstall.lead")}</p>
			{#if phase === "done"}
				<p class="done" role="status">{t("uninstall.done")}</p>
				<div class="actions"><Button onclick={() => windowControls.close()}>{t("uninstall.close")}</Button></div>
			{:else}
				<div class="opt">
					<Toggle danger label={t("uninstall.wipe")} checked={wipe} disabled={phase === "running"} onchange={(v) => (wipe = v)} />
				</div>
				{#if phase === "running"}<p class="status" role="status">{t("uninstall.running")}</p>{/if}
				{#if error}<p class="err" role="alert">{error}</p>{/if}
				<div class="actions">
					<Button disabled={phase === "running"} onclick={() => windowControls.close()}>{t("uninstall.cancel")}</Button>
					<Primary tone="danger" icon="trash" busy={phase === "running"} onclick={run}>{t("uninstall.go")}</Primary>
				</div>
			{/if}
		</Shell>
	</div>
</div>

<style>
	.wrap {
		height: 100%;
		display: grid;
		place-items: center;
		padding: 2px 14px 14px;
	}
	.card {
		width: min(480px, 100%);
	}
	h1 {
		font: 700 22px/1.15 var(--tc-font-display);
		letter-spacing: -0.02em;
	}
	.lead {
		margin-top: 10px;
		color: var(--tc-muted);
	}
	.opt {
		margin-top: 18px;
	}
	.status,
	.err {
		margin-top: 10px;
		font: 500 12px/1.4 var(--tc-font-mono);
	}
	.status {
		color: var(--tc-muted);
	}
	.err {
		color: var(--tc-danger);
	}
	.done {
		margin-top: 18px;
		font-weight: 600;
		color: var(--tc-good);
	}
	.actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 22px;
	}
</style>
