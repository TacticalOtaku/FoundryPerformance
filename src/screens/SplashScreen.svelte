<script lang="ts">
	import Ring from "../components/splash/Ring.svelte";
	import { splashApi, updateApi } from "../lib/api";
	import { t } from "../lib/i18n.svelte";
	import { canSkip, CHECK_TIMEOUT_S, type Phase, ringMode, SplashFlow } from "../lib/splash";

	let { current }: { current: string } = $props();

	let phase = $state<Phase>({ kind: "checking" });
	const flow = new SplashFlow(
		{
			flags: splashApi.flags,
			check: () => updateApi.check(CHECK_TIMEOUT_S),
			apply: updateApi.apply,
			onProgress: (cb) => updateApi.onProgress(cb),
			cancel: updateApi.cancel,
			done: splashApi.done,
			wait: (ms) => new Promise((r) => setTimeout(r, ms))
		},
		(p) => (phase = p)
	);
	void flow.run();

	const title = $derived.by(() => {
		switch (phase.kind) {
			case "checking":
				return t("splash.checking");
			case "downloading":
				return t("splash.downloading", { version: phase.version });
			case "verifying":
				return t("splash.verifying");
			case "restarting":
				return t("splash.restarting");
			case "offline":
				return t("splash.offline");
			case "error":
				return t(phase.key);
			case "updated":
				return t("splash.updated", { version: phase.version });
		}
	});

	const sub = $derived.by(() => {
		switch (phase.kind) {
			case "checking":
				return `v${current}`;
			case "downloading":
				return phase.pct === null ? "" : `${phase.pct} %`;
			case "verifying":
				return t("splash.verifyingHint");
			case "restarting":
				return `v${phase.version}`;
			case "offline":
				return t("splash.openingLauncher");
			case "error":
				return t("splash.keptVersion", { version: current });
			case "updated":
				return "";
		}
	});
</script>

<div class="splash" class:compact={phase.kind === "error"} data-tauri-drag-region>
	<div class="ring-slot" data-tauri-drag-region>
		<Ring mode={ringMode(phase)} pct={phase.kind === "downloading" ? (phase.pct ?? 0) : 0} label={t("splash.progressLabel")} />
	</div>
	<p class="title" role="status" aria-live="polite" data-tauri-drag-region>{title}</p>
	{#if sub}
		<p class="sub" class:prose={phase.kind === "error"} data-tauri-drag-region>{sub}</p>
	{/if}
	{#if canSkip(phase)}
		<button type="button" class="ghost" onclick={() => void flow.skip()}>{t("splash.skip")}</button>
	{:else if phase.kind === "error"}
		<button type="button" class="ghost strong" onclick={() => void flow.close()}>{t("splash.openLauncher")}</button>
	{/if}
</div>

<style>
	.splash {
		height: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 54px 0 16px;
		user-select: none;
	}
	/* ошибка бывает в три строки — кольцо поднимается, кнопка остаётся в окне */
	.compact {
		padding-top: 28px;
	}
	.title {
		margin-top: 26px;
		padding: 0 20px;
		font: 600 15px/1.3 var(--tc-font-ui);
		text-align: center;
	}
	.sub {
		margin-top: 6px;
		padding: 0 24px;
		color: var(--tc-muted);
		font: 400 12.5px/1.4 var(--tc-font-mono);
		text-align: center;
	}
	.prose {
		font-family: var(--tc-font-ui);
	}
	.ghost {
		flex: none;
		margin-top: auto;
		height: 30px;
		padding: 0 14px;
		border: 0;
		border-radius: var(--tc-r-ctl);
		background: transparent;
		color: var(--tc-muted);
		font: 500 12.5px var(--tc-font-ui);
		white-space: nowrap;
		cursor: pointer;
	}
	.ghost:hover {
		background: var(--tc-accent-soft);
		color: var(--tc-ink);
	}
	.strong {
		background: var(--tc-surface);
		box-shadow: var(--tc-raise);
		color: var(--tc-ink);
	}
</style>
