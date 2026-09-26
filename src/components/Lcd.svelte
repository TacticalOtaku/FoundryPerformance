<script lang="ts">
	let { value, unit, caption, digits = 3 }: { value: number | null; unit: string; caption?: string; digits?: number } = $props();

	const REEL = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
	const cells = $derived(
		value === null
			? Array.from({ length: digits }, () => null)
			: String(Math.min(10 ** digits - 1, Math.max(0, Math.round(value))))
					.padStart(digits, "0")
					.split("")
					.map(Number)
	);
</script>

<div class="lcd" role="img" aria-label={value === null ? `— ${unit}` : `${Math.round(value)} ${unit}`}>
	<div class="digits">
		{#each cells as d, i (i)}
			<span class="cell">
				{#if d === null}
					<span class="ghost">-</span>
				{:else}
					<span class="reel" style:transform={`translateY(${-d}em)`}>
						{#each REEL as n (n)}<span>{n}</span>{/each}
					</span>
				{/if}
			</span>
		{/each}
		<span class="unit">{unit}</span>
	</div>
	{#if caption}<div class="cap">{caption}</div>{/if}
</div>

<style>
	.lcd {
		background: var(--lcd-bg);
		color: var(--lcd-fg);
		padding: 12px 14px 10px;
		font-family: var(--font-mono);
	}
	.digits {
		display: flex;
		align-items: baseline;
		justify-content: flex-end;
		font-size: 36px;
		line-height: 1;
		font-variant-numeric: tabular-nums;
	}
	.cell {
		display: inline-block;
		width: 0.74em;
		height: 1em;
		overflow: hidden;
		text-align: center;
	}
	.reel {
		display: flex;
		flex-direction: column;
		transition: transform 0.45s var(--ease-out);
	}
	.reel span {
		height: 1em;
	}
	.ghost {
		color: var(--lcd-ghost);
	}
	.unit {
		margin-left: 6px;
		font-size: 11px;
		color: var(--lcd-dim);
	}
	.cap {
		margin-top: 6px;
		font-size: 11px;
		text-align: right;
		color: var(--lcd-dim);
	}
</style>
