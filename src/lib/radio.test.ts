import { describe, expect, it } from "vitest";
import { nextIndex } from "./radio";

const all = [true, true, true, true];
const noManual = [true, true, true, false];

describe("nextIndex", () => {
	it("moves with arrows and wraps", () => {
		expect(nextIndex(0, "ArrowRight", all)).toBe(1);
		expect(nextIndex(3, "ArrowRight", all)).toBe(0);
		expect(nextIndex(0, "ArrowLeft", all)).toBe(3);
		expect(nextIndex(1, "ArrowDown", all)).toBe(2);
		expect(nextIndex(1, "ArrowUp", all)).toBe(0);
	});

	it("skips disabled items", () => {
		expect(nextIndex(2, "ArrowRight", noManual)).toBe(0);
		expect(nextIndex(0, "ArrowLeft", noManual)).toBe(2);
		expect(nextIndex(3, "ArrowRight", noManual)).toBe(0);
	});

	it("Home and End go to the first and last enabled item", () => {
		expect(nextIndex(2, "Home", noManual)).toBe(0);
		expect(nextIndex(0, "End", noManual)).toBe(2);
	});

	it("ignores other keys and empty groups", () => {
		expect(nextIndex(0, "Enter", all)).toBeNull();
		expect(nextIndex(0, "ArrowRight", [false, false])).toBeNull();
	});

	it("starts from the edge when nothing is selected", () => {
		expect(nextIndex(-1, "ArrowRight", all)).toBe(0);
		expect(nextIndex(-1, "ArrowLeft", all)).toBe(3);
	});
});
