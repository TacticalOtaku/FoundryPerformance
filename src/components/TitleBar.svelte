<script lang="ts">
	import type { Snippet } from "svelte";
	import { windowControls } from "../lib/api";
	import { t } from "../lib/i18n.svelte";
	import Icon from "./tactile/Icon.svelte";
	import IconButton from "./tactile/IconButton.svelte";

	/** `start` — после названия (версия, обновление); `end` — перед кнопками окна (видеокарта, «Настройка»). */
	let { start, end }: { start?: Snippet; end?: Snippet } = $props();
</script>

<header class="bar" data-tauri-drag-region>
	<span class="app" aria-hidden="true" data-tauri-drag-region><Icon name="app" /></span>
	<b class="ttl" data-tauri-drag-region>Foundry Performance</b>
	{#if start}{@render start()}{/if}
	<span class="sp" data-tauri-drag-region></span>
	{#if end}{@render end()}{/if}
	<span class="wb">
		<IconButton label={t("window.minimize")} size={30} onclick={() => windowControls.minimize()}><Icon name="minimize" /></IconButton>
		<IconButton label={t("window.maximize")} size={30} onclick={() => windowControls.toggleMaximize()}><Icon name="maximize" /></IconButton>
		<IconButton label={t("window.close")} size={30} danger onclick={() => windowControls.close()}><Icon name="close" /></IconButton>
	</span>
</header>

<style>
	.bar {
		height: 44px;
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 0 8px 0 10px;
		user-select: none;
	}
	.app {
		width: 28px;
		height: 28px;
		display: grid;
		place-items: center;
		flex: none;
		border-radius: 99px;
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
		color: var(--tc-accent-text);
	}
	/* декор не перехватывает клик: перетаскивание срабатывает только на самой полосе */
	.app :global(svg),
	.bar :global(span.chip),
	.bar :global(span.pill) {
		pointer-events: none;
	}
	.ttl {
		font-weight: 600;
		font-size: 15px;
		line-height: 1.15;
		white-space: nowrap;
	}
	.sp {
		flex: 1;
		align-self: stretch;
	}
	.wb {
		display: flex;
		gap: 2px;
		margin-left: 4px;
	}
</style>
