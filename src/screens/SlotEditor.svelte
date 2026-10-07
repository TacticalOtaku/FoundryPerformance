<script lang="ts">
	import Button from "../components/tactile/Button.svelte";
	import Field from "../components/tactile/Field.svelte";
	import Primary from "../components/tactile/Primary.svelte";
	import Segments from "../components/tactile/Segments.svelte";
	import Shell from "../components/tactile/Shell.svelte";
	import { t } from "../lib/i18n.svelte";
	import { openWindow } from "../lib/motion";
	import { app } from "../lib/store.svelte";
	import type { ProfileId } from "../lib/types";

	const draft = $derived(app.editing!);
	const stored = $derived(app.dto!.servers.find((s) => s.id === draft.id) ?? null);
	const isNew = $derived(stored === null);
	let errors = $state<{ name?: string; url?: string }>({});
	let confirmDelete = $state(false);
	let saving = $state(false);

	const profileOptions = $derived([
		{ value: "inherit" as ProfileId | "inherit", label: t("profile.inherit") },
		...(["quality", "balance", "potato"] as ProfileId[]).map((p) => ({ value: p as ProfileId | "inherit", label: t(`profile.${p}.long`) }))
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

<div class="wrap" use:openWindow>
	<form class="editor" onsubmit={save} novalidate data-part>
		<Shell pad="22px 24px 20px">
			<h1>{isNew ? t("slot.new") : stored!.name}</h1>
			<div class="fields">
				<Field
					label={t("slot.name")}
					bind:value={draft.name}
					placeholder={t("slot.namePlaceholder")}
					maxlength={60}
					error={errors.name}
					oninput={() => (errors.name = undefined)}
				/>
				<Field
					label={t("slot.url")}
					mono
					bind:value={draft.url}
					placeholder={t("slot.urlPlaceholder")}
					spellcheck={false}
					error={errors.url}
					oninput={() => (errors.url = undefined)}
				/>
				<Segments label={t("slot.profile")} options={profileOptions} value={draft.profile ?? "inherit"} onchange={(v) => (draft.profile = v === "inherit" ? null : v)} />
				{#if !isNew}
					<button type="button" class="link" onclick={() => app.openTuning({ kind: "server", id: draft.id })}>{t("slot.tune")}</button>
				{/if}
			</div>
			<footer class="actions">
				{#if !isNew}
					<Button
						danger
						armed={confirmDelete}
						onclick={() => (confirmDelete ? app.deleteServer(draft.id) : (confirmDelete = true))}
						onblur={() => (confirmDelete = false)}
					>
						{confirmDelete ? t("slot.deleteConfirm") : t("slot.delete")}
					</Button>
				{/if}
				<span class="sp"></span>
				<Button onclick={() => app.closeScreen()}>{t("slot.cancel")}</Button>
				<Primary type="submit" busy={saving}>{t("slot.save")}</Primary>
			</footer>
		</Shell>
	</form>
</div>

<style>
	.wrap {
		height: 100%;
		display: grid;
		place-items: center;
		padding: 2px 14px 14px;
	}
	.editor {
		width: min(480px, 100%);
	}
	h1 {
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
		font: 700 18px/1.2 var(--tc-font-display);
		letter-spacing: -0.02em;
	}
	.fields {
		display: grid;
		gap: 16px;
		margin: 20px 0 22px;
	}
	.link {
		justify-self: start;
		font: 500 12px/1 var(--tc-font-mono);
		color: var(--tc-accent-text);
	}
	.link:hover {
		text-decoration: underline;
	}
	.actions {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.sp {
		flex: 1;
	}
</style>
