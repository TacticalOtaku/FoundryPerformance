<script lang="ts">
	import { onMount } from "svelte";

	let {
		value,
		unit,
		caption,
		digits = 3,
		history = []
	}: { value: number | null; unit: string; caption?: string; digits?: number; history?: number[] } = $props();

	const REEL = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

	// «Прогрев» как у настоящего индикатора: все сегменты горят, затем значение
	let warming = $state(true);
	onMount(() => {
		if (matchMedia("(prefers-reduced-motion: reduce)").matches) {
			warming = false;
			return;
		}
		const id = setTimeout(() => (warming = false), 520);
		return () => clearTimeout(id);
	});

	const cells = $derived(
		warming
			? Array.from({ length: digits }, () => 8)
			: value === null
				? Array.from({ length: digits }, () => null)
				: String(Math.min(10 ** digits - 1, Math.max(0, Math.round(value))))
						.padStart(digits, "0")
						.split("")
						.map(Number)
	);
	const peak = $derived(Math.max(1, ...history));
</script>

<div class="lcd" role="img" aria-label={value === null ? `— ${unit}` : `${Math.round(value)} ${unit}`}>
	<div class="digits">
		{#each cells as d, i (i)}
			<span class="cell">
				{#if d === null}
					<span class="dash">-</span>
				{:else}
					<span class="reel" style:translate={`0 ${-d}em`}>
						{#each REEL as n (n)}<span>{n}</span>{/each}
					</span>
				{/if}
			</span>
		{/each}
		<span class="unit">{unit}</span>
	</div>
	{#if history.length > 1}
		<div class="spark" aria-hidden="true">
			{#each history as h, i (i)}
				<i class:last={i === history.length - 1} style:height={`${Math.max(8, (h / peak) * 100)}%`}></i>
			{/each}
		</div>
	{/if}
	{#if caption}<div class="cap">{caption}</div>{/if}
</div>

<style>
	.lcd {
		position: relative;
		overflow: hidden;
		padding: 12px 14px 10px;
		background: var(--lcd-bg);
		color: var(--lcd-fg);
		font-family: var(--font-mono);
		border: 1px solid rgb(0 0 0 / 0.6);
		box-shadow: var(--recess), inset 0 2px 10px rgb(0 0 0 / 0.6);
	}
	/* строки развёртки поверх стекла */
	.lcd::after {
		content: "";
		position: absolute;
		inset: 0;
		pointer-events: none;
		background: repeating-linear-gradient(0deg, rgb(0 0 0 / 0.16) 0 1px, transparent 1px 3px);
	}
	.digits {
		position: relative;
		display: flex;
		align-items: baseline;
		justify-content: flex-end;
		font-size: 38px;
		line-height: 1;
		font-variant-numeric: tabular-nums;
		text-shadow: var(--glow-lcd);
	}
	.cell {
		position: relative;
		display: inline-block;
		width: 0.74em;
		height: 1em;
		overflow: hidden;
		text-align: center;
	}
	/* непогашенный сегмент под каждой цифрой */
	.cell::before {
		content: "8";
		position: absolute;
		inset: 0;
		color: var(--lcd-ghost);
		text-shadow: none;
	}
	.reel,
	.dash {
		position: relative;
	}
	.reel {
		display: flex;
		flex-direction: column;
		transition: translate 0.45s var(--ease-out);
	}
	.reel span {
		height: 1em;
	}
	.dash {
		color: var(--lcd-dim);
	}
	.unit {
		margin-left: 6px;
		font-size: 11px;
		color: var(--lcd-dim);
		text-shadow: none;
	}
	.spark {
		position: relative;
		display: flex;
		align-items: flex-end;
		gap: 3px;
		height: 16px;
		margin-top: 8px;
	}
	.spark i {
		flex: 1;
		background: var(--lcd-dim);
		opacity: 0.55;
	}
	.spark i.last {
		background: var(--lcd-fg);
		opacity: 1;
		box-shadow: var(--glow-lcd);
	}
	.cap {
		position: relative;
		margin-top: 6px;
		font-size: 11px;
		text-align: right;
		color: var(--lcd-dim);
	}
</style>
