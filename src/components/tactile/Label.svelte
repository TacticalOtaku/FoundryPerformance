<script lang="ts">
	import type { Snippet } from "svelte";

	let { children, modified = false, id }: { children: Snippet; modified?: boolean; id?: string } = $props();
</script>

<span class="lbl" {id}>{@render children()}{#if modified}<i class="mod" aria-hidden="true"></i>{/if}</span>

<style>
	.lbl {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		white-space: nowrap;
		font: 500 10.5px/1.3 var(--tc-font-mono);
		color: var(--tc-muted);
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}
	.mod {
		width: 6px;
		height: 6px;
		border-radius: 99px;
		background: var(--tc-accent);
		box-shadow: 0 0 6px var(--tc-glow);
	}
</style>
