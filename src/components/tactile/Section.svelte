<script lang="ts">
	import type { Snippet } from "svelte";
	import Label from "./Label.svelte";

	let { title, children }: { title: string; children?: Snippet } = $props();
</script>

<section class="sec">
	<div class="head"><Label>{title}</Label></div>
	{#if children}<div class="body">{@render children()}</div>{/if}
</section>

<style>
	.sec {
		display: grid;
		gap: 12px;
	}
	.head {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.head::after {
		content: "";
		flex: 1;
		height: 1px;
		background: var(--tc-line);
	}
	.body {
		display: grid;
		gap: 12px;
	}
</style>
