// Собирает dist-release/latest.json — ленту автообновления — из подписанного exe.
// Запускается в GitHub Actions после `tauri signer sign`.
import crypto from "node:crypto";
import fs from "node:fs";
import { notesFor } from "../src/lib/changelog.ts";

const { version } = JSON.parse(fs.readFileSync("src-tauri/tauri.conf.json", "utf8"));
const repo = process.env.GITHUB_REPOSITORY ?? "TacticalOtaku/FoundryPerformance";
const tag = process.env.GITHUB_REF_NAME ?? `v${version}`;
const file = `FoundryPerformance-${version}.exe`;

const exe = fs.readFileSync(`dist-release/${file}`);
const signature = fs.readFileSync(`dist-release/${file}.sig`, "utf8").trim();
// Заметки — раздел версии из CHANGELOG.md; запасной вариант — текст аннотированного тега
const items = notesFor(fs.readFileSync("CHANGELOG.md", "utf8"), version);
const notes = items ? items.map((i) => `- ${i}`).join("\n") : (process.env.RELEASE_NOTES ?? "").trim();

const manifest = {
	version,
	notes,
	url: `https://github.com/${repo}/releases/download/${tag}/${file}`,
	sha256: crypto.createHash("sha256").update(exe).digest("hex"),
	signature
};

fs.writeFileSync("dist-release/latest.json", JSON.stringify(manifest, null, 2) + "\n");
fs.writeFileSync("dist-release/notes.md", notes || `Foundry Performance ${version}`);
console.log(`latest.json for ${version} → ${manifest.url}`);
