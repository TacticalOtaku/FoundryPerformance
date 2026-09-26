<script lang="ts">
	import type { TipData } from "../lib/tips";
	import { tip as tooltip } from "../lib/tooltip.svelte";

	let {
		label,
		checked,
		modified = false,
		tip,
		onchange
	}: { label: string; checked: boolean; modified?: boolean; tip?: TipData; onchange: (v: boolean) => void } = $props();
</script>

<button class="toggle" use:tooltip={tip} role="switch" aria-checked={checked} onclick={() => onchange(!checked)}>
	<span>{label}{#if modified}<i class="dot" aria-hidden="true"></i>{/if}</span>
	<span class="state"><span class="lamp" class:on={checked}></span><span class="sw" class:on={checked}><i></i></span></span>
</button>

<style>
	.toggle {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		width: 100%;
		padding: 5px 0;
		text-align: left;
	}
	.state {
		display: flex;
		align-items: center;
		gap: 9px;
	}
	/* сигнальная лампа рядом с переключателем */
	.lamp {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--led-off);
		box-shadow: var(--recess);
		transition: background 0.15s;
	}
	.lamp.on {
		background: var(--signal);
		box-shadow: var(--glow-signal);
	}
	/* прорезь с ползунком */
	.sw {
		position: relative;
		flex: none;
		width: 34px;
		height: 16px;
		border-radius: 2px;
		background: var(--well);
		box-shadow: var(--recess);
	}
	.sw i {
		position: absolute;
		top: 1px;
		left: 1px;
		width: 16px;
		height: 14px;
		border-radius: 2px;
		background:
			repeating-linear-gradient(90deg, rgb(0 0 0 / 0.14) 0 1px, transparent 1px 3px),
			linear-gradient(var(--cap-hi), var(--cap-lo));
		box-shadow: 0 1px 2px rgb(0 0 0 / 0.4);
		transition: translate 0.16s var(--ease-detent);
	}
	.sw.on i {
		translate: 16px 0;
	}
</style>
