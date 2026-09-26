export type { Levers, PrimeLevel, ProfileId, VideoMode } from "../../agent/src/types";
import type { Levers, ProfileId } from "../../agent/src/types";

export type Overrides = Partial<Levers>;
export type AngleBackend = "d3d11" | "d3d11on12" | "gl" | "vulkan";
export type LocalePref = "auto" | "ru" | "en";
export type ThemePref = "auto" | "day" | "night";
export type LedState = "ok" | "warn" | "err" | "off" | "pending";

export interface EngineSettings {
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

export interface FpsSummary {
	avg: number;
	low1: number;
	profile: ProfileId;
	at: number;
}

export interface BenchResult extends FpsSummary {
	min: number;
}

export interface ServerStats {
	lastSession: FpsSummary | null;
	lastBench: BenchResult | null;
}

export interface Notice {
	key: string;
	params: Record<string, string>;
}

export interface GpuInfo {
	name: string;
	vramMb: number;
	vendorId: number;
}

export interface StateDto {
	settings: Settings;
	servers: Server[];
	stats: Record<string, ServerStats>;
	presets: Record<ProfileId, Levers>;
	gpu: GpuInfo | null;
	recommended: ProfileId;
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

export type Mode = "launcher" | "install" | "uninstall";
export type InstallState = "fresh" | "upgrade" | "current";
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
