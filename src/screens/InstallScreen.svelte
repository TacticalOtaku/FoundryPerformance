<script lang="ts">
	import Lcd from "../components/Lcd.svelte";
	import Toggle from "../components/Toggle.svelte";
	import { setupApi } from "../lib/api";
	import { t } from "../lib/i18n.svelte";
	import type { InstallInfo, InstallStep } from "../lib/types";

	let { info }: { info: InstallInfo } = $props();

	const STEPS: Exclude<InstallStep, "done">[] = ["check", "copy", "shortcuts", "register"];

	// Папка по умолчанию, пока пользователь не выбрал свою
	let dir = $derived(info.defaultDir);
	let desktop = $state(true);
	let launch = $state(true);
	let dirError = $state<string | null>(null);
	let phase = $state<"idle" | "running" | "done" | "error">("idle");
	let step = $state<InstallStep | null>(null);
	let pct = $state(0);
	let error = $state<string | null>(null);


	// Ручка «доворачивается» по мере установки: от КАЧ (−60°) до КРТ (+60°)
	const angle = $derived(-60 + (120 * pct) / 100);
	const title = $derived(
		info.state === "upgrade" ? t("install.titleUpgrade") : info.state === "current" ? t("install.titleCurrent") : t("install.title")
	);
	const lead = $derived(
		info.state === "current" ? t("install.leadCurrent", { version: info.existingVersion ?? "", dir: info.existingDir ?? "" }) : t("install.lead")
	);
	const goLabel = $derived(
		phase === "running"
			? t("install.running")
			: info.state === "upgrade"
				? t("install.upgrade", { from: info.existingVersion ?? "", to: info.currentVersion })
				: info.state === "current"
					? t("install.reinstall")
					: t("install.go")
	);

	function stepState(s: InstallStep): "done" | "now" | "fail" | "todo" {
		if (!step) return "todo";
		const cur = step === "done" ? STEPS.length : STEPS.indexOf(step as (typeof STEPS)[number]);
		const i = STEPS.indexOf(s as (typeof STEPS)[number]);
		if (i < cur) return "done";
		if (i === cur) return phase === "error" ? "fail" : "now";
		return "todo";
	}

	async function validate(): Promise<boolean> {
		try {
			await setupApi.checkDir(dir);
			dirError = null;
			return true;
		} catch (e) {
			dirError = t(String(e));
			return false;
		}
	}

	async function browse() {
		const picked = await setupApi.pickDir(dir);
		if (picked) {
			dir = picked;
			await validate();
		}
	}

	async function run() {
		if (phase === "running" || !(await validate())) return;
		phase = "running";
		error = null;
		pct = 0;
		step = null;
		const off = await setupApi.onProgress((p) => {
			step = p.step;
			pct = p.pct;
		});
		try {
			await setupApi.install({ dir, desktop, launch });
			step = "done";
			pct = 100;
			phase = "done";
		} catch (e) {
			phase = "error";
			error = t(String(e));
		} finally {
			off();
		}
	}
</script>

