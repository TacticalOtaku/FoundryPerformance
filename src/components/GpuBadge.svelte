<script lang="ts">
	import { gpuLevel } from "../lib/gpu";
	import { t } from "../lib/i18n.svelte";
	import { app } from "../lib/store.svelte";
	import Icon from "./tactile/Icon.svelte";
	import IconButton from "./tactile/IconButton.svelte";

	const probe = $derived(app.gpu);
	const level = $derived(probe ? gpuLevel(probe) : null);
	const checks = $derived(
		probe
			? [
					{ ok: probe.webgl2, text: t(probe.webgl2 ? "gpu.webgl2.yes" : "gpu.webgl2.no") },
					{ ok: probe.hardware, text: t(probe.hardware ? "gpu.hw.yes" : "gpu.hw.no") }
				]
			: []
	);
	const color = $derived(level === "ok" ? "var(--tc-good)" : level === "software" ? "var(--tc-warning)" : "var(--tc-danger)");
	const advice = $derived(level === "software" ? t("gpu.advice.software") : level === "none" ? t("gpu.advice.none") : "");
</script>

{#if probe}
	<IconButton
		size={30}
		label={`${t("gpu.title")}: ${checks.map((c) => c.text).join(", ")}`}
		style={`color: ${color}`}
		tip={{ title: t("gpu.title"), body: advice, checks }}
	>
		<Icon name="chip" />
	</IconButton>
{/if}
