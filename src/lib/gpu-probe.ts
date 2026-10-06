/** Рендерер WebGL самого лаунчера: доступен ли GPU для WebView2 ещё до запуска игры. */
export function launcherRenderer(make: () => HTMLCanvasElement = () => document.createElement("canvas")): string | undefined {
	try {
		const c = make();
		const gl = (c.getContext("webgl2") ?? c.getContext("webgl")) as WebGLRenderingContext | null;
		if (!gl) return undefined;
		const ext = gl.getExtension("WEBGL_debug_renderer_info");
		const name: unknown = ext ? gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) : undefined;
		gl.getExtension("WEBGL_lose_context")?.loseContext();
		return typeof name === "string" && name ? name : undefined;
	} catch {
		return undefined;
	}
}
