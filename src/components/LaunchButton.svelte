<script lang="ts">
	import { t } from "../lib/i18n.svelte";

	let { busy, onlaunch }: { busy: boolean; onlaunch: (safe: boolean) => void } = $props();
	let shift = $state(false);

	$effect(() => {
		const track = (e: KeyboardEvent) => (shift = e.shiftKey);
		const reset = () => (shift = false);
		window.addEventListener("keydown", track);
		window.addEventListener("keyup", track);
		window.addEventListener("blur", reset);
		return () => {
			window.removeEventListener("keydown", track);
			window.removeEventListener("keyup", track);
			window.removeEventListener("blur", reset);
		};
	});
</script>

<div class="group">
	<button class="launch" class:safe={shift} aria-busy={busy} onclick={(e) => onlaunch(e.shiftKey)}>
		<span>{busy ? t("main.launching") : shift ? t("main.launchSafe") : t("main.launch")}</span>
		<span aria-hidden="true">▶</span>
	</button>
	<span class="hint mono">{t("main.safeHint")}</span>
</div>

<style>
	.group {
		display: grid;
		gap: 6px;
	}
	.launch {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 15px 18px;
		background: var(--signal);
		color: var(--signal-ink);
		font-weight: 800;
		font-size: 20px;
		letter-spacing: 0.06em;
		box-shadow: 0 3px 0 var(--signal-deep);
		transition:
			transform 0.06s,
			box-shadow 0.06s,
			background 0.12s;
	}
	.launch:active {
		transform: translateY(3px);
		box-shadow: 0 0 0 var(--signal-deep);
	}
	.launch.safe {
		background: var(--inverse-bg);
		color: var(--inverse-fg);
		box-shadow: 0 3px 0 var(--ink-3);
	}
	.launch[aria-busy="true"] {
		cursor: progress;
	}
	.hint {
		color: var(--ink-2);
	}
</style>
