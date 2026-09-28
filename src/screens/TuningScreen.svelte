<script lang="ts">
	import Fader from "../components/Fader.svelte";
	import CacheControl from "../components/CacheControl.svelte";
	import Segmented from "../components/Segmented.svelte";
	import Toggle from "../components/Toggle.svelte";
	import { t } from "../lib/i18n.svelte";
	import { countOverrides, overridesFor, profileFor, resolved } from "../lib/levers";
	import { app } from "../lib/store.svelte";
	import { tip, tipFor } from "../lib/tooltip.svelte";
	import type { AngleBackend, Levers, LocalePref, PrimeLevel, ProfileId, ThemePref, VideoMode } from "../lib/types";

	const dto = $derived(app.dto!);
	const scope = $derived(app.scope);
	const l = $derived(resolved(dto, scope));
	const over = $derived(overridesFor(dto, scope));
	const n = $derived(countOverrides(over));
	const mod = (...keys: (keyof Levers)[]) => keys.some((k) => over[k] !== undefined && over[k] !== null);
	const serverIndex = $derived(scope.kind === "server" ? dto.servers.findIndex((s) => s.id === scope.id) : -1);
	const server = $derived(serverIndex >= 0 ? dto.servers[serverIndex] : null);
	const title = $derived(server ? `A${serverIndex + 1} ${server.name}` : t("tuning.global"));
	// Переключатель уровня: из слота — к общим настройкам (движок, язык, тема) и обратно
	const selIndex = $derived(dto.servers.findIndex((s) => s.id === app.selectedId));
	const scopeOptions = $derived([
		{ value: "global", label: t("tuning.global") },
		...(selIndex >= 0 ? [{ value: dto.servers[selIndex].id, label: `A${selIndex + 1}` }] : [])
	]);
	const pct = (v: number) => `${Math.round(v * 100)}%`;

	const profiles = $derived(
		(["quality", "balance", "potato"] as ProfileId[]).map((p) => ({ value: p as ProfileId | "inherit", label: t(`profile.${p}.long`).toUpperCase() }))
	);
	const profileOptions = $derived(server ? [{ value: "inherit" as const, label: t("profile.inherit") }, ...profiles] : profiles);
	const profileValue = $derived<ProfileId | "inherit">(server ? (server.profile ?? "inherit") : dto.settings.profile);

	const videoOptions = $derived((["play", "pauseUnfocused", "static"] as VideoMode[]).map((v) => ({ value: v, label: t(`video.${v}`) })));
	const primeOptions = $derived((["soft", "medium", "aggressive"] as PrimeLevel[]).map((v) => ({ value: v, label: t(`prime.${v}`) })));
	const perfOptions = [3, 2, 1, 0].map((v) => ({ value: v, label: ["LOW", "MED", "HIGH", "MAX"][v] }));
	const angleOptions = (["d3d11", "d3d11on12", "gl", "vulkan"] as AngleBackend[]).map((v) => ({ value: v, label: v.toUpperCase() }));
	const cacheOptions = $derived([1024, 2048, 4096].map((v) => ({ value: v, label: `${v / 1024} ${t("unit.gb")}` })));
	const localeOptions = $derived((["auto", "ru", "en"] as LocalePref[]).map((v) => ({ value: v, label: v === "auto" ? t("locale.auto") : v.toUpperCase() })));
	const themeOptions = $derived((["auto", "day", "night"] as ThemePref[]).map((v) => ({ value: v, label: t(`theme.${v}`) })));
</script>

