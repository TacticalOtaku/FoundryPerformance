<script lang="ts" generics="T extends string | number">
	import type { TipData } from "../lib/tips";
	import { tip as tooltip } from "../lib/tooltip.svelte";

	let {
		label,
		options,
		value,
		modified = false,
		vertical = false,
		tip,
		onchange
	}: {
		label: string;
		options: { value: T; label: string }[];
		value: T;
		modified?: boolean;
		vertical?: boolean;
		tip?: TipData;
		onchange: (v: T) => void;
	} = $props();
</script>

<div class="seg" class:vertical role="radiogroup" aria-label={label} {@attach tooltip(() => tip)}>
	<span class="lbl silk">{label}{#if modified}<i class="dot" aria-hidden="true"></i>{/if}</span>
	<div class="opts">
		{#each options as o (o.value)}
			<button class="mono" role="radio" aria-checked={o.value === value} class:on={o.value === value} onclick={() => onchange(o.value)}>
				{o.label}
			</button>
		{/each}
	</div>
</div>

<style>
	.seg {
		display: grid;
		gap: 7px;
	}
	.opts {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}
	.vertical {
		justify-items: center;
		align-content: start;
		padding: 14px 8px 12px;
		background: var(--grain), var(--face);
		box-shadow: var(--bevel);
	}
	.vertical .opts {
		flex-direction: column;
		width: 78px;
	}
	/* клавиши: отжатые выпуклые, нажатая утоплена и подсвечена */
	button {
		position: relative;
		padding: 5px 9px;
		color: var(--ink-2);
		background: var(--face-2);
		box-shadow: var(--lift);
		text-align: center;
		transition:
			color 0.12s,
			box-shadow 0.12s;
	}
	button:hover {
		color: var(--ink);
	}
	button.on {
		color: var(--signal-text);
		background: var(--well);
		box-shadow: var(--recess);
	}
	button.on::before {
		content: "";
		position: absolute;
		left: 25%;
		right: 25%;
		top: 2px;
		height: 2px;
		border-radius: 1px;
		background: var(--signal);
		box-shadow: var(--glow-signal);
	}
</style>
