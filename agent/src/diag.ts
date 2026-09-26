import { g } from "./foundry";
import { collectVideos } from "./levers/video";

export const AGENT_VERSION = "0.1.0";

function plain(v: unknown): unknown {
	try {
		return JSON.parse(JSON.stringify(v));
	} catch {
		return String(v);
	}
}

export function collectDiag(extra: Record<string, unknown>): Record<string, unknown> {
	const game = g.game;
	const canvas = g.canvas;
	const r = canvas?.app?.renderer;
	const settings: unknown[] = [];
	if (game?.settings?.settings) {
		for (const [key, cfg] of game.settings.settings) {
			if (cfg.scope === "world") continue;
			let value: unknown;
			try {
				value = game.settings.get(cfg.namespace, cfg.key);
			} catch {
				value = "<error>";
			}
			settings.push({
				key,
				scope: cfg.scope,
				type: cfg.type?.name ?? typeof cfg.default,
				default: plain(cfg.default),
				value: plain(value),
				choices: cfg.choices ? Object.keys(cfg.choices) : undefined,
				range: plain(cfg.range)
			});
		}
	}
	let gpu: string | undefined;
	try {
		const gl = r?.gl;
		const ext = gl?.getExtension("WEBGL_debug_renderer_info");
		gpu = ext ? gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) : undefined;
	} catch {
		gpu = undefined;
	}
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const modules = game?.modules ? [...game.modules].filter((m: any) => m.active).map((m: any) => ({ id: m.id, version: m.version })) : [];
	return {
		at: new Date().toISOString(),
		agent: AGENT_VERSION,
		foundry: game?.version ?? game?.release?.version,
		system: game?.system?.id,
		pixi: g.PIXI?.VERSION,
		devicePixelRatio: window.devicePixelRatio,
		renderer: { type: r?.type, resolution: r?.resolution, screen: [r?.screen?.width, r?.screen?.height], gpu },
		canvasPerformance: plain(canvas?.performance),
		hasVideoMeshes: Boolean(canvas?.primary?.videoMeshes),
		videoCount: collectVideos(canvas).length,
		modules,
		settings,
		...extra
	};
}

export function downloadDiag(data: Record<string, unknown>): void {
	const blob = new Blob([JSON.stringify(data, null, 2)], { type: "application/json" });
	const a = document.createElement("a");
	a.href = URL.createObjectURL(blob);
	a.download = `fp-diag-${Date.now()}.json`;
	document.body.appendChild(a);
	a.click();
	a.remove();
	setTimeout(() => URL.revokeObjectURL(a.href), 5000);
}
