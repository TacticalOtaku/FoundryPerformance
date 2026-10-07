<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";
	import type { TipData } from "../../lib/tips";
	import { tip as tooltip } from "../../lib/tooltip.svelte";

	type Props = HTMLAttributes<HTMLElement> & { children: Snippet; button?: boolean; on?: boolean; tip?: TipData };
	let { children, button = false, on = false, tip, ...rest }: Props = $props();
</script>

<svelte:element this={button ? "button" : "span"} type={button ? "button" : undefined} class="chip" class:on {@attach tooltip(() => tip)} {...rest}>
	{@render children()}
</svelte:element>

<style>
	.chip {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		height: 20px;
		padding: 0 7px;
		flex: none;
		border-radius: 99px;
		background: var(--tc-sunken);
		color: var(--tc-muted);
		font: 500 10px/1 var(--tc-font-mono);
		white-space: nowrap;
		transition: color 160ms var(--tc-ease);
	}
	button.chip:hover {
		color: var(--tc-ink);
	}
	.on {
		background: var(--tc-accent-soft);
		color: var(--tc-accent-text);
	}
</style>
