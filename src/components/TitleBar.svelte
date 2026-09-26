<script lang="ts">
	import { windowControls } from "../lib/api";
	import { t } from "../lib/i18n.svelte";

	let { channel }: { channel: string } = $props();
</script>

<header class="bar" data-tauri-drag-region>
	<b class="brand" data-tauri-drag-region>FOUNDRY/PERFORMANCE</b>
	<span class="mono ch" data-tauri-drag-region>{channel}</span>
	<div class="ctl">
		<button class="mono" aria-label={t("window.minimize")} onclick={() => windowControls.minimize()}>—</button>
		<button class="mono close" aria-label={t("window.close")} onclick={() => windowControls.close()}>✕</button>
	</div>
</header>

<style>
	.bar {
		display: flex;
		align-items: center;
		gap: 14px;
		height: 40px;
		padding-left: 18px;
		background: var(--face);
	}
	.brand {
		font-weight: 800;
		font-size: 13px;
		letter-spacing: 0.14em;
	}
	.ch {
		margin-left: auto;
		color: var(--ink-2);
	}
	.ctl {
		display: flex;
		height: 100%;
	}
	.ctl button {
		width: 46px;
		height: 100%;
		display: grid;
		place-items: center;
	}
	.ctl button:hover {
		background: var(--face-2);
	}
	.ctl .close:hover {
		background: var(--signal);
		color: var(--signal-ink);
	}
</style>
