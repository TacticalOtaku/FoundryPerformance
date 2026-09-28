<script lang="ts">
	import changelog from "../../CHANGELOG.md?raw";
	import { notesFor } from "../lib/changelog";
	import { t } from "../lib/i18n.svelte";
	import { app } from "../lib/store.svelte";
	import { tip } from "../lib/tooltip.svelte";

	let { version }: { version: string } = $props();

	const notes = $derived(notesFor(changelog, version));
	const notesTip = $derived({
		title: t("version.notesTitle", { version }),
		body: notes ? "" : t("version.noNotes"),
		lines: notes ?? undefined
	});
	const check = $derived(app.updateCheck);
	const status = $derived(
		check === "idle" || check === "checking"
			? ""
			: check === "available" && app.update
				? t("update.state.available", { version: app.update.version })
				: t(`update.state.${check}`)
	);
</script>

<div class="version">
	<span class="status mono {check}" role="status">{status}</span>
	<!-- tabindex: список изменений доступен и с клавиатуры -->
	<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
	<span class="tag mono" tabindex="0" {@attach tip(() => notesTip)}>v{version}</span>
	<button
		class="check"
		class:busy={check === "checking"}
		class:ready={app.update !== null}
		aria-label={t("update.check")}
		aria-busy={check === "checking"}
		{@attach tip(() => ({ title: t("update.check"), body: t("update.checkBody") }))}
		onclick={() => app.checkUpdate(true)}
	>
		<svg viewBox="0 0 12 12" width="12" height="12" aria-hidden="true">
			<path d="M10.2 6.4A4.2 4.2 0 1 1 8.9 3" />
			<path d="M9.8 0.9v2.8H7" />
		</svg>
	</button>
</div>

<style>
	.version {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-left: auto;
	}
	.status {
		font-size: 11px;
		color: var(--ink-3);
	}
	.status.current {
		color: var(--led-ok);
	}
	.status.failed,
	.status.available {
		color: var(--signal-text);
	}
	.tag {
		padding: 2px 7px;
		color: var(--ink-2);
		background: var(--well);
		box-shadow: var(--recess);
		cursor: help;
	}
	.tag:hover {
		color: var(--ink);
	}
	/* мини-клавиша на панели: утоплена, пока идёт проверка */
	.check {
		position: relative;
		display: grid;
		place-items: center;
		width: 24px;
		height: 22px;
		color: var(--ink-2);
		background: var(--face-2);
		box-shadow: var(--lift);
	}
	.check:hover {
		color: var(--ink);
	}
	.check:active,
	.check.busy {
		box-shadow: var(--recess);
	}
	.check svg {
		fill: none;
		stroke: currentColor;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
	.busy svg {
		animation: spin 0.8s linear infinite;
	}
	/* найдено обновление — на клавише загорается сигнальная точка */
	.ready::after {
		content: "";
		position: absolute;
		top: 3px;
		right: 3px;
		width: 4px;
		height: 4px;
		border-radius: 50%;
		background: var(--signal);
		box-shadow: 0 0 4px var(--signal);
	}
	@keyframes spin {
		to {
			rotate: 360deg;
		}
	}
</style>
