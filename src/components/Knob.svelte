<script lang="ts">
	import { t } from "../lib/i18n.svelte";
	import { tip, tipFor } from "../lib/tooltip.svelte";
	import type { ProfileId } from "../lib/types";

	let { value, onchange }: { value: ProfileId; onchange: (p: ProfileId) => void } = $props();

	const ORDER: ProfileId[] = ["quality", "balance", "potato"];
	const ANGLE: Record<ProfileId, number> = { quality: -60, balance: 0, potato: 60 };

	function step(delta: number) {
		const i = ORDER.indexOf(value) + delta;
		if (i >= 0 && i < ORDER.length) onchange(ORDER[i]);
	}

	function onkeydown(e: KeyboardEvent) {
		if (e.key === "ArrowRight" || e.key === "ArrowUp") step(1);
		else if (e.key === "ArrowLeft" || e.key === "ArrowDown") step(-1);
		else return;
		e.preventDefault();
	}
</script>

<div class="wrap">
	<div
		class="knob"
		role="slider"
		tabindex="0"
		aria-label={t("knob.label")}
		aria-valuemin={0}
		aria-valuemax={2}
		aria-valuenow={ORDER.indexOf(value)}
		aria-valuetext={t(`profile.${value}.long`)}
		{onkeydown}
		onwheel={(e) => step(e.deltaY > 0 ? 1 : -1)}
		onclick={() => step(value === "potato" ? -2 : 1)}
		use:tip={tipFor("profile", t("knob.label"))}
	>
		<div class="cap" style:transform={`rotate(${ANGLE[value]}deg)`}><i></i></div>
		{#each ORDER as p (p)}<span class="tick" style:transform={`rotate(${ANGLE[p]}deg)`}></span>{/each}
	</div>
	<div class="scale mono">
		{#each ORDER as p (p)}
			<button class:on={p === value} onclick={() => onchange(p)} use:tip={tipFor(p, t(`profile.${p}.long`))}>{t(`profile.${p}`)}</button>
		{/each}
	</div>
</div>

<style>
	.wrap {
		display: grid;
		gap: 10px;
		justify-items: center;
	}
	.knob {
		position: relative;
		width: 116px;
		height: 116px;
		border-radius: 50%;
		background: var(--line);
		display: grid;
		place-items: center;
		cursor: pointer;
	}
	.cap {
		width: 88px;
		height: 88px;
		border-radius: 50%;
		background: var(--inverse-bg);
		position: relative;
		transition: transform 0.32s var(--ease-detent);
	}
	.cap i {
		position: absolute;
		left: 50%;
		top: 8px;
		width: 5px;
		height: 28px;
		margin-left: -2.5px;
		background: var(--signal);
	}
	.tick {
		position: absolute;
		inset: 0;
		pointer-events: none;
	}
	.tick::before {
		content: "";
		position: absolute;
		left: 50%;
		top: -9px;
		width: 2px;
		height: 6px;
		margin-left: -1px;
		background: var(--ink-3);
	}
	.scale {
		display: flex;
		justify-content: space-between;
		width: 100%;
	}
	.scale button {
		padding: 3px 6px;
		color: var(--ink-2);
	}
	.scale button.on {
		color: var(--signal-text);
	}
</style>
