<script lang="ts">
	import type { TipData } from "../../lib/tips";
	import { tip as tooltip } from "../../lib/tooltip.svelte";

	type Props = { label: string; checked: boolean; onchange: (v: boolean) => void; modified?: boolean; tip?: TipData; danger?: boolean; disabled?: boolean };
	let { label, checked, onchange, modified = false, tip, danger = false, disabled = false }: Props = $props();
</script>

<!-- тумблер: утопленный желоб, поднятая шайба; включён — желоб заливается акцентом -->
<button type="button" class="tg" class:danger role="switch" aria-checked={checked} {disabled} {@attach tooltip(() => tip)} onclick={() => onchange(!checked)}>
	<span class="text">{label}{#if modified}<i class="mod" aria-hidden="true"></i>{/if}</span>
	<span class="track" class:on={checked} aria-hidden="true"><span class="puck"></span></span>
</button>

<style>
	.tg {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		width: 100%;
		min-height: 34px;
		padding: 4px 4px 4px 0;
		text-align: left;
		font-size: 13px;
	}
	.text {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
	}
	.mod {
		width: 6px;
		height: 6px;
		flex: none;
		border-radius: 99px;
		background: var(--tc-accent);
		box-shadow: 0 0 6px var(--tc-glow);
	}
	.track {
		position: relative;
		flex: none;
		width: 36px;
		height: 20px;
		border-radius: 99px;
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
		transition:
			background-color 240ms var(--tc-ease),
			box-shadow 240ms var(--tc-ease);
	}
	.puck {
		position: absolute;
		top: 2px;
		left: 2px;
		width: 16px;
		height: 16px;
		border-radius: 99px;
		background: var(--tc-surface);
		box-shadow: var(--tc-raise);
		transition: transform 240ms var(--tc-ease);
	}
	.on {
		background: var(--tc-accent);
		box-shadow:
			inset 0 1px 2px rgb(0 0 0 / 15%),
			0 0 10px -2px var(--tc-glow);
	}
	.on .puck {
		transform: translateX(16px);
	}
	.danger .on {
		background: var(--tc-danger);
		box-shadow: inset 0 1px 2px rgb(0 0 0 / 15%);
	}
	.tg:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}
</style>
