<script lang="ts">
	import Knob from "../components/Knob.svelte";
	import Led from "../components/Led.svelte";
	import LaunchButton from "../components/LaunchButton.svelte";
	import Lcd from "../components/Lcd.svelte";
	import Slot from "../components/Slot.svelte";
	import { accelView } from "../lib/accel";
	import { noteLines } from "../lib/changelog";
	import { t } from "../lib/i18n.svelte";
	import { knobPosition } from "../lib/levers";
	import { app } from "../lib/store.svelte";
	import { tip } from "../lib/tooltip.svelte";

	const dto = $derived(app.dto!);
	const sel = $derived(app.selected);
	const knob = $derived(knobPosition(dto, sel?.id ?? null));
	const stats = $derived(sel ? dto.stats[sel.id] : undefined);
	const lcdValue = $derived(stats?.lastBench?.avg ?? stats?.lastSession?.avg ?? null);
	const lcdCaption = $derived(stats?.lastBench ? t("main.lastBench") : stats?.lastSession ? t("main.lastSession") : t("main.noData"));
	// «NVIDIA GeForce RTX 5070» → «RTX 5070»: марка на табличке не нужна
	const gpuName = $derived(dto.gpu?.name.replace(/^(NVIDIA|AMD|Intel\(R\))\s+(GeForce\s+|Radeon\s+(?=RX))?/i, "") ?? "");
	const accel = $derived(accelView(dto.settings, dto.gpuCheckCurrent));
	const code = (i: number) => `A${i + 1}`;
</script>

