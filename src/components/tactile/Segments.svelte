<script lang="ts" module>
	export interface SegOption<V> {
		value: V;
		label: string;
		/** Короткая подпись неактивного сегмента; у активного — полная. */
		short?: string;
		/** Стрелки пропускают пункт: его выбирают только мышью. */
		arrowSkip?: boolean;
		disabled?: boolean;
	}
</script>

<script lang="ts" generics="T extends string | number">
	import { nextIndex } from "../../lib/radio";
	import type { TipData } from "../../lib/tips";
	import { tip as tooltip } from "../../lib/tooltip.svelte";
	import Label from "./Label.svelte";

	type Props = { label: string; options: SegOption<T>[]; value: T; onchange: (v: T) => void; showLabel?: boolean; modified?: boolean; tip?: TipData };
	let { label, options, value, onchange, showLabel = true, modified = false, tip }: Props = $props();

	const btns: HTMLButtonElement[] = [];
	const current = $derived(options.findIndex((o) => o.value === value));
	const focusable = $derived(current >= 0 ? current : options.findIndex((o) => !o.disabled));
	const wide = $derived(options.some((o) => o.short));

	function onkey(e: KeyboardEvent, i: number) {
		const n = nextIndex(i, e.key, options.map((o) => !o.disabled && !o.arrowSkip));
		if (n === null) return;
		e.preventDefault();
		onchange(options[n].value);
		btns[n]?.focus();
	}
</script>

<div class="wrap" {@attach tooltip(() => tip)}>
	{#if showLabel}<Label {modified}>{label}</Label>{/if}
	<div class="segs" class:wide role="radiogroup" aria-label={label}>
		{#each options as o, i (o.value)}
			<button
				type="button"
				class="seg"
				class:on={i === current}
				role="radio"
				aria-checked={i === current}
				tabindex={i === focusable ? 0 : -1}
				disabled={o.disabled}
				bind:this={btns[i]}
				onclick={() => onchange(o.value)}
				onkeydown={(e) => onkey(e, i)}
			>
				{i === current || !o.short ? o.label : o.short}
			</button>
		{/each}
	</div>
</div>

<style>
	.wrap {
		display: grid;
		gap: 8px;
		min-width: 0;
	}
	.segs {
		display: flex;
		gap: 2px;
		min-width: 0;
		padding: 3px;
		border-radius: var(--tc-r-ctl);
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
	}
	.seg {
		flex: 1 1 0;
		min-width: 0;
		height: 30px;
		padding: 0 10px;
		overflow: hidden;
		border-radius: var(--tc-r-ctl);
		color: var(--tc-muted);
		font-size: 12.5px;
		font-weight: 600;
		white-space: nowrap;
		text-overflow: ellipsis;
		transition:
			color 160ms var(--tc-ease),
			box-shadow 240ms var(--tc-ease),
			background-color 240ms var(--tc-ease);
	}
	.seg:hover:not(:disabled) {
		color: var(--tc-ink);
	}
	.seg.on {
		background: var(--tc-surface);
		color: var(--tc-ink);
		box-shadow: var(--tc-raise);
	}
	.seg:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}
	.wide .seg {
		font: 600 11.5px/1 var(--tc-font-mono);
		letter-spacing: 0.05em;
	}
	.wide .seg.on {
		flex: 1.7 1 0;
		font: 600 13px/1 var(--tc-font-ui);
		letter-spacing: 0;
	}
</style>
