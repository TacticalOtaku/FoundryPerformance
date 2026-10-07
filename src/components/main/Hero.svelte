<script lang="ts">
	import { t } from "../../lib/i18n.svelte";
	import { hostOf, intentReason, lastRun, STATUS_DETAIL, STATUS_TONE } from "../../lib/intent";
	import { type KnobPosition, knobPosition } from "../../lib/levers";
	import { rise } from "../../lib/motion";
	import { PROFILE_POSITIONS, profileTipKey } from "../../lib/profile-tip";
	import { slotStatus } from "../../lib/status";
	import { app } from "../../lib/store.svelte";
	import Icon from "../tactile/Icon.svelte";
	import Label from "../tactile/Label.svelte";
	import Pill from "../tactile/Pill.svelte";
	import Primary from "../tactile/Primary.svelte";
	import Segments from "../tactile/Segments.svelte";
	import Shell from "../tactile/Shell.svelte";

	const dto = $derived(app.dto!);
	const sel = $derived(app.selected);
	const knob = $derived(knobPosition(dto, sel?.id ?? null));
	const probe = $derived(sel ? app.probes[sel.id] : undefined);
	const status = $derived(sel ? slotStatus(sel, probe) : null);
	const run = $derived(sel ? lastRun(dto.stats[sel.id]) : null);
	const reason = $derived(intentReason(sel?.id ?? null, dto.settings.lastServer));
	const world = $derived(probe && probe !== "pending" ? (probe.world ?? "") : "");
	const statusTip = $derived(status ? { title: t(`status.${status}`), body: t(STATUS_DETAIL[status], { world }) } : undefined);
	const profiles = $derived(
		PROFILE_POSITIONS.map((p) => ({ value: p, label: t(`profile.${p}.long`), short: t(`profile.${p}`), arrowSkip: p === "manual" }))
	);

	// Shift при нажатии — безопасный запуск; подпись кнопки меняется, пока Shift зажат
	let shift = $state(false);
	const track = (e: KeyboardEvent) => (shift = e.shiftKey);
	const goLabel = $derived(app.launching ? t("main.launching") : shift ? t("main.launchSafe") : t("main.launch"));
</script>

<svelte:window onkeydown={track} onkeyup={track} onblur={() => (shift = false)} />

<Shell fill pad="22px 24px 20px">
	{#if sel}
		<div class="top">
			<Label>{reason === "last" ? t("main.continue") : t("main.selected")}</Label>
			{#if status}<Pill tone={STATUS_TONE[status]} tip={statusTip}>{t(`status.${status}`)}</Pill>{/if}
		</div>
		<div class="grow"></div>
		<div class="intent" use:rise={sel.id}>
			<h1 class="name">{sel.name}</h1>
			<p class="meta">
				<span>{hostOf(sel.url)}</span>
				{#if run}
					<span>{t(run.kind === "bench" ? "main.lastBench" : "main.lastSession")} <b>{run.fps}</b> FPS</span>
				{:else}
					<span>{t("main.noData")}</span>
				{/if}
			</p>
			{#if reason}<span class="why"><Icon name="clock" size={14} />{t(`main.reason.${reason}`)}</span>{/if}
		</div>
		<div class="grow"></div>
		<Segments label={t("main.profile")} options={profiles} value={knob} onchange={(p: KnobPosition) => app.setKnob(p)} />
		<p class="ptip">{t(profileTipKey(knob))}</p>
		<Primary size="lg" busy={app.launching} onclick={(e: MouseEvent) => app.launch(e.shiftKey)}>{goLabel}</Primary>
		<p class="hint">{t("main.safeHint")}</p>
	{:else}
		<div class="grow"></div>
		<h1 class="name empty">{t("main.noServer")}</h1>
		<div class="grow"></div>
		<Primary size="lg" icon="plus" onclick={() => app.newSlot()}>{t("main.addServer")}</Primary>
	{/if}
</Shell>

<style>
	.top {
		display: flex;
		align-items: center;
		gap: 8px;
		min-height: 18px;
	}
	.grow {
		flex: 1;
		min-height: 12px;
	}
	.intent {
		display: grid;
		justify-items: start;
	}
	.name {
		display: -webkit-box;
		-webkit-box-orient: vertical;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		overflow: hidden;
		overflow-wrap: anywhere;
		font: 700 50px/1.02 var(--tc-font-display);
		letter-spacing: -0.035em;
		text-wrap: balance;
	}
	.name.empty {
		font-size: 34px;
		color: var(--tc-muted);
	}
	.meta {
		display: flex;
		flex-wrap: wrap;
		gap: 4px 14px;
		margin-top: 14px;
		font: 500 12.5px/1.4 var(--tc-font-mono);
		color: var(--tc-muted);
	}
	.meta b {
		color: var(--tc-ink);
		font-weight: 600;
	}
	.why {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		margin-top: 16px;
		padding: 6px 12px 6px 9px;
		border-radius: var(--tc-r-ctl);
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
		font-size: 12.5px;
		color: var(--tc-muted);
	}
	.why :global(.ic) {
		color: var(--tc-accent-text);
	}
	.ptip {
		min-height: 35px;
		margin: 8px 2px 16px;
		font-size: 12.5px;
		line-height: 1.4;
		color: var(--tc-muted);
	}
	.hint {
		margin-top: 9px;
		text-align: center;
		font: 500 10.5px/1.3 var(--tc-font-mono);
		letter-spacing: 0.06em;
		color: var(--tc-muted);
	}
</style>
