/// <reference types="node" />
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import ru from "./ru.json";

function sources(dir: string): string[] {
	return readdirSync(dir).flatMap((f) => {
		const p = join(dir, f);
		if (statSync(p).isDirectory()) return sources(p);
		return /\.(svelte|ts)$/.test(f) && !f.endsWith(".test.ts") ? [p] : [];
	});
}

const code = sources("src").map((f) => readFileSync(f, "utf8")).join("\n");
const keys = Object.keys(ru);
/** Ключи, которые код собирает из частей (`t(\`profile.${p}.long\`)`) или присылает Rust (`notice.*`, `err.*`). */
const DYNAMIC = [
	/^profile\./,
	/^status\./,
	/^led\./,
	/^tip\./,
	/^accent\./,
	/^video\./,
	/^prime\./,
	/^theme\./,
	/^main\.reason\./,
	/^install\.step\./,
	/^update\.state\./,
	/^err\./,
	/^notice\./,
	/^cache\.err\./,
	/^install\.err\./,
	/^uninstall\.err\./,
	/^update\.err\./
];

describe("i18n usage", () => {
	it("every literal t() key exists", () => {
		const used = [...code.matchAll(/\bt\(\s*"([\w.]+)"/g)].map((m) => m[1]);
		expect(used.filter((k) => !keys.includes(k))).toEqual([]);
	});

	it("every static key is used", () => {
		const unused = keys.filter((k) => !DYNAMIC.some((r) => r.test(k)) && !code.includes(`"${k}"`));
		expect(unused).toEqual([]);
	});
});
