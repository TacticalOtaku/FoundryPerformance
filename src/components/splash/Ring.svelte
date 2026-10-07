<script lang="ts">
	import type { RingMode } from "../../lib/splash";

	let { mode, pct = 0, label }: { mode: RingMode; pct?: number; label: string } = $props();

	const uid = $props.id();
	const C = 2 * Math.PI * 60;
	// Полное кольцо — без пунктира: «C 0» Chrome рисует половиной окружности
	const dash = $derived(
		mode === "progress" ? `${(C * Math.min(100, Math.max(0, pct))) / 100} ${C}` : mode === "spin" ? `${C / 4} ${C}` : mode === "muted" ? "4 14" : "none"
	);
	// Свечение — SVG-фильтр с запасом области: CSS drop-shadow обрезается границей SVG квадратом
	const glow = $derived(mode === "progress" || mode === "breathe" || mode === "full");
	const bar = $derived(mode === "progress");
</script>

<div
	class="ring {mode}"
	role={bar ? "progressbar" : undefined}
	aria-label={bar ? label : undefined}
	aria-valuemin={bar ? 0 : undefined}
	aria-valuemax={bar ? 100 : undefined}
	aria-valuenow={bar ? pct : undefined}
	aria-hidden={bar ? undefined : "true"}
>
	<svg viewBox="0 0 150 150" width="150" height="150">
		<defs>
			<filter id="{uid}-halo" x="-50%" y="-50%" width="200%" height="200%" color-interpolation-filters="sRGB">
				<feGaussianBlur in="SourceGraphic" stdDeviation="7" result="b" />
				<feMerge>
					<feMergeNode in="b" />
					<feMergeNode in="b" />
					<feMergeNode in="SourceGraphic" />
				</feMerge>
			</filter>
			<linearGradient id="{uid}-key" x1="0" y1="0" x2="0" y2="1">
				<stop offset="0" class="key-top" />
				<stop offset="1" class="key-bottom" />
			</linearGradient>
		</defs>
		<circle class="track" cx="75" cy="75" r="60" />
		<g class="arc-wrap">
			<circle class="arc" cx="75" cy="75" r="60" stroke-dasharray={dash} filter={glow ? `url(#${uid}-halo)` : undefined} />
		</g>
		<circle class="key-shadow" cx="75" cy="78" r="38" />
		<circle cx="75" cy="75" r="38" fill="url(#{uid}-key)" />
		<circle class="dot" cx="75" cy="75" r="10" />
	</svg>
</div>

<style>
	.ring {
		width: 150px;
		height: 150px;
		flex: none;
	}
	svg {
		display: block;
		overflow: visible;
		/* клики проходят к области перетаскивания сплэша */
		pointer-events: none;
	}
	.track {
		fill: none;
		stroke: var(--tc-sunken);
		stroke-width: 12;
	}
	.arc {
		fill: none;
		stroke: var(--tc-accent);
		stroke-width: 12;
		stroke-linecap: round;
		transition: stroke-dasharray 200ms var(--tc-ease);
	}
	.arc-wrap {
		transform-origin: 75px 75px;
		transform: rotate(-90deg);
	}
	.spin .arc-wrap {
		animation: fp-spin 1.1s linear infinite;
	}
	.breathe .arc {
		animation: fp-breathe 1.6s ease-in-out infinite;
	}
	.full .arc,
	.breathe .arc,
	.error .arc,
	.muted .arc {
		stroke-linecap: butt;
	}
	.muted .arc {
		stroke: var(--tc-muted);
		stroke-opacity: 0.5;
	}
	.error .arc {
		stroke: var(--tc-danger);
	}
	.key-top {
		stop-color: color-mix(in oklab, var(--tc-surface), white 8%);
	}
	.key-bottom {
		stop-color: var(--tc-bg);
	}
	.key-shadow {
		fill: #000;
		opacity: 0.22;
	}
	.dot {
		fill: var(--tc-accent);
	}
	.muted .dot {
		fill: var(--tc-muted);
	}
	.error .dot {
		fill: var(--tc-danger);
	}
	@keyframes fp-spin {
		to {
			transform: rotate(270deg);
		}
	}
	@keyframes fp-breathe {
		50% {
			opacity: 0.45;
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.spin .arc-wrap {
			animation: none;
		}
		.spin .arc {
			animation: fp-breathe 1.6s ease-in-out infinite;
		}
	}
</style>
