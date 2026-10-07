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
		<b class:solo={!d.body && !d.lines?.length}>{d.title}</b>
		{#if d.body}<p>{d.body}</p>{/if}
		{#if d.lines?.length}
			<ul>
				{#each d.lines as line, i (i)}<li>{line}</li>{/each}
			</ul>
		{/if}
		{#if d.checks?.length}
			<ul class="checks">
				{#each d.checks as c, i (i)}<li><i class:ok={c.ok} aria-hidden="true"></i>{c.text}</li>{/each}
			</ul>
		{/if}
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
		display: grid;
		gap: 6px;
		max-width: 300px;
		padding: 10px 12px;
		border-radius: var(--tc-r-block);
		background: var(--tc-surface);
		box-shadow:
			var(--tc-raise),
			0 18px 40px -16px rgb(0 0 0 / 40%);
		color: var(--tc-ink);
		font-size: 12.5px;
		line-height: 1.4;
		pointer-events: none;
		animation: tip-in 160ms var(--tc-ease);
	}
	b {
		font-weight: 600;
	}
	p {
		color: var(--tc-muted);
	}
	ul {
		display: grid;
		gap: 3px;
		list-style: none;
		color: var(--tc-muted);
	}
	ul:not(.checks) li::before {
		content: "– ";
	}
	.checks {
		color: var(--tc-ink);
	}
	.checks li {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.checks i {
		width: 7px;
		height: 7px;
		flex: none;
		border-radius: 99px;
		background: var(--tc-danger);
	}
	.checks i.ok {
		background: var(--tc-good);
	}
	.meters {
		display: flex;
		gap: 14px;
		margin-top: 2px;
		font: 500 9.5px/1 var(--tc-font-mono);
		letter-spacing: 0.06em;
		color: var(--tc-muted);
	}
	.meter {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}
	.bars {
		display: inline-flex;
		gap: 2px;
	}
	.bars i {
		width: 5px;
		height: 10px;
		border-radius: 2px;
		background: var(--tc-sunken);
	}
	.bars.fps i.on {
		background: var(--tc-accent);
	}
	.bars.look i.on {
		background: var(--tc-muted);
	}
	@keyframes tip-in {
		from {
			opacity: 0;
			transform: translateY(2px);
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.tip {
			animation: none;
		}
	}
</style>
