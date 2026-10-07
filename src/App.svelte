<script lang="ts">
	import TitleBar from "./components/TitleBar.svelte";
	import GpuBadge from "./components/GpuBadge.svelte";
	import UpdatePill from "./components/UpdatePill.svelte";
	import VersionChip from "./components/VersionChip.svelte";
	import Chip from "./components/tactile/Chip.svelte";
	import Icon from "./components/tactile/Icon.svelte";
	import IconButton from "./components/tactile/IconButton.svelte";
	import Tooltip from "./components/Tooltip.svelte";
	import InstallScreen from "./screens/InstallScreen.svelte";
	import MainScreen from "./screens/MainScreen.svelte";
	import SplashScreen from "./screens/SplashScreen.svelte";
	import SlotEditor from "./screens/SlotEditor.svelte";
	import TuningScreen from "./screens/TuningScreen.svelte";
	import UninstallScreen from "./screens/UninstallScreen.svelte";
	import Kit from "./dev/Kit.svelte";
	import { setupApi, windowLabel } from "./lib/api";
	import { setLocale, t } from "./lib/i18n.svelte";
	import { accentStyle, DEFAULT_ACCENT } from "./lib/palette";
	import { app } from "./lib/store.svelte";
	import type { ModeDto } from "./lib/types";

	// Режим решает Rust: тот же exe — установщик, удаление или лаунчер; сплэш — отдельное окно
	let mode = $state<ModeDto | null>(null);
	let splash = $state(false);
	void windowLabel().then(async (label) => {
		if (label === "splash") {
			splash = true;
			await app.loadForSplash();
			return;
		}
		const m = await setupApi.getMode();
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

<div class="tc-root" class:fp-app={!splash} bind:this={root} data-theme={dark ? "dark" : "light"} style={accent}>
	{#if splash}
		{#if app.dto}<SplashScreen current={app.dto.version} />{/if}
	{:else}
		{#if mode?.mode === "launcher" && app.dto}
			<TitleBar>
				{#snippet start()}
					<VersionChip version={app.dto!.version} />
					<UpdatePill />
				{/snippet}
				{#snippet end()}
					<GpuBadge />
					<IconButton
						label={t("main.tune")}
						size={30}
						on={app.screen === "tuning"}
						onclick={() => app.openTuning(app.selected ? { kind: "server", id: app.selected.id } : { kind: "global" })}
					>
						<Icon name="sliders" />
					</IconButton>
				{/snippet}
			</TitleBar>
		{:else}
			<TitleBar>
				{#snippet start()}{#if channel}<Chip>{channel}</Chip>{/if}{/snippet}
			</TitleBar>
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
	{/if}
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
