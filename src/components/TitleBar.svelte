<script lang="ts">
	import { windowControls } from "../lib/api";
	import { t } from "../lib/i18n.svelte";

	let { channel }: { channel: string } = $props();
</script>

<header class="bar plate" data-tauri-drag-region>
	<span class="power" aria-hidden="true"></span>
	<b class="brand" data-tauri-drag-region>FOUNDRY/PERFORMANCE</b>
	{#if channel}<span class="tag mono" data-tauri-drag-region>{channel}</span>{/if}
	<div class="ctl">
		<button aria-label={t("window.minimize")} onclick={() => windowControls.minimize()}><span class="glyph min"></span></button>
		<button class="close" aria-label={t("window.close")} onclick={() => windowControls.close()}><span class="glyph x"></span></button>
	</div>
</header>

<style>
	.bar {
		display: flex;
		align-items: center;
		gap: 12px;
		height: 40px;
		padding-left: 20px;
	}
	/* питание: горит, пока приложение открыто */
	.power {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: var(--led-ok);
		box-shadow: 0 0 6px var(--led-ok);
	}
	.brand {
		font-weight: 800;
		font-size: 13px;
		letter-spacing: 0.16em;
		text-shadow: var(--engrave);
	}
	.tag {
		margin-left: auto;
		padding: 2px 7px;
		color: var(--ink-2);
		background: var(--well);
		box-shadow: var(--recess);
	}
	.ctl {
		display: flex;
		height: 100%;
		margin-left: 12px;
	}
	.ctl button {
		display: grid;
		place-items: center;
		width: 46px;
		height: 100%;
		color: var(--ink-2);
	}
	.ctl button:hover {
		color: var(--ink);
		background: color-mix(in srgb, var(--face) 80%, var(--ink) 8%);
	}
	.ctl .close:hover {
		color: var(--signal-ink);
		background: var(--signal);
	}
	/* нарисованные значки вместо символов шрифта — одинаково чёткие в любой теме */
	.glyph {
		position: relative;
		width: 11px;
		height: 11px;
	}
	.min::before,
	.x::before,
	.x::after {
		content: "";
		position: absolute;
		left: 0;
		top: 5px;
		width: 11px;
		height: 1.5px;
		background: currentColor;
	}
	.x::before {
		rotate: 45deg;
	}
	.x::after {
		rotate: -45deg;
	}
</style>
