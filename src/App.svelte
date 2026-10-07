<script lang="ts">
	import TitleBar from "./components/TitleBar.svelte";
	import VersionTag from "./components/VersionTag.svelte";
	import Tooltip from "./components/Tooltip.svelte";
	import InstallScreen from "./screens/InstallScreen.svelte";
	import MainScreen from "./screens/MainScreen.svelte";
	import SlotEditor from "./screens/SlotEditor.svelte";
	import TuningScreen from "./screens/TuningScreen.svelte";
	import UninstallScreen from "./screens/UninstallScreen.svelte";
	import Kit from "./dev/Kit.svelte";
	import { setupApi } from "./lib/api";
	import { setLocale, t } from "./lib/i18n.svelte";
	import { accentStyle, DEFAULT_ACCENT } from "./lib/palette";
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
				: ""
	);

	const kit = import.meta.env.DEV && new URLSearchParams(location.search).has("kit");

	const DARK = "(prefers-color-scheme: dark)";
	let systemDark = $state(matchMedia(DARK).matches);
	$effect(() => {
		const mq = matchMedia(DARK);
		const sync = () => (systemDark = mq.matches);
		mq.addEventListener("change", sync);
		return () => mq.removeEventListener("change", sync);
	});
	const themePref = $derived(app.dto?.settings.theme ?? "auto");
	const dark = $derived(themePref === "night" || (themePref === "auto" && systemDark));
	const accent = $derived(accentStyle(app.dto?.settings.accent ?? DEFAULT_ACCENT));

	// Смена темы гасит переходы на два кадра — иначе токены доезжают вразнобой
	let root = $state<HTMLDivElement>();
	let shownDark: boolean | undefined;
	$effect(() => {
		const next = dark;
		if (!root || shownDark === undefined || shownDark === next) {
			shownDark = next;
			return;
		}
		shownDark = next;
		const el = root;
		el.classList.add("fp-no-transitions");
		requestAnimationFrame(() => requestAnimationFrame(() => el.classList.remove("fp-no-transitions")));
	});
</script>

<div class="tc-root fp-app" bind:this={root} data-theme={dark ? "dark" : "light"} style={accent}>
	{#if mode?.mode === "launcher" && app.dto}
		<TitleBar {channel}><VersionTag version={app.dto.version} /></TitleBar>
	{:else}
		<TitleBar {channel} />
	{/if}
	<main class="screen">
		{#if kit}
			<Kit />
		{:else if mode?.mode === "install" && mode.install}
			<InstallScreen info={mode.install} />
		{:else if mode?.mode === "uninstall"}
			<UninstallScreen />
		{:else if !app.dto}
			<div class="boot">…</div>
		{:else if app.screen === "tuning"}
			<TuningScreen />
		{:else if app.screen === "slot" && app.editing}
			<SlotEditor />
		{:else}
			<MainScreen />
		{/if}
	</main>
	<Tooltip />
</div>

<style>
	.fp-app {
		display: grid;
		grid-template-rows: 44px minmax(0, 1fr);
	}
	.screen {
		min-height: 0;
	}
	.boot {
		display: grid;
		place-items: center;
		height: 100%;
		color: var(--tc-muted);
		font: 500 12px var(--tc-font-mono);
	}
</style>
