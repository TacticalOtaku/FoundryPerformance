<script lang="ts">
	import { tip } from "../lib/tooltip.svelte";
	import type { LedState } from "../lib/types";

	let { state, label }: { state: LedState; label: string } = $props();
</script>

<span class="hit" use:tip={label ? { title: label, body: "" } : undefined}>
	<span class="led {state}" role="img" aria-label={label}></span>
</span>

<style>
	/* Кружок 10 px — наводить на него трудно, поэтому зона наведения шире */
	.hit {
		display: grid;
		place-items: center;
		width: 22px;
		height: 22px;
		margin: -6px;
	}
	.led {
		flex: none;
		width: 10px;
		height: 10px;
		border-radius: 50%;
		background: var(--led-off);
	}
	.ok {
		background: var(--led-ok);
	}
	.warn {
		background: var(--led-warn);
	}
	.err {
		background: var(--led-err);
	}
	.pending {
		background: var(--signal);
		animation: blink 0.9s steps(2, start) infinite;
	}
	@keyframes blink {
		to {
			visibility: hidden;
		}
	}
</style>
