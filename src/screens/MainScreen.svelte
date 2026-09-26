<script lang="ts">
	import Knob from "../components/Knob.svelte";
	import LaunchButton from "../components/LaunchButton.svelte";
	import Lcd from "../components/Lcd.svelte";
	import Slot from "../components/Slot.svelte";
	import { t } from "../lib/i18n.svelte";
	import { app } from "../lib/store.svelte";

	const dto = $derived(app.dto!);
	const sel = $derived(app.selected);
	const profile = $derived(sel?.profile ?? dto.settings.profile);
	const stats = $derived(sel ? dto.stats[sel.id] : undefined);
	const lcdValue = $derived(stats?.lastBench?.avg ?? stats?.lastSession?.avg ?? null);
	const lcdCaption = $derived(stats?.lastBench ? t("main.lastBench") : stats?.lastSession ? t("main.lastSession") : t("main.noData"));
	const gpuLine = $derived(
		dto.gpu
			? `${dto.gpu.name} · ${Math.round(dto.gpu.vramMb / 1024)} ${t("unit.gb")} · ${dto.settings.engine.angle.toUpperCase()}`
			: t("gpu.unknown")
	);
	const code = (i: number) => `A${i + 1}`;
</script>

<div class="main">
	<section class="left">
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
				<span class="mono">{t("main.addSlot")}</span>
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
		<Lcd value={lcdValue} unit="FPS" caption={lcdCaption} />
		<Knob value={profile} onchange={(p) => app.setKnob(p)} />
		<div class="gpu mono">{gpuLine}</div>
		<button class="tune mono" onclick={() => app.openTuning(sel ? { kind: "server", id: sel.id } : { kind: "global" })}>
			{t("main.tune")} →
		</button>
	</aside>
</div>

<style>
	.main {
		display: grid;
		grid-template-columns: minmax(0, 1fr) 232px;
		gap: 2px;
		height: 100%;
		min-height: 0;
	}
	.left,
	.right {
		background: var(--face);
		padding: 18px;
		min-height: 0;
	}
	.left {
		display: grid;
		grid-template-rows: minmax(0, 1fr) auto auto;
		gap: 12px;
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
		grid-template-columns: 34px 1fr auto;
		gap: 12px;
		align-items: center;
		padding: 12px 14px;
		border: 1px dashed var(--ink-3);
		color: var(--ink-2);
		text-align: left;
	}
	.empty:hover {
		border-color: var(--signal);
		color: var(--ink);
	}
	.notice {
		display: flex;
		justify-content: space-between;
		gap: 12px;
		padding: 8px 12px;
		background: var(--lcd-bg);
		color: var(--signal);
	}
	.right {
		display: grid;
		align-content: start;
		justify-items: stretch;
		gap: 18px;
	}
	.gpu {
		color: var(--ink-2);
		text-align: center;
	}
	.tune {
		padding: 10px;
		border: 1px solid var(--ink-3);
	}
	.tune:hover {
		border-color: var(--signal);
	}
</style>
