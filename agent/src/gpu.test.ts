import { describe, expect, it } from "vitest";
import { rendererName, verdictMessage } from "./gpu";
import { strings } from "./i18n";

const UNMASKED = 0x9246;
const fake = (name: unknown, withExt = true) => ({
	gl: {
		getExtension: (n: string) => (withExt && n === "WEBGL_debug_renderer_info" ? { UNMASKED_RENDERER_WEBGL: UNMASKED } : null),
		getParameter: (p: number) => (p === UNMASKED ? name : undefined)
	}
});

describe("rendererName", () => {
	it("reads the unmasked renderer string", () => {
		expect(rendererName(fake("ANGLE (NVIDIA, x)"))).toBe("ANGLE (NVIDIA, x)");
	});

	it("is undefined without a context, the extension or a string", () => {
		expect(rendererName(undefined)).toBeUndefined();
		expect(rendererName(fake("x", false))).toBeUndefined();
		expect(rendererName(fake(""))).toBeUndefined();
		expect(
			rendererName({
				gl: {
					getExtension: () => {
						throw new Error("context lost");
					}
				}
			})
		).toBeUndefined();
	});
});

describe("verdictMessage", () => {
	const t = strings("ru");

	it("warns only about software render and the wrong GPU", () => {
		expect(verdictMessage({ kind: "software" }, t)).toBe(t.gpuSoftware);
		expect(verdictMessage({ kind: "wrongGpu", backend: "d3d11" }, t)).toBe(t.gpuWrong);
		expect(verdictMessage({ kind: "hardware", backend: "d3d11" }, t)).toBeNull();
		expect(verdictMessage({ kind: "unknown" }, t)).toBeNull();
		expect(verdictMessage(null, t)).toBeNull();
	});
});
