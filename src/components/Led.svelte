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
		box-shadow: var(--recess);
	}
	/* горящий светодиод: линза с бликом и ореол того же цвета */
	.ok,
	.warn,
	.err,
	.pending {
		--c: var(--led-ok);
		background: radial-gradient(circle at 35% 30%, rgb(255 255 255 / 0.55), transparent 45%), var(--c);
		box-shadow:
			0 0 0 3px color-mix(in srgb, var(--c) calc(var(--led-halo) * 100%), transparent),
			0 0 8px var(--c);
	}
	.warn {
		--c: var(--led-warn);
	}
	.err {
		--c: var(--led-err);
	}
	.pending {
		--c: var(--signal);
		animation: blink 0.9s steps(2, start) infinite;
	}
	@keyframes blink {
		to {
			visibility: hidden;
		}
	}
</style>
