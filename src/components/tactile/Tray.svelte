<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";

	type Props = HTMLAttributes<HTMLDivElement> & { children: Snippet; pad?: string; fill?: boolean; scroll?: boolean };
	let { children, pad = "6px", fill = false, scroll = false, ...rest }: Props = $props();
</script>

<!-- лоток для списков: внутри плоские строки, выступает только выбранная -->
<div class="tray" class:fill class:scroll style:padding={pad} {...rest}>{@render children()}</div>

<style>
	.tray {
		min-width: 0;
		min-height: 0;
		display: flex;
		flex-direction: column;
		border-radius: var(--tc-r-block);
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
	}
	.fill {
		height: 100%;
	}
	.scroll {
		overflow-y: auto;
	}
</style>
