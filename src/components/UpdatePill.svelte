<script lang="ts">
	import { noteLines } from "../lib/changelog";
	import { t } from "../lib/i18n.svelte";
	import { app } from "../lib/store.svelte";
	import Pill from "./tactile/Pill.svelte";
</script>

{#if app.update}
	{@const u = app.update}
	<Pill
		tone="accent"
		button
		onclick={() => void app.restartToUpdate()}
		tip={{ title: t("update.notesTitle", { version: u.version }), body: t("update.noNotes"), lines: noteLines(u.notes) }}
	>
		{t("update.available", { version: u.version })}
	</Pill>
{/if}
