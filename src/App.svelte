<script lang="ts">
	import TitleBar from "./components/TitleBar.svelte";
	import Tooltip from "./components/Tooltip.svelte";
	import InstallScreen from "./screens/InstallScreen.svelte";
	import MainScreen from "./screens/MainScreen.svelte";
	import SlotEditor from "./screens/SlotEditor.svelte";
	import TuningScreen from "./screens/TuningScreen.svelte";
	import UninstallScreen from "./screens/UninstallScreen.svelte";
	import { setupApi } from "./lib/api";
	import { setLocale, t } from "./lib/i18n.svelte";
	import { app } from "./lib/store.svelte";
	import type { ModeDto } from "./lib/types";

	// Режим решает Rust: тот же exe — установщик, удаление или лаунчер
	let mode = $state<ModeDto | null>(null);
	void setupApi.getMode().then((m) => {
		mode = m;
		if (m.mode === "launcher") void app.load();
		else setLocale(navigator.language.toLowerCase().startsWith("ru") ? "ru" : "en");
	});

	const channel = $derived(
		mode?.mode === "install" && mode.install
			? `${t("install.channel")} · v${mode.install.currentVersion}`
			: mode?.mode === "uninstall"
				? t("uninstall.channel")
				: app.dto
					? `v${app.dto.version}`
					: ""
	);

	$effect(() => {
		const pref = app.dto?.settings.theme ?? "auto";
		const mq = matchMedia("(prefers-color-scheme: dark)");
		const apply = () => {
			document.documentElement.dataset.theme = pref === "auto" ? (mq.matches ? "night" : "day") : pref;
		};
		apply();
		mq.addEventListener("change", apply);
		return () => mq.removeEventListener("change", apply);
	});
</script>

<div class="frame">
	<TitleBar {channel} />
	<main class="screen">
		{#if mode?.mode === "install" && mode.install}
			<InstallScreen info={mode.install} />
		{:else if mode?.mode === "uninstall"}
			<UninstallScreen />
		{:else if !app.dto}
			<div class="boot mono">…</div>
		{:else if app.screen === "tuning"}
			<TuningScreen />
		{:else if app.screen === "slot" && app.editing}
			<SlotEditor />
		{:else}
			<MainScreen />
		{/if}
	</main>
</div>
<Tooltip />

<style>
	.frame {
		display: grid;
		grid-template-rows: 40px minmax(0, 1fr);
		gap: 2px;
		height: 100vh;
		padding: 0 2px 2px;
		background: var(--panel);
	}
	.screen {
		min-height: 0;
	}
	.boot {
		display: grid;
		place-items: center;
		height: 100%;
		background: var(--grain), var(--face);
		box-shadow: var(--bevel);
		color: var(--ink-2);
	}
</style>
