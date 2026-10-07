<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";

	type Props = HTMLAttributes<HTMLDivElement> & { children: Snippet; pad?: string; fill?: boolean; scroll?: boolean };
	let { children, pad = "16px", fill = false, scroll = false, ...rest }: Props = $props();
</script>

<div class="core" class:fill class:scroll style:padding={pad} {...rest}>{@render children()}</div>

<style>
	.core {
		min-width: 0;
		min-height: 0;
		display: flex;
		flex-direction: column;
		border-radius: var(--tc-r-block);
		background: var(--tc-surface);
		box-shadow: var(--tc-raise);
	}
	.fill {
		height: 100%;
	}
	.scroll {
		overflow-y: auto;
	}
</style>
