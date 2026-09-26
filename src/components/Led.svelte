<script lang="ts">
	import type { LedState } from "../lib/types";

	let { state, label }: { state: LedState; label: string } = $props();
</script>

<span class="led {state}" role="img" aria-label={label} title={label}></span>

<style>
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
