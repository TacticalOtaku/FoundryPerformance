<script lang="ts">
	import Segmented from "../components/Segmented.svelte";
	import { t } from "../lib/i18n.svelte";
	import { app } from "../lib/store.svelte";
	import type { ProfileId } from "../lib/types";

	const draft = $derived(app.editing!);
	const index = $derived(app.dto!.servers.findIndex((s) => s.id === draft.id));
	const isNew = $derived(index < 0);
	let errors = $state<{ name?: string; url?: string }>({});
	let confirmDelete = $state(false);
	let saving = $state(false);

	const profileOptions = $derived([
		{ value: "inherit" as ProfileId | "inherit", label: t("profile.inherit") },
		...(["quality", "balance", "potato"] as ProfileId[]).map((p) => ({ value: p as ProfileId | "inherit", label: t(`profile.${p}.long`).toUpperCase() }))
	]);

	async function save(e: SubmitEvent) {
		e.preventDefault();
		errors = {};
		if (!draft.name.trim()) errors.name = t("err.nameRequired");
		if (!draft.url.trim()) errors.url = t("err.urlInvalid");
		if (errors.name || errors.url) return;
		saving = true;
		const err = await app.saveServer(draft);
		saving = false;
		if (err === "err.urlInvalid") errors.url = t(err);
		else if (err === "err.nameRequired") errors.name = t(err);
		else if (err) app.error = err;
	}
</script>

<form class="editor" onsubmit={save} novalidate>
	<header class="head">
		<b>{isNew ? t("slot.new") : t("slot.edit", { code: `A${index + 1}` })}</b>
	</header>

	<div class="fields">
		<label class="field">
			<span class="mono">{t("slot.name")}</span>
			<input
				bind:value={draft.name}
				placeholder={t("slot.namePlaceholder")}
				maxlength="60"
				aria-invalid={Boolean(errors.name)}
				oninput={() => (errors.name = undefined)}
			/>
			{#if errors.name}<span class="err mono" role="alert">{errors.name}</span>{/if}
		</label>
		<label class="field">
			<span class="mono">{t("slot.url")}</span>
			<input
				class="mono"
				bind:value={draft.url}
				placeholder={t("slot.urlPlaceholder")}
				spellcheck="false"
				aria-invalid={Boolean(errors.url)}
				oninput={() => (errors.url = undefined)}
			/>
			{#if errors.url}<span class="err mono" role="alert">{errors.url}</span>{/if}
		</label>
		<Segmented
			label={t("slot.profile")}
			options={profileOptions}
			value={draft.profile ?? "inherit"}
			onchange={(v) => (draft.profile = v === "inherit" ? null : v)}
		/>
		{#if !isNew}
			<button type="button" class="link mono" onclick={() => app.openTuning({ kind: "server", id: draft.id })}>{t("slot.tune")}</button>
		{/if}
	</div>

	<footer class="actions">
		<button type="submit" class="primary" aria-busy={saving}>{t("slot.save")}</button>
		<button type="button" class="secondary mono" onclick={() => app.closeScreen()}>{t("slot.cancel")}</button>
		{#if !isNew}
			<button
				type="button"
				class="danger mono"
				class:armed={confirmDelete}
				onclick={() => (confirmDelete ? app.deleteServer(draft.id) : (confirmDelete = true))}
				onblur={() => (confirmDelete = false)}
			>
				{confirmDelete ? t("slot.deleteConfirm") : t("slot.delete")}
			</button>
		{/if}
	</footer>
</form>

<style>
	.editor {
		display: grid;
		grid-template-rows: auto minmax(0, 1fr) auto;
		gap: 2px;
		height: 100%;
	}
	.head,
	.fields,
	.actions {
		background: var(--grain), var(--face);
		box-shadow: var(--bevel);
		padding: 14px 18px;
	}
	.head b {
		font-weight: 800;
		letter-spacing: 0.08em;
	}
	.fields {
		display: grid;
		align-content: start;
		gap: 18px;
	}
	.field {
		display: grid;
		gap: 6px;
		max-width: 560px;
	}
	input {
		height: 42px;
		padding: 0 12px;
		background: var(--well);
		border: 1px solid transparent;
		box-shadow: var(--recess);
		font-size: 15px;
	}
	input[aria-invalid="true"] {
		border-color: var(--led-err);
	}
	.err {
		color: var(--led-err);
	}
	.link {
		justify-self: start;
		color: var(--signal-text);
	}
	.actions {
		display: flex;
		gap: 8px;
		align-items: center;
	}
	.primary {
		padding: 11px 22px;
		background: var(--signal);
		color: var(--signal-ink);
		font-weight: 800;
		letter-spacing: 0.06em;
	}
	.secondary,
	.danger {
		padding: 10px 14px;
		border: 1px solid var(--ink-3);
	}
	.danger {
		margin-left: auto;
		color: var(--led-err);
		border-color: var(--led-err);
	}
	.danger.armed {
		background: var(--led-err);
		color: var(--face);
	}
</style>
