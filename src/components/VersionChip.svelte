<script lang="ts">
	import changelog from "../../CHANGELOG.md?raw";
	import { notesFor } from "../lib/changelog";
	import { t } from "../lib/i18n.svelte";
	import { app } from "../lib/store.svelte";
	import Chip from "./tactile/Chip.svelte";
	import Pill from "./tactile/Pill.svelte";

	let { version }: { version: string } = $props();

	const notes = $derived(notesFor(changelog, version));
	const check = $derived(app.updateCheck);
	const status = $derived(
		check === "idle"
			? ""
			: check === "available" && app.update
				? t("update.state.available", { version: app.update.version })
				: t(`update.state.${check}`)
	);
</script>

<!-- версия — кнопка ручной проверки обновлений; в подсказке — что нового в этой версии -->
<Chip
	button
	aria-label={t("update.check")}
	aria-busy={check === "checking"}
	onclick={() => app.checkUpdate(true)}
	tip={{ title: t("version.notesTitle", { version }), body: notes ? t("update.checkHint") : `${t("version.noNotes")} ${t("update.checkHint")}`, lines: notes ?? undefined }}
>
	v{version}
</Chip>
{#if status}<Pill tone={check === "failed" ? "warn" : "plain"} role="status">{status}</Pill>{/if}
