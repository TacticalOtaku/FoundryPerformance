// npm run version:set 0.2.0 — одна версия во всех местах, где она записана.
import fs from "node:fs";

const version = process.argv[2];
if (!/^\d+\.\d+\.\d+$/.test(version ?? "")) {
	console.error("usage: npm run version:set 1.2.3");
	process.exit(1);
}

const editJson = (file, edit) => {
	const data = JSON.parse(fs.readFileSync(file, "utf8"));
	edit(data);
	fs.writeFileSync(file, JSON.stringify(data, null, 2) + "\n");
};
const editText = (file, pattern, replacement) => {
	const text = fs.readFileSync(file, "utf8");
	if (!pattern.test(text)) throw new Error(`version pattern not found in ${file}`);
	fs.writeFileSync(file, text.replace(pattern, replacement));
};

editJson("package.json", (p) => (p.version = version));
editJson("src-tauri/tauri.conf.json", (c) => (c.version = version));
editText("src-tauri/Cargo.toml", /^version = ".*"$/m, `version = "${version}"`);
editText("src-tauri/Cargo.lock", /(name = "foundry-performance"\r?\nversion = )".*"/, `$1"${version}"`);
editText("agent/src/diag.ts", /AGENT_VERSION = ".*";/, `AGENT_VERSION = "${version}";`);

console.log(`version set to ${version} — now commit, tag v${version} and push the tag`);
