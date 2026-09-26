import type { InstallProgress, InstallStep, Levers, ModeDto, ProfileId, Server, Settings, StateDto } from "./types";

const progressListeners = new Set<(p: InstallProgress) => void>();

export function mockOnProgress(cb: (p: InstallProgress) => void): () => void {
	progressListeners.add(cb);
	return () => progressListeners.delete(cb);
}

/** Превью экранов установки: `?mode=install`, `?mode=install&state=upgrade|current`, `?mode=uninstall`. */
function mockMode(): ModeDto {
	const q = new URLSearchParams(location.search);
	const mode = (q.get("mode") ?? "launcher") as ModeDto["mode"];
	const state = (q.get("state") ?? "fresh") as "fresh" | "upgrade" | "current";
	return {
		mode,
		install:
			mode === "install"
				? {
						currentVersion: "0.1.0",
						defaultDir: "C:\\Users\\Niki\\AppData\\Local\\Programs\\FoundryPerformance",
						existingDir: state === "fresh" ? null : "C:\\Users\\Niki\\AppData\\Local\\Programs\\FoundryPerformance",
						existingVersion: state === "upgrade" ? "0.0.9" : state === "current" ? "0.1.0" : null,
						state
					}
				: null
	};
}

const lv = (o: Partial<Levers>): Levers => ({
	perfMode: 1,
	maxFps: 60,
	resMin: 0.7,
	resMax: 1,
	adaptive: true,
	pixelRatioScaling: false,
	lightAnimation: true,
	visionAnimation: false,
	mipmap: true,
	video: "pauseUnfocused",
	uiBlur: false,
	sequencer: true,
	fxmaster: true,
	unfocusedFps: 15,
	prime: "medium",
	...o
});

/** Копия пресетов из src-tauri/src/profile.rs — только для превью в браузере. */
const presets: Record<ProfileId, Levers> = {
	quality: lv({ perfMode: 2, resMin: 1, adaptive: false, pixelRatioScaling: true, visionAnimation: true, video: "play", uiBlur: true, unfocusedFps: 60, prime: "soft" }),
	balance: lv({}),
	potato: lv({ perfMode: 0, maxFps: 45, resMin: 0.55, resMax: 0.85, lightAnimation: false, mipmap: false, video: "static", sequencer: false, fxmaster: false, unfocusedFps: 10, prime: "aggressive" })
};

let settings: Settings = {
	schema: 1,
	profile: "balance",
	overrides: {},
	engine: { angle: "d3d11", diskCacheMb: 2048, extraArgs: "" },
	locale: "auto",
	theme: "auto"
};

let servers: Server[] = [
	{ id: "a1", name: "Проклятие Страда", url: "https://vtt.example.com/", profile: null, overrides: {} },
	{ id: "a3", name: "Aldarion", url: "https://www.sqyre.app/games/aldarionv210-3c11b9b6/", profile: null, overrides: {} },
	{ id: "a2", name: "Ваншот по пятницам", url: "http://192.168.1.40:30000/", profile: "quality", overrides: { maxFps: 45 } }
];

const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));

export async function mockInvoke(cmd: string, args: Record<string, unknown>): Promise<unknown> {
	await delay(120);
	switch (cmd) {
		case "get_state":
			return {
				settings,
				servers,
				stats: { a1: { lastSession: { avg: 52.4, low1: 31, profile: "balance", at: 0 }, lastBench: null } },
				presets,
				gpu: { name: "NVIDIA GeForce GTX 1060 6GB", vramMb: 6144, vendorId: 0x10de },
				recommended: "balance",
				notice: null,
				locale: "ru",
				version: "0.1.0-preview"
			} satisfies StateDto;
		case "save_server": {
			const s = args.server as Server;
			if (!s.name.trim()) throw "err.nameRequired";
			if (!/^(https?:\/\/)?[\w.-]+(:\d+)?(\/.*)?$/.test(s.url.trim())) throw "err.urlInvalid";
			servers = servers.some((x) => x.id === s.id) ? servers.map((x) => (x.id === s.id ? s : x)) : [...servers, s];
			return servers;
		}
		case "delete_server":
			servers = servers.filter((s) => s.id !== args.id);
			return servers;
		case "save_settings":
			settings = args.settings as Settings;
			return settings;
		case "probe_server":
			await delay(700);
			if (String(args.url).includes("sqyre.app/games")) return { reachable: true, foundry: false, active: false, version: null, world: null, system: null, users: null };
			return String(args.url).includes("192.168")
				? { reachable: false, foundry: false, active: false, version: null, world: null, system: null, users: null }
				: { reachable: true, foundry: true, active: true, version: "14.349", world: "strahd", system: "dnd5e", users: 3 };
		case "launch":
			console.info("[preview] launch", args);
			return null;
		case "clear_notice":
			return null;
		case "get_mode":
			return mockMode();
		case "pick_install_dir":
			return "D:\\Games\\FoundryPerformance";
		case "check_install_dir": {
			const dir = String(args.dir).trim();
			if (!/^[a-z]:\\/i.test(dir)) throw "install.err.notAbsolute";
			if (/^[a-z]:\\?$/i.test(dir)) throw "install.err.root";
			if (/^c:\\windows/i.test(dir)) throw "install.err.system";
			return null;
		}
		case "install": {
			const steps: InstallStep[] = ["check", "copy", "shortcuts", "register", "done"];
			const pcts = [5, 30, 60, 85, 100];
			for (let i = 0; i < steps.length; i++) {
				progressListeners.forEach((cb) => cb({ step: steps[i], pct: pcts[i] }));
				await delay(450);
			}
			return null;
		}
		case "open_installed":
			return null;
		case "uninstall":
			await delay(600);
			return null;
		default:
			throw `unknown command ${cmd}`;
	}
}
