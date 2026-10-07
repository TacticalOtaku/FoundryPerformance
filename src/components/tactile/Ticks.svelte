<script lang="ts">
	let { items }: { items: { at: number; label: string }[] } = $props();
	const shift = (at: number) => (at <= 0 ? "none" : at >= 1 ? "translateX(-100%)" : "translateX(-50%)");
</script>

<div class="ticks" aria-hidden="true">
	{#each items as it (it.label)}<span style:left={`${it.at * 100}%`} style:transform={shift(it.at)}>{it.label}</span>{/each}
</div>

<style>
	.ticks {
		position: relative;
		height: 11px;
		font: 500 9.5px/1 var(--tc-font-mono);
		color: var(--tc-muted);
	}
	.ticks span {
		position: absolute;
		top: 0;
	}
</style>
