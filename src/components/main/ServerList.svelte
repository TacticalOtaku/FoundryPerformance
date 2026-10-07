<script lang="ts">
	import { t } from "../../lib/i18n.svelte";
	import { hostOf, lastRun, STATUS_TONE } from "../../lib/intent";
	import { slotStatus } from "../../lib/status";
	import { app } from "../../lib/store.svelte";
	import Icon from "../tactile/Icon.svelte";
	import IconButton from "../tactile/IconButton.svelte";
	import Label from "../tactile/Label.svelte";
	import Tray from "../tactile/Tray.svelte";

	const dto = $derived(app.dto!);

	/** Клик выбирает; Enter на уже выбранной строке и двойной клик — запускают. */
	function pick(e: MouseEvent, id: string, selected: boolean) {
		if (selected && e.detail === 0) void app.launch(e.shiftKey);
		else app.selectedId = id;
	}
</script>

<Tray fill scroll pad="6px">
	<div class="lbl"><Label>{t("main.slots")}</Label></div>
	<div class="rows">
		{#each dto.servers as s (s.id)}
			{@const status = slotStatus(s, app.probes[s.id])}
			{@const run = lastRun(dto.stats[s.id])}
			{@const selected = s.id === app.selectedId}
			<div class="row" class:on={selected}>
				<button type="button" class="pick" aria-current={selected} onclick={(e) => pick(e, s.id, selected)} ondblclick={() => app.launch(false)}>
					<span class="dot {STATUS_TONE[status]}" class:hollow={status !== "running" && status !== "idle"} aria-hidden="true"></span>
					<span class="text">
						<span class="name">{s.name}</span>
						<span class="meta" class:bad={status === "stopped"}>
							{hostOf(s.url)} · {status === "stopped" || !run ? t(`status.${status}`) : `${run.fps} FPS`}
						</span>
					</span>
				</button>
				<span class="edit">
					<IconButton label={t("slot.editAria", { name: s.name })} onclick={() => app.editSlot(s)}><Icon name="edit" size={14} /></IconButton>
				</span>
			</div>
		{/each}
		<button type="button" class="add" onclick={() => app.newSlot()}><Icon name="plus" />{t("main.addServer")}</button>
	</div>
</Tray>

<style>
	.lbl {
		padding: 8px 10px 6px;
	}
	.rows {
		display: grid;
		gap: 2px;
	}
	.row {
		position: relative;
		border-radius: var(--tc-r-row);
		transition:
			background-color 240ms var(--tc-ease),
			box-shadow 240ms var(--tc-ease);
	}
	.row:hover {
		background: color-mix(in oklab, var(--tc-surface) 55%, transparent);
	}
	.row.on {
		background: var(--tc-surface);
		box-shadow:
			var(--tc-raise),
			0 0 0 1.5px var(--tc-accent),
			0 0 16px -4px var(--tc-glow);
	}
	.pick {
		display: flex;
		align-items: center;
		gap: 11px;
		width: 100%;
		height: 56px;
		padding: 0 44px 0 12px;
		border-radius: inherit;
		text-align: left;
	}
	.dot {
		--tone: var(--tc-muted);
		width: 7px;
		height: 7px;
		flex: none;
		border-radius: 99px;
		background: var(--tone);
	}
	.dot.good {
		--tone: var(--tc-good);
	}
	.dot.warn {
		--tone: var(--tc-warning);
	}
	.dot.danger {
		--tone: var(--tc-danger);
	}
	.dot.hollow {
		background: transparent;
		box-shadow: inset 0 0 0 1.5px var(--tone);
	}
	.text {
		flex: 1;
		min-width: 0;
	}
	.name,
	.meta {
		display: block;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}
	.name {
		font-weight: 600;
		font-size: 14px;
		line-height: 1.2;
	}
	.meta {
		margin-top: 3px;
		font: 500 11px/1.3 var(--tc-font-mono);
		color: var(--tc-muted);
	}
	.meta.bad {
		color: var(--tc-danger);
	}
	.edit {
		position: absolute;
		top: 50%;
		right: 8px;
		transform: translateY(-50%);
		opacity: 0;
		transition: opacity 160ms var(--tc-ease);
	}
	.row:hover .edit,
	.row.on .edit,
	.edit:focus-within {
		opacity: 1;
	}
	.add {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		width: 100%;
		height: 44px;
		margin-top: 4px;
		border-radius: var(--tc-r-row);
		color: var(--tc-muted);
		font-size: 13px;
		transition: color 160ms var(--tc-ease);
	}
	.add:hover {
		color: var(--tc-ink);
	}
</style>
