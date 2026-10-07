<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";

	type Props = HTMLAttributes<HTMLDivElement> & { children: Snippet; pad?: string; fill?: boolean };
	let { children, pad = "16px", fill = false, ...rest }: Props = $props();
</script>

<!-- двойной кант: утопленный лоток вокруг поднятой пластины; радиусы концентричны -->
<div class="shell" class:fill {...rest}>
	<div class="core" style:padding={pad}>{@render children()}</div>
</div>

<style>
	.shell {
		min-width: 0;
		min-height: 0;
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		grid-template-rows: minmax(0, 1fr);
		padding: 6px;
		border-radius: calc(var(--tc-r-block) + 6px);
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
	}
	.fill {
		height: 100%;
	}
	.core {
		min-width: 0;
		min-height: 0;
		display: flex;
		flex-direction: column;
		border-radius: var(--tc-r-block);
		background: var(--tc-surface);
		box-shadow: var(--tc-raise);
	}
</style>
