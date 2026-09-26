// @vitest-environment happy-dom
import { describe, expect, it } from "vitest";
import { setUiBlur } from "./blur";

describe("setUiBlur", () => {
	it("injects and removes the no-blur stylesheet idempotently", () => {
		setUiBlur(document, false);
		setUiBlur(document, false);
		expect(document.querySelectorAll("#fp-no-blur")).toHaveLength(1);
		expect(document.getElementById("fp-no-blur")!.textContent).toContain("backdrop-filter:none");
		setUiBlur(document, true);
		expect(document.getElementById("fp-no-blur")).toBeNull();
	});
});
