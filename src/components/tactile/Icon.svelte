<script lang="ts" module>
	export type IconName = "app" | "chip" | "sliders" | "minimize" | "maximize" | "close" | "edit" | "plus" | "clock" | "enter" | "back" | "refresh" | "folder" | "trash";

	/** Тонкие линейные значки 16×16 в духе Phosphor Light — вне Foundry значков Font Awesome нет. */
	const PATHS: Record<IconName, string> = {
		app: "M8 2l5.2 3v6L8 14l-5.2-3V5zM5 10.5h6L8 5z",
		chip: "M5.5 4h5A1.5 1.5 0 0 1 12 5.5v5a1.5 1.5 0 0 1-1.5 1.5h-5A1.5 1.5 0 0 1 4 10.5v-5A1.5 1.5 0 0 1 5.5 4zM6 1.5v2M10 1.5v2M6 12.5v2M10 12.5v2M1.5 6h2M1.5 10h2M12.5 6h2M12.5 10h2",
		sliders: "M2.5 4.5h6M12 4.5h1.5M2.5 11.5h1.5M7.5 11.5h6M11.8 4.5a1.6 1.6 0 1 1-3.2 0a1.6 1.6 0 1 1 3.2 0M7.4 11.5a1.6 1.6 0 1 1-3.2 0a1.6 1.6 0 1 1 3.2 0",
		minimize: "M4 8h8",
		maximize: "M5.5 4.5h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1h-5a1 1 0 0 1-1-1v-5a1 1 0 0 1 1-1z",
		close: "M4.5 4.5l7 7M11.5 4.5l-7 7",
		edit: "M10.5 2.5l3 3L6 13H3v-3zM9 4l3 3",
		plus: "M8 3v10M3 8h10",
		clock: "M13.5 8a5.5 5.5 0 1 1-11 0a5.5 5.5 0 1 1 11 0M8 5v3.2l2 1.3",
		enter: "M12.5 3.5v4a2 2 0 0 1-2 2h-7M6 6.5l-3 3 3 3",
		back: "M10 3.5L5.5 8l4.5 4.5",
		refresh: "M12.8 8.6A4.9 4.9 0 1 1 11.3 4.2M12.2 1.8v3h-3",
		folder: "M2 4.5A1.5 1.5 0 0 1 3.5 3h2.8l1.5 1.5h4.7A1.5 1.5 0 0 1 14 6v5.5a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 2 11.5z",
		trash: "M3 4.5h10M6.5 4.5V3h3v1.5M4.5 4.5l.7 8.5h5.6l.7-8.5"
	};
</script>

<script lang="ts">
	let { name, size = 16 }: { name: IconName; size?: number } = $props();
</script>

<svg class="ic" viewBox="0 0 16 16" width={size} height={size} aria-hidden="true"><path d={PATHS[name]} /></svg>

<style>
	.ic {
		flex: none;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
</style>
