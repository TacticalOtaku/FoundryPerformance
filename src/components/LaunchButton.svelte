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
		<span class="play" aria-hidden="true"></span>
	</button>
	<span class="hint silk">{t("main.safeHint")}</span>
</div>

<style>
	.group {
		display: grid;
		gap: 8px;
	}
	/* большая клавиша: выпуклая, с ходом вниз при нажатии */
	.launch {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 14px 16px 14px 20px;
		color: var(--signal-ink);
		font-weight: 800;
		font-size: 21px;
		letter-spacing: 0.08em;
		background: linear-gradient(var(--signal-hi), var(--signal));
		box-shadow:
			inset 0 1px 0 rgb(255 255 255 / 0.35),
			0 4px 0 var(--signal-deep),
			0 6px 14px rgb(0 0 0 / 0.35);
		transition:
			translate 0.07s,
			box-shadow 0.07s,
			background 0.15s;
	}
	.launch:hover {
		background: linear-gradient(#ff8656, var(--signal-hi));
	}
	.launch:active {
		translate: 0 4px;
		box-shadow:
			inset 0 1px 0 rgb(255 255 255 / 0.25),
			0 0 0 var(--signal-deep),
			0 2px 4px rgb(0 0 0 / 0.3);
	}
	.play {
		display: grid;
		place-items: center;
		width: 30px;
		height: 30px;
		border-radius: 50%;
		background: rgb(0 0 0 / 0.16);
		box-shadow: inset 0 1px 2px rgb(0 0 0 / 0.25);
	}
	.play::before {
		content: "";
		margin-left: 3px;
		border-left: 10px solid currentColor;
		border-top: 6px solid transparent;
		border-bottom: 6px solid transparent;
	}
	.launch.safe {
		color: var(--inverse-fg);
		background: linear-gradient(color-mix(in srgb, var(--inverse-bg) 88%, white), var(--inverse-bg));
		box-shadow:
			inset 0 1px 0 rgb(255 255 255 / 0.15),
			0 4px 0 var(--ink-3),
			0 6px 14px rgb(0 0 0 / 0.35);
	}
	.launch[aria-busy="true"] {
		cursor: progress;
	}
	.hint {
		font-size: 10px;
		letter-spacing: 0.06em;
		text-transform: none;
	}
</style>
