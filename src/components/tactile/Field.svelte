<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLInputAttributes } from "svelte/elements";

	type Props = Omit<HTMLInputAttributes, "value"> & { label: string; value?: string; error?: string | null; hint?: string; mono?: boolean; end?: Snippet };
	let { label, value = $bindable(""), error = null, hint, mono = false, end, ...rest }: Props = $props();
	const id = $props.id();
	const note = $derived(error || hint ? `${id}-note` : undefined);
</script>

<div class="field">
	<label class="lbl" for={id}>{label}</label>
	<div class="row">
		<span class="cap" class:bad={Boolean(error)}>
			<input {id} class:mono bind:value aria-invalid={Boolean(error)} aria-describedby={note} {...rest} />
		</span>
		{#if end}{@render end()}{/if}
	</div>
	{#if error}<span class="note bad" id={note} role="alert">{error}</span>{:else if hint}<span class="note" id={note}>{hint}</span>{/if}
</div>

<style>
	.field {
		display: grid;
		gap: 6px;
		min-width: 0;
	}
	.lbl {
		font: 500 10.5px/1.3 var(--tc-font-mono);
		color: var(--tc-muted);
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
	}
	.cap {
		flex: 1;
		min-width: 0;
		display: flex;
		align-items: center;
		height: 34px;
		padding: 0 14px;
		border-radius: var(--tc-r-ctl);
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
		transition: box-shadow 160ms var(--tc-ease);
	}
	.cap:focus-within {
		box-shadow:
			var(--tc-press),
			0 0 0 2px var(--tc-accent);
	}
	.cap.bad {
		box-shadow:
			var(--tc-press),
			0 0 0 1.5px var(--tc-danger);
	}
	input {
		flex: 1;
		min-width: 0;
		padding: 0;
		border: 0;
		background: none;
		outline: none;
		font-size: 13px;
		color: var(--tc-ink);
	}
	input:focus-visible {
		outline: none;
	}
	input::placeholder {
		color: var(--tc-muted);
	}
	input:disabled {
		opacity: 0.6;
	}
	.mono {
		font-family: var(--tc-font-mono);
		font-size: 12.5px;
	}
	.note {
		font: 500 11px/1.35 var(--tc-font-mono);
		color: var(--tc-muted);
	}
	.note.bad {
		color: var(--tc-danger);
	}
</style>
