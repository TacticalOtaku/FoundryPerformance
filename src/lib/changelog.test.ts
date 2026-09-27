import { describe, expect, it } from "vitest";
import { noteLines, notesFor } from "./changelog";

const MD = `# Что нового

## 0.2.1
- Max FPS до 240
- Положение «Ручной»
  на ручке пульта

## 0.2.0 — первый релиз с автообновлением
* Статус сервера в кружке
`;

describe("notesFor", () => {
	it("returns the items of the version's section", () => {
		expect(notesFor(MD, "0.2.1")).toEqual(["Max FPS до 240", "Положение «Ручной» на ручке пульта"]);
	});

	it("accepts a heading with trailing text and * bullets", () => {
		expect(notesFor(MD, "0.2.0")).toEqual(["Статус сервера в кружке"]);
	});

	it("does not match a version by prefix", () => {
		expect(notesFor(MD, "0.2")).toBeNull();
		expect(notesFor("## 0.2.10\n- x", "0.2.1")).toBeNull();
	});

	it("tolerates CRLF", () => {
		expect(notesFor(MD.replaceAll("\n", "\r\n"), "0.2.1")).toHaveLength(2);
	});

	it("returns null for a missing or empty section", () => {
		expect(notesFor(MD, "9.9.9")).toBeNull();
		expect(notesFor("## 1.0.0\n\n## 0.9.0\n- x", "1.0.0")).toBeNull();
	});
});

describe("noteLines", () => {
	it("splits feed notes into items", () => {
		expect(noteLines("- один\n- два\n\n")).toEqual(["один", "два"]);
		expect(noteLines("просто текст")).toEqual(["просто текст"]);
		expect(noteLines("")).toEqual([]);
	});
});
