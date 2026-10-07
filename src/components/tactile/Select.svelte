<script lang="ts" generics="T extends string | number">
	import type { TipData } from "../../lib/tips";
	import { tip as tooltip } from "../../lib/tooltip.svelte";

	type Props = { label: string; options: { value: T; label: string }[]; value: T; onchange: (v: T) => void; tip?: TipData };
	let { label, options, value, onchange, tip }: Props = $props();
	const id = $props.id();
</script>

<div class="sel" {@attach tooltip(() => tip)}>
	<label class="lbl" for={id}>{label}</label>
	<select {id} onchange={(e) => onchange(options[e.currentTarget.selectedIndex].value)}>
		{#each options as o (o.value)}<option value={String(o.value)} selected={o.value === value}>{o.label}</option>{/each}
	</select>
</div>

<style>
	.sel {
		display: grid;
		gap: 8px;
		min-width: 0;
	}
	.lbl {
		font: 500 10.5px/1.3 var(--tc-font-mono);
		color: var(--tc-muted);
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}
	select {
		height: 34px;
		padding: 0 30px 0 14px;
		border: 0;
		border-radius: var(--tc-r-ctl);
		appearance: none;
		cursor: pointer;
		font-size: 12.5px;
		font-weight: 600;
		background-color: var(--tc-surface);
		box-shadow: var(--tc-raise);
		background-image:
			linear-gradient(45deg, transparent 50%, var(--tc-muted) 50%),
			linear-gradient(135deg, var(--tc-muted) 50%, transparent 50%);
		background-position:
			right 16px center,
			right 11px center;
		background-size: 5px 5px;
		background-repeat: no-repeat;
	}
</style>
