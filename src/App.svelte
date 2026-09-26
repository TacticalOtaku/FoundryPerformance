<script lang="ts">
	import TitleBar from "./components/TitleBar.svelte";
	import MainScreen from "./screens/MainScreen.svelte";
	import SlotEditor from "./screens/SlotEditor.svelte";
	import TuningScreen from "./screens/TuningScreen.svelte";
	import { app } from "./lib/store.svelte";

	void app.load();

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
	<TitleBar channel={app.dto ? `v${app.dto.version}` : ""} />
	<main class="screen">
		{#if !app.dto}
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
		background: var(--face);
		color: var(--ink-2);
	}
</style>
