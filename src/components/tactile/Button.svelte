<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLButtonAttributes } from "svelte/elements";
	import { press } from "../../lib/motion";
	import type { TipData } from "../../lib/tips";
	import { tip as tooltip } from "../../lib/tooltip.svelte";

	type Props = HTMLButtonAttributes & { children: Snippet; danger?: boolean; armed?: boolean; tip?: TipData };
	let { children, danger = false, armed = false, tip, type = "button", ...rest }: Props = $props();
</script>

<button {type} class="btn" class:danger class:armed use:press {@attach tooltip(() => tip)} {...rest}>{@render children()}</button>

<style>
	.btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		height: 34px;
		padding: 0 14px;
		border-radius: var(--tc-r-ctl);
		background: var(--tc-surface);
		box-shadow: var(--tc-raise);
		font-size: 12.5px;
		font-weight: 600;
		white-space: nowrap;
		transition: color 160ms var(--tc-ease);
	}
	.btn:hover:not(:disabled) {
		color: var(--tc-accent-text);
	}
	.danger:hover:not(:disabled),
	.armed {
		color: var(--tc-danger);
	}
	.btn:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}
	.btn[aria-busy="true"] {
		cursor: progress;
	}
</style>
