/** Что лаунчер знает о видеокарте — как в FLC, но честно: есть ли WebGL2 и не программный ли он. */
export interface GpuProbe {
	webgl2: boolean;
	hardware: boolean;
}

export type GpuLevel = "ok" | "software" | "none";

interface CanvasLike {
	getContext(id: "webgl2", attrs?: WebGLContextAttributes): unknown;
}

type Releasable = { getExtension(name: string): { loseContext(): void } | null };

function opens(make: () => CanvasLike, attrs?: WebGLContextAttributes): boolean {
	try {
		const gl = make().getContext("webgl2", attrs) as Releasable | null;
		if (!gl) return false;
		gl.getExtension("WEBGL_lose_context")?.loseContext();
		return true;
	} catch {
		return false;
	}
}

/**
 * Проба WebView самого лаунчера: флаги ANGLE и драйвер у него те же, что у игрового окна,
 * но холст Foundry здесь не проверяется. С `failIfMajorPerformanceCaveat` Chromium отказывает,
 * если WebGL2 есть только программный (SwiftShader, WARP).
 */
export function probeGpu(make: () => CanvasLike = () => document.createElement("canvas") as CanvasLike): GpuProbe {
	const webgl2 = opens(make);
	return { webgl2, hardware: webgl2 && opens(make, { failIfMajorPerformanceCaveat: true }) };
}

export const gpuLevel = (p: GpuProbe): GpuLevel => (!p.webgl2 ? "none" : p.hardware ? "ok" : "software");
