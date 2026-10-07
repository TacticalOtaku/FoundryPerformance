<script lang="ts">
	import { t } from "../../lib/i18n.svelte";
	import { ACCENTS } from "../../lib/palette";
	import { nextIndex } from "../../lib/radio";
	import { tip as tooltip } from "../../lib/tooltip.svelte";
	import type { AccentId } from "../../lib/types";
	import Label from "./Label.svelte";

	let { label, value, onchange }: { label: string; value: AccentId; onchange: (v: AccentId) => void } = $props();
	const btns: HTMLButtonElement[] = [];

	function onkey(e: KeyboardEvent, i: number) {
		const n = nextIndex(i, e.key, ACCENTS.map(() => true));
		if (n === null) return;
		e.preventDefault();
		onchange(ACCENTS[n].id);
		btns[n]?.focus();
	}
</script>

<div class="wrap">
	<Label>{label}</Label>
	<div class="sw" role="radiogroup" aria-label={label}>
		{#each ACCENTS as a, i (a.id)}
			<button
				type="button"
				class="dot"
				class:on={a.id === value}
				role="radio"
				aria-checked={a.id === value}
				aria-label={t(`accent.${a.id}`)}
				tabindex={a.id === value ? 0 : -1}
				style={`--h: ${a.h}; --c: ${a.c}`}
				bind:this={btns[i]}
				{@attach tooltip(() => ({ title: t(`accent.${a.id}`), body: "" }))}
				onclick={() => onchange(a.id)}
				onkeydown={(e) => onkey(e, i)}
			></button>
		{/each}
	</div>
</div>

<style>
	.wrap {
		display: grid;
		gap: 8px;
	}
	.sw {
		display: flex;
		flex-wrap: wrap;
		gap: 10px;
	}
	.dot {
		width: 24px;
		height: 24px;
		border-radius: 99px;
		background: oklch(0.73 var(--c) var(--h));
		box-shadow:
			inset 0 1px 0 rgb(255 255 255 / 40%),
			var(--tc-raise);
		transition:
			box-shadow 160ms var(--tc-ease),
			transform 160ms var(--tc-ease);
	}
	:global(.tc-root[data-theme="dark"]) .dot {
		background: oklch(0.78 var(--c) var(--h));
	}
	.dot:hover {
		transform: translateY(-1px);
	}
	.dot.on {
		box-shadow:
			0 0 0 2px var(--tc-surface),
			0 0 0 4px var(--tc-ink);
	}
</style>
