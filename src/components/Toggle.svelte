<script lang="ts">
	import type { TipData } from "../lib/tips";
	import { tip as tooltip } from "../lib/tooltip.svelte";

	let {
		label,
		checked,
		modified = false,
		tip,
		onchange
	}: { label: string; checked: boolean; modified?: boolean; tip?: TipData; onchange: (v: boolean) => void } = $props();
</script>

<button class="toggle" use:tooltip={tip} role="switch" aria-checked={checked} onclick={() => onchange(!checked)}>
	<span>{label}{#if modified}<i class="dot" aria-hidden="true"></i>{/if}</span>
	<span class="sw" class:on={checked}><i></i></span>
</button>

<style>
	.toggle {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		width: 100%;
		padding: 4px 0;
		text-align: left;
	}
	.sw {
		position: relative;
		flex: none;
		width: 32px;
		height: 16px;
		background: var(--inverse-bg);
	}
	.sw i {
		position: absolute;
		top: 2px;
		left: 2px;
		width: 12px;
		height: 12px;
		background: var(--inverse-fg);
		transition:
			transform 0.14s var(--ease-detent),
			background 0.14s;
	}
	.sw.on i {
		transform: translateX(16px);
		background: var(--signal);
	}
</style>
