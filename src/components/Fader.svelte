<script lang="ts">
	import type { TipData } from "../lib/tips";
	import { tip as tooltip } from "../lib/tooltip.svelte";

	let {
		label,
		value,
		min,
		max,
		step,
		format,
		modified = false,
		tip,
		marks = [],
		presets = [],
		editable = false,
		inputLabel = "",
		onchange
	}: {
		label: string;
		value: number;
		min: number;
		max: number;
		step: number;
		format: (v: number) => string;
		modified?: boolean;
		tip?: TipData;
		/** Подписи шкалы слева от прорези. */
		marks?: number[];
		/** Быстрые значения под фейдером. */
		presets?: number[];
		/** Точный ввод числа на мини-индикаторе. */
		editable?: boolean;
		inputLabel?: string;
		onchange: (v: number) => void;
	} = $props();

	const TRACK = 108;
	const THUMB = 14;

	// Пока бегунок тянут, значение только показывается; сохраняется на `change`
	let live = $state(0);
	$effect(() => {
		live = value;
	});

	const clamp = (v: number) => Math.min(max, Math.max(min, Math.round(v / step) * step));
	const markY = (m: number) => (TRACK - THUMB) * (1 - (m - min) / (max - min)) + THUMB / 2;

	function commitTyped(e: Event) {
		const input = e.currentTarget as HTMLInputElement;
		const n = Number(input.value);
		if (!Number.isFinite(n) || input.value.trim() === "") {
			input.value = String(live);
			return;
		}
		live = Math.min(max, Math.max(min, Math.round(n)));
		input.value = String(live);
		onchange(live);
	}
</script>

<div class="fader" use:tooltip={tip}>
	<span class="lbl silk">{label}{#if modified}<i class="dot" aria-hidden="true"></i>{/if}</span>
	<div class="row" style:height={`${TRACK}px`}>
		{#if marks.length}
			<div class="scale" aria-hidden="true">
				{#each marks as m (m)}
					<span style:top={`${markY(m)}px`}>{m}</span>
				{/each}
			</div>
		{/if}
		<input
			type="range"
			{min}
			{max}
			{step}
			value={live}
			aria-label={label}
			aria-valuetext={format(live)}
			style:height={`${TRACK}px`}
			oninput={(e) => (live = Number(e.currentTarget.value))}
			onchange={(e) => onchange(clamp(Number(e.currentTarget.value)))}
		/>
	</div>
	{#if editable}
		<input
			class="readout mono"
			type="number"
			inputmode="numeric"
			{min}
			{max}
			value={live}
			aria-label={inputLabel || label}
			onchange={commitTyped}
			onkeydown={(e) => e.key === "Enter" && e.currentTarget.blur()}
		/>
	{:else}
		<span class="val mono">{format(live)}</span>
	{/if}
	{#if presets.length}
		<div class="presets">
			{#each presets as p (p)}
				<button
					class="mono"
					class:on={live === p}
					onclick={() => {
						live = p;
						onchange(p);
					}}>{p}</button
				>
			{/each}
		</div>
	{/if}
</div>

<style>
	.fader {
		display: grid;
		justify-items: center;
		align-content: start;
		gap: 10px;
		padding: 14px 8px 12px;
		background: var(--grain), var(--face);
		box-shadow: var(--bevel);
	}
	.lbl,
	.val {
		white-space: nowrap;
	}
	.row {
		display: flex;
		gap: 6px;
	}
	.scale {
		position: relative;
		width: 26px;
	}
	.scale span {
		position: absolute;
		right: 0;
		translate: 0 -50%;
		font: 400 10px var(--font-mono);
		color: var(--ink-3);
	}
	/* прорезь в панели и рифлёный бегунок */
	input[type="range"] {
		appearance: none;
		writing-mode: vertical-lr;
		direction: rtl;
		width: 30px;
		margin: 0;
		background: transparent;
		cursor: ns-resize;
	}
	input[type="range"]::-webkit-slider-runnable-track {
		width: 8px;
		margin: 0 auto;
		border-radius: 2px;
		background: var(--well);
		box-shadow: var(--recess);
	}
	input[type="range"]::-webkit-slider-thumb {
		appearance: none;
		width: 30px;
		height: 14px;
		margin-left: -11px;
		border-radius: 2px;
		background:
			repeating-linear-gradient(0deg, rgb(0 0 0 / 0.22) 0 1px, transparent 1px 3px),
			linear-gradient(var(--signal-hi), var(--signal));
		box-shadow:
			0 2px 4px rgb(0 0 0 / 0.45),
			inset 0 1px 0 rgb(255 255 255 / 0.35);
	}
	/* мини-индикатор с точным значением */
	.readout {
		width: 58px;
		padding: 4px 7px;
		text-align: right;
		color: var(--lcd-fg);
		background: var(--lcd-bg);
		border: 1px solid rgb(0 0 0 / 0.6);
		box-shadow: var(--recess);
		text-shadow: var(--glow-lcd);
		appearance: textfield;
	}
	.readout::-webkit-inner-spin-button,
	.readout::-webkit-outer-spin-button {
		appearance: none;
		margin: 0;
	}
	.readout:focus-visible {
		outline-offset: 1px;
	}
	.presets {
		display: flex;
		gap: 3px;
	}
	.presets button {
		padding: 2px 5px;
		font-size: 10px;
		color: var(--ink-2);
		background: var(--face-2);
		box-shadow: var(--lift);
	}
	.presets button.on {
		color: var(--signal-text);
		background: var(--well);
		box-shadow: var(--recess);
	}
</style>
