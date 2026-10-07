<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";
	import type { TipData } from "../../lib/tips";
	import { tip as tooltip } from "../../lib/tooltip.svelte";

	type Props = HTMLAttributes<HTMLElement> & { children: Snippet; tone?: "accent" | "plain" | "warn" | "good" | "danger"; button?: boolean; tip?: TipData };
	let { children, tone = "accent", button = false, tip, ...rest }: Props = $props();
</script>

<svelte:element this={button ? "button" : "span"} type={button ? "button" : undefined} class="pill {tone}" {@attach tooltip(() => tip)} {...rest}>
	{@render children()}
</svelte:element>

<style>
	.pill {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		height: 18px;
		padding: 0 7px;
		flex: none;
		border-radius: 99px;
		font: 600 9.5px/1 var(--tc-font-mono);
		letter-spacing: 0.05em;
		text-transform: uppercase;
		white-space: nowrap;
	}
	.accent {
		background: var(--tc-accent-soft);
		color: var(--tc-accent-text);
	}
	.plain {
		color: var(--tc-muted);
		box-shadow: inset 0 0 0 1px var(--tc-line);
	}
	.warn {
		background: color-mix(in oklab, var(--tc-warning) 14%, transparent);
		color: var(--tc-warning);
	}
	.good {
		background: color-mix(in oklab, var(--tc-good) 14%, transparent);
		color: var(--tc-good);
	}
	.danger {
		background: color-mix(in oklab, var(--tc-danger) 14%, transparent);
		color: var(--tc-danger);
	}
	button.pill {
		height: 22px;
		padding: 0 9px;
		transition: box-shadow 160ms var(--tc-ease);
	}
	button.pill:hover {
		box-shadow: var(--tc-raise);
	}
</style>
