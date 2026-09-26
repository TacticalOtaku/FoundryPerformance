import type { Levers, ProfileId, Server, Settings, StateDto } from "./types";

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
			return String(args.url).includes("192.168")
				? { reachable: false, foundry: false, active: false, version: null, world: null, system: null, users: null }
				: { reachable: true, foundry: true, active: true, version: "14.349", world: "strahd", system: "dnd5e", users: 3 };
		case "launch":
			console.info("[preview] launch", args);
			return null;
		case "clear_notice":
			return null;
		default:
			throw `unknown command ${cmd}`;
	}
}
