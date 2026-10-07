<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLButtonAttributes } from "svelte/elements";
	import { press } from "../../lib/motion";
	import Icon, { type IconName } from "./Icon.svelte";

	type Props = HTMLButtonAttributes & { children: Snippet; size?: "sm" | "lg"; tone?: "accent" | "danger"; busy?: boolean; icon?: IconName };
	let { children, size = "sm", tone = "accent", busy = false, icon = "enter", type = "button", disabled, ...rest }: Props = $props();
</script>

<!-- главное действие: акцентная капсула со вложенной лункой справа (кнопка в кнопке) -->
<button {type} class="primary {size} {tone}" aria-busy={busy} disabled={disabled || busy} use:press {...rest}>
	<span class="text">{@render children()}</span>
	<span class="nest" aria-hidden="true"><Icon name={icon} size={size === "lg" ? 20 : 14} /></span>
</button>

<style>
	.primary {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		border-radius: var(--tc-r-ctl);
		background: var(--tc-accent);
		color: var(--tc-on-accent);
		font-weight: 700;
		white-space: nowrap;
		box-shadow:
			inset 0 1px 0 rgb(255 255 255 / 40%),
			0 8px 18px -8px var(--tc-glow);
		transition:
			box-shadow 240ms var(--tc-ease),
			opacity 160ms var(--tc-ease);
	}
	.sm {
		height: 34px;
		padding: 0 3px 0 16px;
		font-size: 12.5px;
	}
	.lg {
		width: 100%;
		height: 58px;
		padding: 0 5px 0 26px;
		font-size: 17px;
		letter-spacing: 0.08em;
		box-shadow:
			inset 0 1px 0 rgb(255 255 255 / 40%),
			0 14px 28px -12px var(--tc-glow);
	}
	.text {
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.nest {
		display: grid;
		place-items: center;
		flex: none;
		border-radius: var(--tc-r-ctl);
		background: rgb(255 255 255 / 28%);
	}
	.sm .nest {
		width: 28px;
		height: 28px;
	}
	.lg .nest {
		width: 48px;
		height: 48px;
	}
	.danger {
		background: var(--tc-danger);
		color: #fff;
		box-shadow:
			inset 0 1px 0 rgb(255 255 255 / 30%),
			0 8px 18px -8px color-mix(in oklab, var(--tc-danger) 60%, transparent);
	}
	:global(.tc-root[data-theme="dark"]) .danger {
		color: var(--tc-sunken);
	}
	.primary:disabled {
		opacity: 0.45;
		box-shadow: none;
		cursor: not-allowed;
	}
	.primary[aria-busy="true"] {
		opacity: 0.75;
		cursor: progress;
	}
</style>
