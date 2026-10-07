<script lang="ts">
	import { reduceMotion, TACTILE } from "../../lib/motion";

	type Props = { value: number; ghost?: number | null; stops?: number[]; label?: string; animate?: boolean };
	let { value, ghost = null, stops = [], label, animate = true }: Props = $props();

	const pct = (v: number) => `${Math.max(0, Math.min(1, v)) * 100}%`;

	/** Шкала доливается по ширине за 350 ms, как в AIM. */
	function fill(el: HTMLElement, v: number) {
		let cur = v;
		return {
			update(next: number) {
				if (next === cur) return;
				const from = pct(cur);
				cur = next;
				if (animate && !reduceMotion()) el.animate([{ width: from }, { width: pct(next) }], { duration: 350, easing: TACTILE });
			}
		};
	}
</script>

<div
	class="band"
	role={label ? "meter" : undefined}
	aria-label={label}
	aria-valuemin={label ? 0 : undefined}
	aria-valuemax={label ? 100 : undefined}
	aria-valuenow={label ? Math.round(Math.max(0, Math.min(1, value)) * 100) : undefined}
>
	{#if ghost !== null}<span class="ghost" style:width={pct(ghost)}></span>{/if}
	<span class="fill" style:width={pct(value)} use:fill={value}></span>
	{#each stops as s (s)}<span class="stop" style:left={pct(s)}></span>{/each}
</div>

<style>
	.band {
		position: relative;
		height: 6px;
		overflow: hidden;
		border-radius: 99px;
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
	}
	.fill,
	.ghost {
		position: absolute;
		left: 0;
		top: 0;
		height: 100%;
		border-radius: 99px;
		background: var(--tc-accent);
		box-shadow: 0 0 10px var(--tc-glow);
	}
	.ghost {
		background: var(--tc-muted);
		box-shadow: none;
		opacity: 0.35;
	}
	.stop {
		position: absolute;
		top: 0;
		bottom: 0;
		width: 2px;
		margin-left: -1px;
		background: var(--tc-surface);
	}
</style>
