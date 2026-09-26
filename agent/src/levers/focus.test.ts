import { describe, expect, it } from "vitest";
import { FocusThrottle } from "./focus";

describe("FocusThrottle", () => {
	it("switches ticker maxFPS on focus changes", () => {
		const ticker = { maxFPS: 0 };
		const f = new FocusThrottle(() => ticker, 60, 15);
		f.apply();
		expect(ticker.maxFPS).toBe(60);
		f.setFocused(false);
		expect(ticker.maxFPS).toBe(15);
		f.setFps(45, 10);
		expect(ticker.maxFPS).toBe(10);
		f.setFocused(true);
		expect(ticker.maxFPS).toBe(45);
	});

	it("tolerates a missing ticker", () => {
		const f = new FocusThrottle(() => undefined, 60, 15);
		expect(() => f.setFocused(false)).not.toThrow();
	});
});
