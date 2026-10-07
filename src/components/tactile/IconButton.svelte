<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLButtonAttributes } from "svelte/elements";
	import { press } from "../../lib/motion";
	import type { TipData } from "../../lib/tips";
	import { tip as tooltip } from "../../lib/tooltip.svelte";

	type Props = HTMLButtonAttributes & { label: string; children: Snippet; size?: 28 | 30; on?: boolean; danger?: boolean; tip?: TipData };
	let { label, children, size = 28, on = false, danger = false, tip, ...rest }: Props = $props();
</script>

<button type="button" class="ib" class:on class:danger style:--s={`${size}px`} aria-label={label} aria-pressed={on ? true : undefined} use:press {@attach tooltip(() => tip)} {...rest}>
	{@render children()}
</button>

<style>
	.ib {
		width: var(--s);
		height: var(--s);
		display: grid;
		place-items: center;
		flex: none;
		border-radius: var(--tc-r-ctl);
		color: var(--tc-muted);
		transition:
			color 160ms var(--tc-ease),
			box-shadow 160ms var(--tc-ease);
	}
	.ib:hover {
		color: var(--tc-ink);
		box-shadow: var(--tc-raise);
	}
	.on {
		color: var(--tc-accent-text);
	}
	.danger:hover {
		color: var(--tc-danger);
	}
</style>
