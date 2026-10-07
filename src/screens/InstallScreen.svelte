<script lang="ts">
	import Band from "../components/tactile/Band.svelte";
	import Button from "../components/tactile/Button.svelte";
	import Field from "../components/tactile/Field.svelte";
	import Icon from "../components/tactile/Icon.svelte";
	import Primary from "../components/tactile/Primary.svelte";
	import Shell from "../components/tactile/Shell.svelte";
	import Toggle from "../components/tactile/Toggle.svelte";
	import Tray from "../components/tactile/Tray.svelte";
	import { setupApi } from "../lib/api";
	import { t } from "../lib/i18n.svelte";
	import { openWindow } from "../lib/motion";
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

	const title = $derived(
		info.state === "upgrade"
			? t("install.titleUpgrade", { version: info.currentVersion })
			: info.state === "current"
				? t("install.titleCurrent")
				: t("install.title")
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

<div class="install" use:openWindow>
	<Shell fill pad="26px 28px 24px">
		<div class="cols">
			<section class="left" data-part>
				<h1>{title}</h1>
				<p class="lead">{lead}</p>

				{#if phase === "done"}
					<p class="ok" role="status">{t("install.done")}</p>
					<div class="grow"></div>
					{#if !launch}<Primary size="lg" onclick={() => setupApi.openInstalled()}>{t("install.openInstalled")}</Primary>{/if}
				{:else}
					<div class="form">
						<Field
							label={t("install.dir")}
							mono
							bind:value={dir}
							spellcheck={false}
							disabled={phase === "running"}
							error={dirError}
							hint={t("install.dirHint")}
							onchange={validate}
						>
							{#snippet end()}
								<Button disabled={phase === "running"} onclick={browse}><Icon name="folder" size={14} />{t("install.browse")}</Button>
							{/snippet}
						</Field>
						<div>
							<Toggle label={t("install.desktop")} checked={desktop} disabled={phase === "running"} onchange={(v) => (desktop = v)} />
							<Toggle label={t("install.launch")} checked={launch} disabled={phase === "running"} onchange={(v) => (launch = v)} />
						</div>
						{#if error}<p class="err" role="alert">{error}</p>{/if}
					</div>
					<div class="grow"></div>
					{#if info.state === "current" && phase !== "running"}
						<div class="pair">
							<Button onclick={run}>{phase === "error" ? t("install.retry") : goLabel}</Button>
							<Primary size="lg" onclick={() => setupApi.openInstalled()}>{t("install.openInstalled")}</Primary>
						</div>
					{:else}
						<Primary size="lg" busy={phase === "running"} onclick={run}>{phase === "error" ? t("install.retry") : goLabel}</Primary>
					{/if}
				{/if}
			</section>

			<aside class="right" data-part>
				<Tray pad="10px 12px">
					<ol class="steps">
						{#each STEPS as s (s)}
							{@const st = stepState(s)}
							<li class={st}>
								<span class="dot" aria-hidden="true"></span>
								<span>{t(`install.step.${s}`)}</span>
							</li>
						{/each}
					</ol>
				</Tray>
				<Band value={pct / 100} label={t("install.progress")} />
				<span class="pct">{pct} %</span>
			</aside>
		</div>
	</Shell>
</div>

<style>
	.install {
		height: 100%;
		padding: 2px 14px 14px;
	}
	.cols {
		height: 100%;
		display: grid;
		grid-template-columns: minmax(0, 1.2fr) minmax(0, 1fr);
		gap: 28px;
	}
	.left {
		min-width: 0;
		display: flex;
		flex-direction: column;
	}
	h1 {
		font: 700 28px/1.1 var(--tc-font-display);
		letter-spacing: -0.03em;
		text-wrap: balance;
	}
	.lead {
		margin-top: 10px;
		color: var(--tc-muted);
	}
	.form {
		display: grid;
		gap: 16px;
		margin-top: 24px;
	}
	.ok {
		margin-top: 24px;
		font-weight: 600;
		color: var(--tc-good);
	}
	.err {
		font: 500 12px/1.4 var(--tc-font-mono);
		color: var(--tc-danger);
	}
	.grow {
		flex: 1;
		min-height: 16px;
	}
	.pair {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.right {
		display: flex;
		flex-direction: column;
		justify-content: center;
		gap: 12px;
	}
	.steps {
		display: grid;
		gap: 10px;
		list-style: none;
		font: 500 12px/1.3 var(--tc-font-mono);
		color: var(--tc-muted);
	}
	.steps li {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.dot {
		width: 8px;
		height: 8px;
		flex: none;
		border-radius: 99px;
		box-shadow: inset 0 0 0 1.5px var(--tc-muted);
	}
	.now {
		color: var(--tc-ink);
	}
	.now .dot {
		background: var(--tc-accent);
		box-shadow: 0 0 8px var(--tc-glow);
	}
	.done .dot {
		background: var(--tc-good);
		box-shadow: none;
	}
	.fail {
		color: var(--tc-danger);
	}
	.fail .dot {
		background: var(--tc-danger);
		box-shadow: none;
	}
	.pct {
		align-self: flex-end;
		font: 600 12.5px/1 var(--tc-font-mono);
	}
</style>
