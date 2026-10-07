<script lang="ts">
	import { t } from "../../lib/i18n.svelte";
	import { lastRun } from "../../lib/intent";
	import { countTo } from "../../lib/motion";
	import { app } from "../../lib/store.svelte";
	import Core from "../tactile/Core.svelte";
	import Label from "../tactile/Label.svelte";

	const run = $derived(app.selected ? lastRun(app.dto!.stats[app.selected.id]) : null);
</script>

<Core pad="14px 16px 15px">
	<Label>{t(run?.kind === "bench" ? "main.lastBench" : "main.lastSession")}</Label>
	<div class="sr">
		{#if run}
			<span><span class="num" use:countTo={run.fps}></span><span class="unit">FPS</span></span>
			<Label>{t(`profile.${run.profile}.long`)}</Label>
		{:else}
			<span class="nodata">{t("main.noData")}</span>
		{/if}
	</div>
</Core>

<style>
	.sr {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		min-height: 30px;
		margin-top: 8px;
	}
	.num {
		font: 700 30px/1 var(--tc-font-display);
		letter-spacing: -0.04em;
	}
	.unit {
		margin-left: 5px;
		font: 600 11px/1 var(--tc-font-mono);
		color: var(--tc-muted);
	}
	.nodata {
		align-self: center;
		font: 500 12.5px/1.3 var(--tc-font-mono);
		color: var(--tc-muted);
	}
</style>
