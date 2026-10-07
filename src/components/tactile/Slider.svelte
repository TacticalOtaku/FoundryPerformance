<script lang="ts">
	import type { TipData } from "../../lib/tips";
	import { tip as tooltip } from "../../lib/tooltip.svelte";
	import Band from "./Band.svelte";
	import Chip from "./Chip.svelte";
	import Label from "./Label.svelte";
	import Ticks from "./Ticks.svelte";

	type Props = {
		label: string;
		value: number;
		min: number;
		max: number;
		step: number;
		format: (v: number) => string;
		onchange: (v: number) => void;
		modified?: boolean;
		tip?: TipData;
		/** Подписи под шкалой. */
		ticks?: number[];
		/** Быстрые значения под шкалой. */
		presets?: number[];
		/** Точный ввод числа вместо подписи значения. */
		editable?: boolean;
		inputLabel?: string;
	};
	let { label, value, min, max, step, format, onchange, modified = false, tip, ticks = [], presets = [], editable = false, inputLabel = "" }: Props = $props();

	const id = $props.id();
	// Пока ручку тянут, значение только показывается; сохраняется на `change`.
	let live = $derived(value);
	const frac = (v: number) => (v - min) / (max - min);
	const clamp = (v: number) => Math.min(max, Math.max(min, Math.round(v / step) * step));

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

<div class="slider" {@attach tooltip(() => tip)}>
	<div class="head">
		<Label {modified} id={`${id}-l`}>{label}</Label>
		{#if editable}
			<input
				class="readout"
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
			<span class="val">{format(live)}</span>
		{/if}
	</div>
	<div class="track">
		<Band value={frac(live)} animate={false} />
		<span class="thumb" style:left={`${frac(live) * 100}%`} aria-hidden="true"></span>
		<input
			type="range"
			{min}
			{max}
			{step}
			value={live}
			aria-labelledby={`${id}-l`}
			aria-valuetext={format(live)}
			oninput={(e) => (live = Number(e.currentTarget.value))}
			onchange={(e) => onchange(clamp(Number(e.currentTarget.value)))}
		/>
	</div>
	{#if ticks.length}<Ticks items={ticks.map((v) => ({ at: frac(v), label: String(v) }))} />{/if}
	{#if presets.length}
		<div class="presets">
			{#each presets as p (p)}
				<Chip
					button
					on={live === p}
					onclick={() => {
						live = p;
						onchange(p);
					}}>{p}</Chip
				>
			{/each}
		</div>
	{/if}
</div>

<style>
	.slider {
		display: grid;
		gap: 8px;
		min-width: 0;
	}
	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		min-height: 24px;
	}
	.val,
	.readout {
		font: 600 12.5px/1 var(--tc-font-mono);
		color: var(--tc-ink);
	}
	.readout {
		width: 60px;
		height: 24px;
		padding: 0 10px;
		border: 0;
		border-radius: var(--tc-r-ctl);
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
		text-align: right;
	}
	.track {
		position: relative;
		display: grid;
		align-items: center;
		height: 16px;
	}
	.thumb {
		position: absolute;
		top: 0;
		width: 16px;
		height: 16px;
		margin-left: -8px;
		border-radius: 99px;
		background: var(--tc-surface);
		box-shadow: var(--tc-raise);
		pointer-events: none;
	}
	input[type="range"] {
		position: absolute;
		inset: 0;
		width: 100%;
		margin: 0;
		opacity: 0;
		cursor: pointer;
	}
	.track:has(input:focus-visible) .thumb {
		outline: 2px solid var(--tc-accent);
		outline-offset: 2px;
	}
	.presets {
		display: flex;
		gap: 4px;
	}
</style>
