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

<div class="seg" class:vertical role="radiogroup" aria-label={label} use:tooltip={tip}>
	<span class="lbl mono">{label}{#if modified}<i class="dot" aria-hidden="true"></i>{/if}</span>
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
		gap: 6px;
	}
	.opts {
		display: flex;
		flex-wrap: wrap;
		gap: 3px;
	}
	.vertical {
		justify-items: center;
		align-content: start;
		padding: 14px 8px 12px;
		background: var(--face);
	}
	.vertical .opts {
		flex-direction: column;
		width: 76px;
	}
	button {
		padding: 4px 8px;
		border: 1px solid var(--ink-3);
		text-align: center;
	}
	button.on {
		background: var(--inverse-bg);
		color: var(--inverse-fg);
		border-color: var(--inverse-bg);
	}
</style>
