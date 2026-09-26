<script lang="ts">
	import { t } from "../lib/i18n.svelte";
	import type { LedState, ProbeResult, Server } from "../lib/types";
	import Led from "./Led.svelte";

	let {
		server,
		code,
		selected,
		probe,
		onselect,
		onlaunch,
		onedit
	}: {
		server: Server;
		code: string;
		selected: boolean;
		probe: ProbeResult | "pending" | undefined;
		onselect: () => void;
		onlaunch: () => void;
		onedit: () => void;
	} = $props();

	const led = $derived<LedState>(
		probe === undefined ? "off" : probe === "pending" ? "pending" : !probe.reachable || !probe.foundry ? "err" : probe.active ? "ok" : "warn"
	);
	const status = $derived(
		probe === undefined
			? ""
			: probe === "pending"
				? t("led.pending")
				: !probe.reachable
					? t("led.err")
					: !probe.foundry
						? t("led.notFoundry")
						: probe.active
							? t("led.ok", { world: probe.world ?? "" })
							: t("led.idle")
	);
	const host = $derived(server.url.replace(/^https?:\/\//, "").replace(/\/$/, ""));
</script>

<div
	class="slot"
	class:selected
	role="option"
	aria-selected={selected}
	tabindex="0"
	onclick={onselect}
	ondblclick={onlaunch}
	onkeydown={(e) => {
		if (e.key === "Enter") onlaunch();
		else if (e.key === " ") {
			e.preventDefault();
			onselect();
		}
	}}
>
	<span class="code mono">{code}</span>
	<span class="body">
		<span class="name">{server.name}</span>
		<span class="meta mono">{host}{#if status}&nbsp;· {status}{/if}</span>
	</span>
	<button
		class="edit mono"
		aria-label={t("slot.editAria", { name: server.name })}
		onclick={(e) => {
			e.stopPropagation();
			onedit();
		}}>{t("slot.editShort")}</button
	>
	<Led state={led} label={status} />
</div>

<style>
	.slot {
		display: grid;
		grid-template-columns: 34px minmax(0, 1fr) auto 12px;
		align-items: center;
		gap: 12px;
		padding: 12px 14px;
		background: var(--face-2);
		cursor: pointer;
		transition: background 0.12s;
	}
	.slot:hover {
		background: var(--face);
	}
	.slot.selected {
		background: var(--inverse-bg);
		color: var(--inverse-fg);
	}
	.code {
		color: var(--ink-2);
	}
	.selected .code,
	.selected .meta {
		color: var(--ink-3);
	}
	.body {
		display: grid;
		min-width: 0;
	}
	.name {
		font-weight: 500;
		font-size: 17px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.meta {
		color: var(--ink-2);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.edit {
		padding: 4px 8px;
		border: 1px solid var(--line);
		opacity: 0;
		transition: opacity 0.12s;
	}
	.slot:hover .edit,
	.slot:focus-within .edit,
	.selected .edit {
		opacity: 1;
	}
</style>
