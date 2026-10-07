<script lang="ts">
	import Button from "../components/tactile/Button.svelte";
	import Chip from "../components/tactile/Chip.svelte";
	import Core from "../components/tactile/Core.svelte";
	import Field from "../components/tactile/Field.svelte";
	import Icon from "../components/tactile/Icon.svelte";
	import IconButton from "../components/tactile/IconButton.svelte";
	import Label from "../components/tactile/Label.svelte";
	import Pill from "../components/tactile/Pill.svelte";
	import Primary from "../components/tactile/Primary.svelte";
	import Section from "../components/tactile/Section.svelte";
	import Shell from "../components/tactile/Shell.svelte";
	import Tray from "../components/tactile/Tray.svelte";
	import Segments from "../components/tactile/Segments.svelte";
	import Toggle from "../components/tactile/Toggle.svelte";
	import Slider from "../components/tactile/Slider.svelte";
	import Band from "../components/tactile/Band.svelte";
	import Select from "../components/tactile/Select.svelte";
	import Swatches from "../components/tactile/Swatches.svelte";
	import type { AccentId } from "../lib/types";

	let text = $state("Проклятие Страда");
	let seg = $state<"quality" | "balance" | "potato" | "manual">("balance");
	let on = $state(true);
	let fps = $state(60);
	let accent = $state<AccentId>("peach");
	let angle = $state("d3d11");
</script>

<!-- Витрина примитивов для превью: http://localhost:1420/?kit (только dev) -->
<div class="kit">
	<Shell pad="18px">
		<Section title="Кнопки">
			<div class="row">
				<Button>Обычная</Button>
				<Button danger>Удалить</Button>
				<Button disabled>Отключена</Button>
				<IconButton label="Изменить"><Icon name="edit" /></IconButton>
				<IconButton label="Настройка" size={30} on><Icon name="sliders" /></IconButton>
				<Primary>Сохранить</Primary>
				<Primary tone="danger" icon="trash">Удалить</Primary>
				<Primary busy>Занято</Primary>
			</div>
			<Primary size="lg">ЗАПУСК</Primary>
		</Section>
		<Section title="Метки">
			<div class="row">
				<Label>Профиль</Label>
				<Label modified>Изменено</Label>
				<Pill>accent</Pill><Pill tone="plain">plain</Pill><Pill tone="warn">warn</Pill><Pill tone="good">онлайн</Pill><Pill tone="danger">не отвечает</Pill>
				<Chip>v0.2.2</Chip><Chip button on>60</Chip>
			</div>
		</Section>
		<Section title="Поля">
			<Field label="Название" bind:value={text} />
			<Field label="Адрес" mono value="vtt.example.com" error="Не похоже на адрес" />
		</Section>
		<Section title="Выбор">
			<Segments
				label="Профиль"
				options={[
					{ value: "quality", label: "Качество", short: "КАЧ" },
					{ value: "balance", label: "Баланс", short: "БАЛ" },
					{ value: "potato", label: "Картошка", short: "КРТ" },
					{ value: "manual", label: "Ручной", short: "РУЧ", arrowSkip: true }
				]}
				value={seg}
				onchange={(v) => (seg = v)}
			/>
			<Toggle label="Анимация света" checked={on} modified onchange={(v) => (on = v)} />
			<Toggle label="Удалить данные" danger checked={!on} onchange={(v) => (on = !v)} />
			<Slider label="Макс. FPS" value={fps} min={20} max={240} step={1} ticks={[60, 144, 240]} presets={[60, 144, 240]} editable format={String} onchange={(v) => (fps = v)} />
			<Band value={0.42} ghost={0.6} stops={[0.25, 0.5, 0.75]} label="Кэш" />
			<Select label="ANGLE" options={[{ value: "d3d11", label: "D3D11" }, { value: "gl", label: "GL" }]} value={angle} onchange={(v) => (angle = v)} />
			<Swatches label="Акцент" value={accent} onchange={(v) => (accent = v)} />
		</Section>
	</Shell>
	<Tray pad="10px"><Label>Лоток</Label></Tray>
	<Core><Label>Пластина</Label></Core>
</div>

<style>
	.kit {
		height: 100%;
		overflow-y: auto;
		display: grid;
		gap: 14px;
		align-content: start;
		padding: 2px 14px 14px;
	}
	.row {
		display: flex;
		flex-wrap: wrap;
		gap: 10px;
		align-items: center;
	}
</style>
