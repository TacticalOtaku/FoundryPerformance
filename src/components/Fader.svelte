<script lang="ts">
	let {
		label,
		value,
		min,
		max,
		step,
		format,
		modified = false,
		onchange
	}: {
		label: string;
		value: number;
		min: number;
		max: number;
		step: number;
		format: (v: number) => string;
		modified?: boolean;
		onchange: (v: number) => void;
	} = $props();

	// Пока бегунок тянут, значение только показывается; сохраняется на `change`
	let live = $state(0);
	$effect(() => {
		live = value;
	});
</script>

<label class="fader">
	<span class="lbl mono">{label}{#if modified}<i class="dot" aria-hidden="true"></i>{/if}</span>
	<input
		type="range"
		{min}
		{max}
		{step}
		value={live}
		aria-valuetext={format(live)}
		oninput={(e) => (live = Number(e.currentTarget.value))}
		onchange={(e) => onchange(Number(e.currentTarget.value))}
	/>
	<span class="val mono">{format(live)}</span>
</label>

<style>
	.fader {
		display: grid;
		justify-items: center;
		align-content: start;
		gap: 10px;
		padding: 14px 8px 12px;
		background: var(--face);
	}
	.lbl,
	.val {
		white-space: nowrap;
	}
	input {
		appearance: none;
		writing-mode: vertical-lr;
		direction: rtl;
		width: 28px;
		height: 108px;
		margin: 0;
		background: linear-gradient(var(--ink), var(--ink)) center / 6px 100% no-repeat;
		cursor: ns-resize;
	}
	input::-webkit-slider-runnable-track {
		background: transparent;
	}
	input::-webkit-slider-thumb {
		appearance: none;
		width: 28px;
		height: 13px;
		background: var(--signal);
		border: 1px solid var(--signal-ink);
	}
</style>
