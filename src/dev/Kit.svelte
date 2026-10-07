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

	let text = $state("Проклятие Страда");
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
