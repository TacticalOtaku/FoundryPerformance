import { describe, expect, it, vi } from "vitest";
import { launcherRenderer } from "./gpu-probe";

const UNMASKED = 0x9246;
function fakeGl(name: unknown) {
	const loseContext = vi.fn();
	const gl = {
		getExtension: (n: string) =>
			n === "WEBGL_debug_renderer_info" ? { UNMASKED_RENDERER_WEBGL: UNMASKED } : n === "WEBGL_lose_context" ? { loseContext } : null,
		getParameter: (p: number) => (p === UNMASKED ? name : undefined)
	};
	return { gl, loseContext };
}
const canvas = (ctx: Record<string, unknown>) => () => ({ getContext: (k: string) => ctx[k] ?? null }) as unknown as HTMLCanvasElement;

describe("launcherRenderer", () => {
	it("reads the renderer and releases the context", () => {
		const { gl, loseContext } = fakeGl("ANGLE (NVIDIA, x)");
		expect(launcherRenderer(canvas({ webgl2: gl }))).toBe("ANGLE (NVIDIA, x)");
		expect(loseContext).toHaveBeenCalled();
	});

	it("falls back to WebGL 1", () => {
		expect(launcherRenderer(canvas({ webgl: fakeGl("r").gl }))).toBe("r");
	});

	it("is undefined without a context or a renderer string, and never throws", () => {
		expect(launcherRenderer(canvas({}))).toBeUndefined();
		expect(launcherRenderer(canvas({ webgl2: fakeGl("").gl }))).toBeUndefined();
		expect(
			launcherRenderer(() => {
				throw new Error("no canvas");
			})
		).toBeUndefined();
	});
});
