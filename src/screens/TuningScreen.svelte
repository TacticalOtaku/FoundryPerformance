<script lang="ts">
	import CacheControl from "../components/CacheControl.svelte";
	import Button from "../components/tactile/Button.svelte";
	import Core from "../components/tactile/Core.svelte";
	import Field from "../components/tactile/Field.svelte";
	import Icon from "../components/tactile/Icon.svelte";
	import IconButton from "../components/tactile/IconButton.svelte";
	import Pill from "../components/tactile/Pill.svelte";
	import Section from "../components/tactile/Section.svelte";
	import Segments from "../components/tactile/Segments.svelte";
	import Select from "../components/tactile/Select.svelte";
	import Slider from "../components/tactile/Slider.svelte";
	import Swatches from "../components/tactile/Swatches.svelte";
	import Toggle from "../components/tactile/Toggle.svelte";
	import { t } from "../lib/i18n.svelte";
	import { countOverrides, overridesFor, resolved } from "../lib/levers";
	import { openWindow } from "../lib/motion";
	import { app } from "../lib/store.svelte";
	import { tipFor } from "../lib/tooltip.svelte";
	import type { TipKey } from "../lib/tips";
	import type { AngleBackend, Levers, LocalePref, PrimeLevel, ProfileId, ThemePref, VideoMode } from "../lib/types";

	const dto = $derived(app.dto!);
	const scope = $derived(app.scope);
	const l = $derived(resolved(dto, scope));
	const over = $derived(overridesFor(dto, scope));
	const n = $derived(countOverrides(over));
	const mod = (...keys: (keyof Levers)[]) => keys.some((k) => over[k] !== undefined && over[k] !== null);
	const server = $derived(scope.kind === "server" ? (dto.servers.find((s) => s.id === scope.id) ?? null) : null);
	const title = $derived(server ? server.name : t("tuning.global"));
	// Переключатель уровня: общие настройки (движок, вид) и правки выбранного сервера
	const selected = $derived(dto.servers.find((s) => s.id === app.selectedId) ?? null);
	const scopeOptions = $derived([{ value: "global", label: t("tuning.global") }, ...(selected ? [{ value: selected.id, label: selected.name }] : [])]);
	const toggles = $derived<{ key: keyof Levers; label: string; tipKey: TipKey }[]>([
		{ key: "adaptive", label: t("tuning.adaptive"), tipKey: "adaptive" },
		{ key: "lightAnimation", label: t("tuning.lightAnimation"), tipKey: "lightAnimation" },
		{ key: "visionAnimation", label: t("tuning.visionAnimation"), tipKey: "visionAnimation" },
		{ key: "mipmap", label: t("tuning.mipmap"), tipKey: "mipmap" },
		{ key: "pixelRatioScaling", label: t("tuning.pixelRatio"), tipKey: "pixelRatio" },
		{ key: "uiBlur", label: t("tuning.uiBlur"), tipKey: "uiBlur" },
		{ key: "sequencer", label: "Sequencer", tipKey: "sequencer" },
		{ key: "fxmaster", label: "FXMaster", tipKey: "fxmaster" }
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

<div class="tuning" use:openWindow>
	<header class="head" data-part>
		<IconButton label={t("tuning.back")} onclick={() => app.closeScreen()}><Icon name="back" /></IconButton>
		<h1>{t("tuning.title")}</h1>
		<span class="for">{title}</span>
		{#if n > 0}<Pill>{t("tuning.changed", { n })}</Pill>{/if}
		<span class="sp"></span>
		{#if scopeOptions.length > 1}
			<div class="scope">
				<Segments
					showLabel={false}
					label={t("tuning.scope")}
					tip={tipFor("scope", t("tuning.scope"))}
					options={scopeOptions}
					value={scope.kind === "global" ? "global" : scope.id}
					onchange={(v) => app.openTuning(v === "global" ? { kind: "global" } : { kind: "server", id: v })}
				/>
			</div>
		{/if}
	</header>

	<div class="body" data-part>
		<Core fill scroll pad="18px 20px 22px">
			<div class="sections">
				<Section title={t("tuning.sec.levers")}>
					<Segments
						label={t("tuning.profile")}
						tip={tipFor("profile", t("tuning.profile"))}
						options={profileOptions}
						value={profileValue}
						onchange={(v) => app.setScopeProfile(v === "inherit" ? null : (v as ProfileId))}
					/>
					<div class="grid">
						<Slider
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
						<Slider
							label={t("tuning.maxFps")}
							tip={tipFor("maxFps", t("tuning.maxFps"))}
							value={l.maxFps}
							min={20}
							max={240}
							step={1}
							ticks={[60, 144, 240]}
							presets={[60, 144, 240]}
							editable
							inputLabel={t("tuning.fpsInput")}
							format={(v) => String(v)}
							modified={mod("maxFps")}
							onchange={(v) => app.setLevers({ maxFps: v })}
						/>
						<Slider
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
						<Segments
							label={t("tuning.perfMode")}
							tip={tipFor("perfMode", t("tuning.perfMode"))}
							options={perfOptions}
							value={l.perfMode}
							modified={mod("perfMode")}
							onchange={(v) => app.setLevers({ perfMode: v as Levers["perfMode"] })}
						/>
					</div>
					<div class="grid toggles">
						{#each toggles as tg (tg.key)}
							<Toggle
								label={tg.label}
								tip={tipFor(tg.tipKey, tg.label)}
								checked={Boolean(l[tg.key])}
								modified={mod(tg.key)}
								onchange={(v) => app.setLevers({ [tg.key]: v } as Partial<Levers>)}
							/>
						{/each}
					</div>
					<div class="grid">
						<Segments label={t("tuning.video")} tip={tipFor("video", t("tuning.video"))} options={videoOptions} value={l.video} modified={mod("video")} onchange={(v) => app.setLevers({ video: v as VideoMode })} />
						<Segments label={t("tuning.prime")} tip={tipFor("prime", "Prime Performance")} options={primeOptions} value={l.prime} modified={mod("prime")} onchange={(v) => app.setLevers({ prime: v as PrimeLevel })} />
					</div>
				</Section>

				{#if !server}
					<Section title={t("tuning.sec.engine")}>
						<div class="grid">
							<Select
								label={t("tuning.angle")}
								tip={tipFor("angle", t("tuning.angle"))}
								options={angleOptions}
								value={dto.settings.engine.angle}
								onchange={(v) => app.saveSettings({ engine: { ...dto.settings.engine, angle: v as AngleBackend } })}
							/>
							<Segments
								label={t("tuning.cache")}
								tip={tipFor("cache", t("tuning.cache"))}
								options={cacheOptions}
								value={dto.settings.engine.diskCacheMb}
								onchange={(v) => app.saveSettings({ engine: { ...dto.settings.engine, diskCacheMb: v } })}
							/>
						</div>
						<CacheControl />
						<Field
							label={t("tuning.extraArgs")}
							mono
							value={dto.settings.engine.extraArgs}
							placeholder="--flag=value"
							spellcheck={false}
							hint={t("tuning.extraArgsHint")}
							onchange={(e) => app.saveSettings({ engine: { ...dto.settings.engine, extraArgs: e.currentTarget.value } })}
						/>
					</Section>

					<Section title={t("tuning.sec.look")}>
						<div class="grid">
							<Segments label={t("tuning.theme")} options={themeOptions} value={dto.settings.theme} onchange={(v) => app.saveSettings({ theme: v as ThemePref })} />
							<Segments label={t("tuning.language")} options={localeOptions} value={dto.settings.locale} onchange={(v) => app.saveSettings({ locale: v as LocalePref })} />
						</div>
						<Swatches label={t("tuning.accent")} value={dto.settings.accent} onchange={(v) => app.saveSettings({ accent: v })} />
					</Section>
				{/if}
			</div>
		</Core>
	</div>

	<footer class="foot" data-part>
		<span>{t("tuning.engineLine", { angle: dto.settings.engine.angle.toUpperCase(), cache: dto.settings.engine.diskCacheMb / 1024 })}</span>
		<span class="sp"></span>
		<span class="legend"><i aria-hidden="true"></i>{t("tuning.modifiedLegend")}</span>
		<Button disabled={n === 0} onclick={() => app.resetOverrides()}>{t("tuning.reset")}</Button>
	</footer>
</div>

<style>
	.tuning {
		height: 100%;
		display: grid;
		grid-template-rows: auto minmax(0, 1fr) auto;
		gap: 10px;
		padding: 2px 14px 14px;
	}
	.head {
		display: flex;
		align-items: center;
		gap: 10px;
		min-width: 0;
	}
	h1 {
		font: 700 18px/1.1 var(--tc-font-display);
		letter-spacing: -0.02em;
	}
	.for {
		min-width: 0;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
		font: 500 12.5px/1 var(--tc-font-mono);
		color: var(--tc-muted);
	}
	.sp {
		flex: 1;
	}
	.scope {
		width: 300px;
		flex: none;
	}
	.body {
		min-height: 0;
	}
	.sections {
		display: grid;
		gap: 26px;
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 16px 24px;
		align-items: start;
	}
	.toggles {
		gap: 0 24px;
	}
	.foot {
		display: flex;
		align-items: center;
		gap: 12px;
		font: 500 11px/1 var(--tc-font-mono);
		color: var(--tc-muted);
	}
	.legend {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}
	.legend i {
		width: 6px;
		height: 6px;
		border-radius: 99px;
		background: var(--tc-accent);
		box-shadow: 0 0 6px var(--tc-glow);
	}
</style>