<div class="main">
	<section class="left">
		<span class="silk">{t("main.channels")}</span>
		<div class="slots" role="listbox" aria-label={t("main.slots")}>
			{#each dto.servers as s, i (s.id)}
				<Slot
					server={s}
					code={code(i)}
					selected={s.id === app.selectedId}
					probe={app.probes[s.id]}
					onselect={() => (app.selectedId = s.id)}
					onlaunch={() => {
						app.selectedId = s.id;
						void app.launch(false);
					}}
					onedit={() => app.editSlot(s)}
				/>
			{/each}
			<button class="empty" onclick={() => app.newSlot()}>
				<span class="mono">{code(dto.servers.length)}</span>
				<span>{dto.servers.length ? t("main.emptySlot") : t("main.noServer")}</span>
				<span class="plus" aria-hidden="true"></span>
			</button>
		</div>
		{#if app.message}
			<div class="notice mono" role="status">
				<span>{app.message}</span>
				<button aria-label={t("window.close")} onclick={() => app.dismiss()}>✕</button>
			</div>
		{/if}
		<LaunchButton busy={app.launching} onlaunch={(safe) => (sel ? app.launch(safe) : app.newSlot())} />
	</section>

	<aside class="right">
		{#each ["tl", "tr", "bl", "br"] as c (c)}<span class="screw {c}" aria-hidden="true"></span>{/each}
		{#if app.updating !== null}
			<Lcd value={app.updating} unit="%" caption={t("update.downloading")} />
		{:else}
			<Lcd value={lcdValue} unit="FPS" caption={lcdCaption} history={stats?.history ?? []} />
		{/if}
		{#if app.update && app.updating === null}
			<button
				class="update"
				{@attach tip(() =>
					app.update ? { title: t("update.notesTitle", { version: app.update.version }), body: t("update.noNotes"), lines: noteLines(app.update.notes) } : undefined
				)}
				onclick={() => app.applyUpdate()}
			>
				<span class="mono">{t("update.available", { version: app.update.version })}</span><span class="lamp" aria-hidden="true"></span>
			</button>
		{/if}
		<Knob value={knob} onchange={(p) => app.setKnob(p)} />
		{#if dto.gpu}
			<dl class="passport mono">
				<dt>GPU</dt>
				<dd title={dto.gpu.name}>{gpuName}</dd>
				<dt>VRAM</dt>
				<dd>{Math.round(dto.gpu.vramMb / 1024)} {t("unit.gb")}</dd>
				<dt>API</dt>
				<dd>{accel.api.toUpperCase()}</dd>
				<dt>{t("accel.label")}</dt>
				<dd class="accel" {@attach tip(() => ({ title: t(`accel.${accel.state}`), body: t(`accel.tip.${accel.state}`) }))}>
					<Led state={accel.led} label="" />{t(`accel.${accel.state}`)}
				</dd>
			</dl>
		{:else}
			<div class="passport mono">{t("gpu.unknown")}</div>
		{/if}
		<button class="tune silk" onclick={() => app.openTuning(sel ? { kind: "server", id: sel.id } : { kind: "global" })}>
			{t("main.tune")}
		</button>
	</aside>
</div>

<style>
	.main {
		display: grid;
		grid-template-columns: minmax(0, 1fr) 236px;
		gap: 2px;
		height: 100%;
		min-height: 0;
	}
	.left,
	.right {
		background: var(--grain), var(--face);
		box-shadow: var(--bevel);
		padding: 16px 18px 18px;
		min-height: 0;
	}
	.left {
		display: grid;
		grid-template-rows: auto minmax(0, 1fr) auto auto;
		gap: 10px;
	}
	.slots {
		display: grid;
		align-content: start;
		gap: 6px;
		overflow-y: auto;
		min-height: 0;
	}
	.empty {
		display: grid;
		grid-template-columns: 38px 1fr auto;
		gap: 12px;
		align-items: center;
		padding: 11px 14px 11px 12px;
		border: 1px dashed var(--ink-3);
		color: var(--ink-2);
		text-align: left;
	}
	.empty .mono {
		text-align: center;
	}
	.empty:hover {
		border-color: var(--signal);
		color: var(--ink);
	}
	.plus {
		position: relative;
		width: 11px;
		height: 11px;
	}
	.plus::before,
	.plus::after {
		content: "";
		position: absolute;
		left: 0;
		top: 5px;
		width: 11px;
		height: 1.5px;
		background: currentColor;
	}
	.plus::after {
		rotate: 90deg;
	}
	.notice {
		display: flex;
		justify-content: space-between;
		gap: 12px;
		padding: 8px 12px;
		color: var(--signal-hi);
		background: var(--lcd-bg);
		box-shadow: var(--recess);
	}
	.right {
		position: relative;
		display: grid;
		align-content: start;
		justify-items: center;
		gap: 14px;
	}
	.right > :global(.lcd),
	.passport,
	.tune,
	.update {
		justify-self: stretch;
	}
	.update {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-top: -6px;
		padding: 9px 12px;
		color: var(--signal-ink);
		background: linear-gradient(var(--signal-hi), var(--signal));
		box-shadow:
			inset 0 1px 0 rgb(255 255 255 / 0.3),
			0 3px 0 var(--signal-deep);
		transition: translate 0.07s;
	}
	.update:active {
		translate: 0 3px;
		box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.3);
	}
	.update .lamp {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: var(--signal-ink);
		animation: pulse 1.6s ease-in-out infinite;
	}
	@keyframes pulse {
		50% {
			opacity: 0.25;
		}
	}
	/* винты приборной панели: шлиц повёрнут у каждого по-своему */
	.screw {
		position: absolute;
		width: 9px;
		height: 9px;
		border-radius: 50%;
		background: radial-gradient(circle at 35% 35%, var(--knurl-a), var(--knurl-b));
		box-shadow: var(--recess);
	}
	.screw::after {
		content: "";
		position: absolute;
		left: 1px;
		right: 1px;
		top: 4px;
		height: 1px;
		background: rgb(0 0 0 / 0.55);
	}
	.tl {
		left: 6px;
		top: 6px;
		rotate: 35deg;
	}
	.tr {
		right: 6px;
		top: 6px;
		rotate: -20deg;
	}
	.bl {
		left: 6px;
		bottom: 6px;
		rotate: 80deg;
	}
	.br {
		right: 6px;
		bottom: 6px;
		rotate: 10deg;
	}
	/* табличка с данными железа, как шильдик на корпусе */
	.passport {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 3px 10px;
		margin: 0;
		padding: 8px 10px;
		color: var(--ink-3);
		background: var(--well);
		box-shadow: var(--recess);
	}
	.passport dd {
		margin: 0;
		text-align: right;
		color: var(--ink);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	/* ореол лампочки шире строки — не обрезаем; статусы короткие, многоточие не нужно */
	.passport .accel {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 8px;
		overflow: visible;
	}
	.tune {
		padding: 10px;
		background: var(--face-2);
		box-shadow: var(--lift);
		transition: color 0.12s;
	}
	.tune:hover {
		color: var(--ink);
	}
	.tune:active {
		box-shadow: var(--recess);
	}
</style>
