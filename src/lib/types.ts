export type { Levers, PrimeLevel, ProfileId, VideoMode } from "../../agent/src/types";
import type { Levers, ProfileId } from "../../agent/src/types";

export type Overrides = Partial<Levers>;
export type AngleBackend = "d3d11" | "d3d11on12" | "gl" | "vulkan";

export type LocalePref = "auto" | "ru" | "en";
export type ThemePref = "auto" | "day" | "night";
export type AccentId = "peach" | "amber" | "sage" | "mint" | "azure" | "periwinkle" | "lavender" | "orchid" | "rose" | "steel";

interface EngineSettings {
	angle: AngleBackend;
	diskCacheMb: number;
	extraArgs: string;
}

export interface Settings {
	schema: number;
	profile: ProfileId;
	overrides: Overrides;
	engine: EngineSettings;
	locale: LocalePref;
	theme: ThemePref;
	accent: AccentId;
	/** Последний запущенный сервер; пишет только Rust. */
	lastServer: string | null;
}

export interface Server {
	id: string;
	name: string;
	url: string;
	profile: ProfileId | null;
	overrides: Overrides;
	/** Настоящий адрес Foundry, если вход через хостинг; узнаётся агентом. */
	gameUrl?: string | null;
}

interface FpsSummary {
	avg: number;
	low1: number;
	profile: ProfileId;
	at: number;
}

interface BenchResult extends FpsSummary {
	min: number;
}

interface ServerStats {
	lastSession: FpsSummary | null;
	lastBench: BenchResult | null;
	/** Средний FPS последних отчётов сеанса — мини-график на ЖК. */
	history?: number[];
}

interface Notice {
	key: string;
	params: Record<string, string>;
}

export interface StateDto {
	settings: Settings;
	servers: Server[];
	stats: Record<string, ServerStats>;
	presets: Record<ProfileId, Levers>;
	notice: Notice | null;
	locale: "ru" | "en";
	version: string;
}

export interface ProbeResult {
	reachable: boolean;
	foundry: boolean;
	active: boolean;
	version: string | null;
	world: string | null;
	system: string | null;
	users: number | null;
}

type Mode = "launcher" | "install" | "uninstall";
type InstallState = "fresh" | "upgrade" | "current";
export type InstallStep = "check" | "copy" | "shortcuts" | "register" | "done";

export interface InstallInfo {
	currentVersion: string;
	defaultDir: string;
	existingDir: string | null;
	existingVersion: string | null;
	state: InstallState;
}

export interface ModeDto {
	mode: Mode;
	install: InstallInfo | null;
}

export interface InstallProgress {
	step: InstallStep;
	pct: number;
}

export interface UpdateInfo {
	version: string;
	notes: string;
}
