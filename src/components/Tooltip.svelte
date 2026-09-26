<script lang="ts">
	import { t } from "../lib/i18n.svelte";
	import { placeTip, type TipPlacement } from "../lib/tip-place";
	import { TIP_ID, tipState } from "../lib/tooltip.svelte";

	let el = $state<HTMLDivElement>();
	let pos = $state<TipPlacement>({ x: -9999, y: -9999, side: "top", arrow: 0 });

	// Размер таблички известен только после отрисовки — меряем и ставим на место
	$effect(() => {
		const cur = tipState.current;
		if (!cur || !el) return;
		const r = cur.rect;
		pos = placeTip({ x: r.left, y: r.top, w: r.width, h: r.height }, { w: el.offsetWidth, h: el.offsetHeight }, { w: innerWidth, h: innerHeight });
	});

	$effect(() => {
		const hide = () => tipState.hide();
		window.addEventListener("scroll", hide, true);
		window.addEventListener("blur", hide);
		return () => {
			window.removeEventListener("scroll", hide, true);
			window.removeEventListener("blur", hide);
		};
	});

	const bars = (n: number) => Array.from({ length: 5 }, (_, i) => i < n);
</script>

{#if tipState.current}
	{@const d = tipState.current.data}
	<div
		bind:this={el}
		id={TIP_ID}
		role="tooltip"
		class="tip {pos.side}"
		style:left={`${pos.x}px`}
		style:top={`${pos.y}px`}
		style:--arrow={`${pos.arrow}px`}
	>
		<b class:solo={!d.body}>{d.title}</b>
		{#if d.body}<p>{d.body}</p>{/if}
		{#if d.fps !== undefined || d.look !== undefined}
			<div class="meters mono">
				{#if d.fps !== undefined}
					<span class="meter">
						{t("tip.meter.fps")}
						<span class="bars fps" aria-label={`${d.fps}/5`}>{#each bars(d.fps) as on, i (i)}<i class:on></i>{/each}</span>
					</span>
				{/if}
				{#if d.look !== undefined}
					<span class="meter">
						{t("tip.meter.look")}
						<span class="bars look" aria-label={`${d.look}/5`}>{#each bars(d.look) as on, i (i)}<i class:on></i>{/each}</span>
					</span>
				{/if}
			</div>
		{/if}
	</div>
{/if}

<style>
	.tip {
		position: fixed;
		z-index: 100;
		width: max-content;
		max-width: 290px;
		padding: 10px 12px;
		background: var(--inverse-bg);
		color: var(--inverse-fg);
		pointer-events: none;
		animation: appear 0.12s var(--ease-out);
	}
	.tip::after {
		content: "";
		position: absolute;
		left: calc(var(--arrow) - 5px);
		border: 5px solid transparent;
	}
	.top::after {
		top: 100%;
		border-top-color: var(--inverse-bg);
	}
	.bottom::after {
		bottom: 100%;
		border-bottom-color: var(--inverse-bg);
	}
	b {
		display: block;
		font-weight: 500;
		margin-bottom: 4px;
	}
	b.solo {
		margin-bottom: 0;
	}
	p {
		margin: 0;
		font-size: 12px;
		line-height: 1.45;
		color: var(--ink-3);
	}
	.meters {
		display: grid;
		grid-template-columns: 1fr auto;
		align-items: center;
		gap: 5px 16px;
		margin-top: 8px;
		padding-top: 8px;
		border-top: 1px solid var(--ink-2);
		white-space: nowrap;
	}
	.meter {
		display: contents;
	}
	.bars {
		display: inline-flex;
		gap: 2px;
	}
	.bars i {
		width: 5px;
		height: 10px;
		background: var(--ink-2);
	}
	.fps i.on {
		background: var(--led-ok);
	}
	.look i.on {
		background: var(--signal);
	}
	@keyframes appear {
		from {
			opacity: 0;
		}
	}
</style>