<div class="tuning">
	<header class="head">
		<button class="back silk" onclick={() => app.closeScreen()}>{t("tuning.back")}</button>
		<b>{t("tuning.title")} · {title}</b>
		<span class="silk">
			{t("tuning.profile")}
			{t(`profile.${profileFor(dto, scope)}`)}
			{#if n > 0}<span class="badge">{t("tuning.changed", { n })}</span>{/if}
		</span>
	</header>

	<div class="body">
		<div class="profile">
			{#if scopeOptions.length > 1}
				<Segmented
					label={t("tuning.scope")}
					tip={tipFor("scope", t("tuning.scope"))}
					options={scopeOptions}
					value={scope.kind === "global" ? "global" : scope.id}
					onchange={(v) => app.openTuning(v === "global" ? { kind: "global" } : { kind: "server", id: v })}
				/>
			{/if}
			<Segmented
				label={t("tuning.profile")}
				options={profileOptions}
				tip={tipFor("profile", t("tuning.profile"))}
				value={profileValue}
				onchange={(v) => app.setScopeProfile(v === "inherit" ? null : v)}
			/>
		</div>

		<div class="faders">
			<Fader
				label={t("tuning.resolution")}
				tip={tipFor("resolution", t("tuning.resolution"))}
				value={l.resMin}
				min={0.4}
				max={1}
				step={0.05}
				format={(v) => (l.adaptive ? `${pct(v)}–${pct(l.resMax)}` : pct(v))}
				modified={mod("resMin", "resMax")}
				onchange={(v) => app.setLevers(l.adaptive ? { resMin: Math.min(v, l.resMax) } : { resMin: v, resMax: v })}
			/>
			<Fader
				label={t("tuning.maxFps")}
				tip={tipFor("maxFps", t("tuning.maxFps"))}
				value={l.maxFps}
				min={20}
				max={240}
				step={1}
				marks={[240, 144, 60]}
				presets={[60, 144, 240]}
				editable
				inputLabel={t("tuning.fpsInput")}
				format={(v) => String(v)}
				modified={mod("maxFps")}
				onchange={(v) => app.setLevers({ maxFps: v })}
			/>
			<Fader
				label={t("tuning.unfocused")}
				tip={tipFor("unfocused", t("tuning.unfocused"))}
				value={l.unfocusedFps}
				min={5}
				max={60}
				step={5}
				format={(v) => String(v)}
				modified={mod("unfocusedFps")}
				onchange={(v) => app.setLevers({ unfocusedFps: v })}
			/>
			<Segmented
				vertical
				label={t("tuning.perfMode")}
				tip={tipFor("perfMode", t("tuning.perfMode"))}
				options={perfOptions}
				value={l.perfMode}
				modified={mod("perfMode")}
				onchange={(v) => app.setLevers({ perfMode: v as Levers["perfMode"] })}
			/>
		</div>

		<div class="toggles">
			<Toggle label={t("tuning.adaptive")} tip={tipFor("adaptive", t("tuning.adaptive"))} checked={l.adaptive} modified={mod("adaptive")} onchange={(v) => app.setLevers({ adaptive: v })} />
			<Toggle label={t("tuning.lightAnimation")} tip={tipFor("lightAnimation", t("tuning.lightAnimation"))} checked={l.lightAnimation} modified={mod("lightAnimation")} onchange={(v) => app.setLevers({ lightAnimation: v })} />
			<Toggle label={t("tuning.visionAnimation")} tip={tipFor("visionAnimation", t("tuning.visionAnimation"))} checked={l.visionAnimation} modified={mod("visionAnimation")} onchange={(v) => app.setLevers({ visionAnimation: v })} />
			<Toggle label={t("tuning.mipmap")} tip={tipFor("mipmap", t("tuning.mipmap"))} checked={l.mipmap} modified={mod("mipmap")} onchange={(v) => app.setLevers({ mipmap: v })} />
			<Toggle label={t("tuning.pixelRatio")} tip={tipFor("pixelRatio", t("tuning.pixelRatio"))} checked={l.pixelRatioScaling} modified={mod("pixelRatioScaling")} onchange={(v) => app.setLevers({ pixelRatioScaling: v })} />
			<Toggle label={t("tuning.uiBlur")} tip={tipFor("uiBlur", t("tuning.uiBlur"))} checked={l.uiBlur} modified={mod("uiBlur")} onchange={(v) => app.setLevers({ uiBlur: v })} />
			<Toggle label="Sequencer" tip={tipFor("sequencer", "Sequencer")} checked={l.sequencer} modified={mod("sequencer")} onchange={(v) => app.setLevers({ sequencer: v })} />
			<Toggle label="FXMaster" tip={tipFor("fxmaster", "FXMaster")} checked={l.fxmaster} modified={mod("fxmaster")} onchange={(v) => app.setLevers({ fxmaster: v })} />
		</div>

		<div class="rows">
			<Segmented label={t("tuning.video")} tip={tipFor("video", t("tuning.video"))} options={videoOptions} value={l.video} modified={mod("video")} onchange={(v) => app.setLevers({ video: v })} />
			<Segmented label={t("tuning.prime")} tip={tipFor("prime", "Prime Performance")} options={primeOptions} value={l.prime} modified={mod("prime")} onchange={(v) => app.setLevers({ prime: v })} />
		</div>

		{#if !server}
			<div class="rows engine">
				<Segmented
					label={t("tuning.angle")}
					tip={tipFor("angle", t("tuning.angle"))}
					options={angleOptions}
					value={dto.settings.engine.angle}
					onchange={(v) => app.saveSettings({ engine: { ...dto.settings.engine, angle: v } })}
				/>
				<Segmented
					label={t("tuning.cache")}
					tip={tipFor("cache", t("tuning.cache"))}
					options={cacheOptions}
					value={dto.settings.engine.diskCacheMb}
					onchange={(v) => app.saveSettings({ engine: { ...dto.settings.engine, diskCacheMb: v } })}
				/>
				<CacheControl />
				<label class="extra" {@attach tip(() => tipFor("extraArgs", t("tuning.extraArgs")))}>
					<span class="silk">{t("tuning.extraArgs")}</span>
					<input
						class="mono"
						value={dto.settings.engine.extraArgs}
						placeholder="--flag=value"
						spellcheck="false"
						onchange={(e) => app.saveSettings({ engine: { ...dto.settings.engine, extraArgs: e.currentTarget.value } })}
					/>
					<span class="mono hint">{t("tuning.extraArgsHint")}</span>
				</label>
				<Segmented label={t("tuning.language")} options={localeOptions} value={dto.settings.locale} onchange={(v) => app.saveSettings({ locale: v })} />
				<Segmented label={t("tuning.theme")} options={themeOptions} value={dto.settings.theme} onchange={(v) => app.saveSettings({ theme: v })} />
			</div>
		{/if}
	</div>

	<footer class="foot mono">
		<span>{t("tuning.engineLine", { angle: dto.settings.engine.angle.toUpperCase(), cache: dto.settings.engine.diskCacheMb / 1024 })}</span>
		<span>
			{t("tuning.modifiedLegend")} ·
			<button class="reset" onclick={() => app.resetOverrides()}>{t("tuning.reset")}</button>
		</span>
	</footer>
</div>

<style>
	.tuning {
		display: grid;
		grid-template-rows: auto minmax(0, 1fr) auto;
		gap: 2px;
		height: 100%;
		min-height: 0;
	}
	.head,
	.foot {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 10px 18px;
		background: var(--grain), var(--face);
		box-shadow: var(--bevel);
	}
	.head b {
		font-weight: 800;
		letter-spacing: 0.08em;
		margin-right: auto;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.back {
		padding: 6px 10px;
		background: var(--face-2);
		box-shadow: var(--lift);
	}
	.back:active {
		box-shadow: var(--recess);
	}
	.back:hover,
	.reset:hover {
		color: var(--signal-text);
	}
	.badge {
		margin-left: 6px;
		padding: 1px 6px;
		color: var(--signal-ink);
		background: var(--signal);
		box-shadow: var(--glow-signal);
		text-shadow: none;
	}
	.body {
		display: grid;
		gap: 2px;
		overflow-y: auto;
		min-height: 0;
	}
	.profile,
	.toggles,
	.rows {
		background: var(--grain), var(--face);
		box-shadow: var(--bevel);
		padding: 12px 18px;
	}
	.profile {
		display: flex;
		gap: 28px;
	}
	.faders {
		display: grid;
		grid-template-columns: repeat(4, minmax(0, 1fr));
		gap: 2px;
	}
	.toggles {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 6px 28px;
	}
	.rows {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 14px 28px;
	}
	.extra {
		grid-column: 1 / -1;
		display: grid;
		gap: 6px;
	}
	.extra input {
		height: 34px;
		padding: 0 10px;
		background: var(--well);
		border: 1px solid transparent;
		box-shadow: var(--recess);
	}
	.hint {
		color: var(--ink-2);
	}
	.foot {
		justify-content: space-between;
		color: var(--ink-2);
	}
</style>
