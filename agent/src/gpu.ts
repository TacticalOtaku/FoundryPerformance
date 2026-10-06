import type { Strings } from "./i18n";
import type { Verdict } from "./types";

/** Настоящее имя рендерера WebGL; undefined — нет контекста, расширения или строки. */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function rendererName(renderer: any): string | undefined {
	try {
		const gl = renderer?.gl;
		const ext = gl?.getExtension("WEBGL_debug_renderer_info");
		const name: unknown = ext ? gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) : undefined;
		return typeof name === "string" && name ? name : undefined;
	} catch {
		return undefined;
	}
}

/** Тост в игре — только когда с ускорением беда. */
export function verdictMessage(v: Verdict | null, t: Strings): string | null {
	if (v?.kind === "software") return t.gpuSoftware;
	if (v?.kind === "wrongGpu") return t.gpuWrong;
	return null;
}
