<script lang="ts">
	import { t } from "../lib/i18n.svelte";
	import type { KnobPosition } from "../lib/levers";
	import { tip, tipFor } from "../lib/tooltip.svelte";
	import type { ProfileId } from "../lib/types";

	let { value, onchange }: { value: KnobPosition; onchange: (p: KnobPosition) => void } = $props();

	const PRESETS: ProfileId[] = ["quality", "balance", "potato"];
	const POSITIONS: KnobPosition[] = [...PRESETS, "manual"];
	// Фиксаторы по дуге сверху: КАЧ слева, РУЧ справа
	const ANGLE: Record<KnobPosition, number> = { quality: -75, balance: -25, potato: 25, manual: 75 };
	const R_LABEL = 80;
	const R_TICK = 66;
	const CX = 95;
	const CY = 92;

	const polar = (deg: number, r: number) => {
		const rad = (deg * Math.PI) / 180;
		return { x: CX + r * Math.sin(rad), y: CY - r * Math.cos(rad) };
	};

	// Колесо и стрелки ходят только по пресетам; «РУЧ» — отдельное действие (вернуть правки или открыть настройку)
	function step(delta: number) {
		const from = value === "manual" ? PRESETS.length : PRESETS.indexOf(value);
		const i = Math.max(0, Math.min(PRESETS.length - 1, from + delta));
		if (PRESETS[i] !== value) onchange(PRESETS[i]);
	}

	function onkeydown(e: KeyboardEvent) {
		if (e.key === "ArrowRight" || e.key === "ArrowUp") step(1);
		else if (e.key === "ArrowLeft" || e.key === "ArrowDown") step(-1);
		else return;
		e.preventDefault();
	}

	const label = (p: KnobPosition) => t(`profile.${p}`);
	const tipOf = (p: KnobPosition) => tipFor(p, t(`profile.${p}.long`));
</script>

<div class="dial" style:--cx={`${CX}px`} style:--cy={`${CY}px`}>
	{#each POSITIONS as p (p)}
		{@const tick = polar(ANGLE[p], R_TICK)}
		{@const at = polar(ANGLE[p], R_LABEL)}
		<span class="tick" class:on={p === value} style:left={`${tick.x}px`} style:top={`${tick.y}px`} style:rotate={`${ANGLE[p]}deg`}></span>
		<button
			class="label silk"
			class:on={p === value}
			class:manual={p === "manual"}
			style:left={`${at.x}px`}
			style:top={`${at.y}px`}
			use:tip={tipOf(p)}
			onclick={() => onchange(p)}>{label(p)}</button
		>
	{/each}

	<div
		class="knob"
		role="slider"
		tabindex="0"
		aria-label={t("knob.label")}
		aria-valuemin={0}
		aria-valuemax={3}
		aria-valuenow={POSITIONS.indexOf(value)}
		aria-valuetext={t(`profile.${value}.long`)}
		{onkeydown}
		onwheel={(e) => step(e.deltaY > 0 ? 1 : -1)}
		onclick={() => step(value === "potato" || value === "manual" ? -9 : 1)}
		use:tip={tipFor("profile", t("knob.label"))}
	>
		<div class="cap" style:rotate={`${ANGLE[value]}deg`}><i></i></div>
	</div>
</div>

<style>
	.dial {
		position: relative;
		width: 190px;
		height: 168px;
	}
	.knob {
		position: absolute;
		left: calc(var(--cx) - 58px);
		top: calc(var(--cy) - 58px);
		width: 116px;
		height: 116px;
		border-radius: 50%;
		display: grid;
		place-items: center;
		cursor: pointer;
		/* рифлёное кольцо */
		background: repeating-conic-gradient(var(--knurl-a) 0 4deg, var(--knurl-b) 4deg 8deg);
		box-shadow: var(--lift), 0 8px 18px rgb(0 0 0 / 0.35);
	}
	.cap {
		position: relative;
		width: 88px;
		height: 88px;
		border-radius: 50%;
		background: radial-gradient(circle at 38% 30%, var(--cap-hi), var(--cap-lo) 85%);
		box-shadow:
			inset 0 -3px 6px rgb(0 0 0 / 0.3),
			inset 0 2px 1px rgb(255 255 255 / 0.25),
			0 2px 4px rgb(0 0 0 / 0.4);
		transition: rotate 0.34s var(--ease-detent);
	}
	.cap i {
		position: absolute;
		left: 50%;
		top: 7px;
		width: 5px;
		height: 27px;
		margin-left: -2.5px;
		border-radius: 2px;
		background: var(--signal);
		box-shadow: var(--glow-signal);
	}
	.tick {
		position: absolute;
		width: 2px;
		height: 7px;
		margin: -3.5px 0 0 -1px;
		background: var(--ink-3);
	}
	.tick.on {
		background: var(--signal);
		box-shadow: var(--glow-signal);
	}
	.label {
		position: absolute;
		translate: -50% -50%;
		padding: 3px 4px;
		transition: color 0.15s;
	}
	.label:hover {
		color: var(--ink);
	}
	.label.on {
		color: var(--signal-text);
		text-shadow: var(--glow-signal);
	}
</style>
