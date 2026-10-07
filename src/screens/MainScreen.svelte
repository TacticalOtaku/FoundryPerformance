<script lang="ts">
	import Hero from "../components/main/Hero.svelte";
	import LastRun from "../components/main/LastRun.svelte";
	import ServerList from "../components/main/ServerList.svelte";
	import Icon from "../components/tactile/Icon.svelte";
	import IconButton from "../components/tactile/IconButton.svelte";
	import { t } from "../lib/i18n.svelte";
	import { openWindow } from "../lib/motion";
	import { app } from "../lib/store.svelte";
</script>

<div class="main" use:openWindow>
	<div class="hero" data-part><Hero /></div>
	<div class="side">
		<div class="list" data-part><ServerList /></div>
		{#if app.message}
			<div class="notice" role="status">
				<span>{app.message}</span>
				<IconButton label={t("window.close")} onclick={() => app.dismiss()}><Icon name="close" size={14} /></IconButton>
			</div>
		{/if}
		<div data-part><LastRun /></div>
	</div>
</div>

<style>
	.main {
		height: 100%;
		display: grid;
		grid-template-columns: minmax(0, 1.5fr) minmax(0, 1fr);
		gap: 14px;
		padding: 2px 14px 14px;
	}
	.hero,
	.list {
		min-height: 0;
	}
	.side {
		min-height: 0;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.list {
		flex: 1;
	}
	.notice {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		padding: 8px 6px 8px 14px;
		border-radius: var(--tc-r-block);
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
		font-size: 12.5px;
		line-height: 1.4;
	}
	.notice span {
		flex: 1;
		padding-top: 5px;
	}
</style>
