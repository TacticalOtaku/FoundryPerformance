import { describe, expect, it } from "vitest";
import { formatBytes } from "./bytes";

const units = { mb: "МБ", gb: "ГБ" };

describe("formatBytes", () => {
	it("shows megabytes below a gigabyte", () => {
		expect(formatBytes(0, units, "ru")).toBe("0 МБ");
		expect(formatBytes(91_000_000, units, "ru")).toBe("87 МБ");
	});

	it("shows gigabytes with one decimal in the locale's format", () => {
		expect(formatBytes(1.3 * 1024 ** 3, units, "ru")).toBe("1,3 ГБ");
		expect(formatBytes(1.3 * 1024 ** 3, { mb: "MB", gb: "GB" }, "en")).toBe("1.3 GB");
		expect(formatBytes(2 * 1024 ** 3, units, "ru")).toBe("2 ГБ");
	});
});