<div class="install">
	<section class="left">
		<header>
			<h1>{title}</h1>
			<p>{lead}</p>
		</header>

		{#if phase === "done"}
			<div class="done">
				<p>{t("install.done")}</p>
				{#if !launch}
					<button class="go" onclick={() => setupApi.openInstalled()}>
						<span>{t("install.openInstalled")}</span><span aria-hidden="true">▶</span>
					</button>
				{/if}
			</div>
		{:else}
			<label class="field">
				<span class="mono">{t("install.dir")}</span>
				<span class="row">
					<input
						class="mono"
						bind:value={dir}
						spellcheck="false"
						disabled={phase === "running"}
						aria-invalid={Boolean(dirError)}
						onchange={validate}
					/>
					<button class="browse mono" onclick={browse} disabled={phase === "running"}>{t("install.browse")}</button>
				</span>
				{#if dirError}
					<span class="err mono" role="alert">{dirError}</span>
				{:else}
					<span class="hint mono">{t("install.dirHint")}</span>
				{/if}
			</label>

			<div class="toggles">
				<Toggle label={t("install.desktop")} checked={desktop} onchange={(v) => (desktop = v)} />
				<Toggle label={t("install.launch")} checked={launch} onchange={(v) => (launch = v)} />
			</div>

			{#if error}<div class="err mono" role="alert">{error}</div>{/if}

			<div class="actions">
				{#if info.state === "current" && phase !== "running"}
					<button class="go" onclick={() => setupApi.openInstalled()}>
						<span>{t("install.openInstalled")}</span><span aria-hidden="true">▶</span>
					</button>
					<button class="secondary mono" onclick={run}>{goLabel}</button>
				{:else}
					<button class="go" aria-busy={phase === "running"} onclick={run}>
						<span>{goLabel}</span><span aria-hidden="true">▶</span>
					</button>
				{/if}
			</div>
		{/if}
	</section>

	<aside class="right">
		<Lcd value={pct} unit="%" caption={step ? t(`install.step.${step}`) : `v${info.currentVersion}`} />
		<div class="knob" aria-hidden="true">
			<div class="cap" style:transform={`rotate(${angle}deg)`}><i></i></div>
		</div>
		<ol class="steps mono">
			{#each STEPS as s (s)}
				<li class={stepState(s)}>
					<span aria-hidden="true">{{ done: "✓", now: "●", fail: "✕", todo: "○" }[stepState(s)]}</span>
					{t(`install.step.${s}`)}
				</li>
			{/each}
		</ol>
	</aside>
</div>

<style>
	.install {
		display: grid;
		grid-template-columns: minmax(0, 1fr) 232px;
		gap: 2px;
		height: 100%;
		min-height: 0;
	}
	.left,
	.right {
		background: var(--grain), var(--face);
		box-shadow: var(--bevel);
		padding: 22px;
		min-height: 0;
	}
	.left {
		display: grid;
		align-content: start;
		gap: 20px;
	}
	h1 {
		margin: 0;
		font-weight: 800;
		font-size: 30px;
		line-height: 1.05;
		letter-spacing: -0.01em;
	}
	header p {
		margin: 8px 0 0;
		color: var(--ink-2);
		font-size: 14px;
		max-width: 460px;
		overflow-wrap: anywhere;
	}
	.field {
		display: grid;
		gap: 6px;
	}
	.row {
		display: flex;
		gap: 6px;
	}
	input {
		flex: 1;
		min-width: 0;
		height: 38px;
		padding: 0 10px;
		background: var(--well);
		border: 1px solid transparent;
		box-shadow: var(--recess);
	}
	input[aria-invalid="true"] {
		border-color: var(--led-err);
	}
	.browse,
	.secondary {
		padding: 0 14px;
		border: 1px solid var(--ink-3);
	}
	.browse:hover,
	.secondary:hover {
		border-color: var(--signal);
	}
	.secondary {
		height: 52px;
	}
	.hint {
		color: var(--ink-2);
	}
	.err {
		color: var(--led-err);
	}
	.toggles {
		display: grid;
		gap: 8px;
		max-width: 360px;
	}
	.actions {
		display: flex;
		gap: 8px;
		align-items: stretch;
	}
	.go {
		flex: 1;
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
			box-shadow 0.06s;
	}
	.go:active {
		transform: translateY(3px);
		box-shadow: 0 0 0 var(--signal-deep);
	}
	.go[aria-busy="true"] {
		cursor: progress;
	}
	.done p {
		margin: 0 0 16px;
		font-size: 16px;
	}
	.right {
		display: grid;
		align-content: start;
		justify-items: center;
		gap: 18px;
	}
	.right :global(.lcd) {
		width: 100%;
	}
	.knob {
		width: 116px;
		height: 116px;
		border-radius: 50%;
		background: var(--line);
		display: grid;
		place-items: center;
	}
	.cap {
		position: relative;
		width: 88px;
		height: 88px;
		border-radius: 50%;
		background: var(--inverse-bg);
		transition: transform 0.45s var(--ease-detent);
	}
	.cap i {
		position: absolute;
		left: 50%;
		top: 8px;
		width: 5px;
		height: 28px;
		margin-left: -2.5px;
		background: var(--signal);
	}
	.steps {
		list-style: none;
		margin: 0;
		padding: 0;
		width: 100%;
		display: grid;
		gap: 6px;
		color: var(--ink-3);
	}
	.steps .done {
		color: var(--ink);
	}
	.steps .now {
		color: var(--signal-text);
	}
	.steps .fail {
		color: var(--led-err);
	}
</style>
