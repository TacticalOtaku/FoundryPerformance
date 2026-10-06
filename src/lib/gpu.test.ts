import { describe, expect, it, vi } from "vitest";
import { gpuLevel, probeGpu } from "./gpu";

/** Холст, который открывает обычный и/или строгий (`failIfMajorPerformanceCaveat`) контекст. */
function fakeCanvas(plain: boolean, strict: boolean) {
	const lose = vi.fn();
	const make = vi.fn(() => ({
		getContext: (_id: "webgl2", attrs?: WebGLContextAttributes) =>
			(attrs?.failIfMajorPerformanceCaveat ? strict : plain) ? { getExtension: () => ({ loseContext: lose }) } : null
	}));
	return { make, lose };
}

describe("probeGpu", () => {
	it("hardware WebGL2: both contexts open and are released", () => {
		const { make, lose } = fakeCanvas(true, true);
		expect(probeGpu(make)).toEqual({ webgl2: true, hardware: true });
		expect(make).toHaveBeenCalledTimes(2);
		expect(lose).toHaveBeenCalledTimes(2);
	});

	it("software WebGL2: the strict context is refused", () => {
		expect(probeGpu(fakeCanvas(true, false).make)).toEqual({ webgl2: true, hardware: false });
	});

	it("no WebGL2: the strict probe is skipped", () => {
		const { make } = fakeCanvas(false, true);
		expect(probeGpu(make)).toEqual({ webgl2: false, hardware: false });
		expect(make).toHaveBeenCalledTimes(1);
	});

	it("a throwing getContext counts as missing", () => {
		const make = () => ({
			getContext: () => {
				throw new Error("blocked");
			}
		});
		expect(probeGpu(make)).toEqual({ webgl2: false, hardware: false });
	});
});

describe("gpuLevel", () => {
	it("maps the probe onto ok / software / none", () => {
		expect(gpuLevel({ webgl2: true, hardware: true })).toBe("ok");
		expect(gpuLevel({ webgl2: true, hardware: false })).toBe("software");
		expect(gpuLevel({ webgl2: false, hardware: false })).toBe("none");
	});
});
