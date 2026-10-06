import type { Levers, Overrides, ProfileId, Server, StateDto } from "./types";

export type Scope = { kind: "global" } | { kind: "server"; id: string };

function applyOverrides(base: Levers, o: Overrides): Levers {
	const defined = Object.entries(o).filter(([, v]) => v !== undefined && v !== null);
	return { ...base, ...Object.fromEntries(defined) } as Levers;
}

function serverOf(dto: StateDto, scope: Scope): Server | null {
	return scope.kind === "server" ? (dto.servers.find((s) => s.id === scope.id) ?? null) : null;
}

export function profileFor(dto: StateDto, scope: Scope): ProfileId {
	return serverOf(dto, scope)?.profile ?? dto.settings.profile;
}

/** То, от чего считаются правки текущего уровня: для сервера это уже профиль + глобальные правки. */
export function baseFor(dto: StateDto, scope: Scope): Levers {
	const preset = dto.presets[profileFor(dto, scope)];
	return scope.kind === "global" ? preset : applyOverrides(preset, dto.settings.overrides);
}

export function overridesFor(dto: StateDto, scope: Scope): Overrides {
	return scope.kind === "global" ? dto.settings.overrides : (serverOf(dto, scope)?.overrides ?? {});
}

export function resolved(dto: StateDto, scope: Scope): Levers {
	return applyOverrides(baseFor(dto, scope), overridesFor(dto, scope));
}

const same = (a: unknown, b: unknown) =>
	typeof a === "number" && typeof b === "number" ? Math.abs(a - b) < 1e-6 : a === b;

export function withOverrides(o: Overrides, base: Levers, patch: Partial<Levers>): Overrides {
	const next: Record<string, unknown> = { ...o };
	for (const [k, v] of Object.entries(patch)) {
		if (same(base[k as keyof Levers], v)) delete next[k];
		else next[k] = v;
	}
	return next as Overrides;
}

export function countOverrides(o: Overrides): number {
	return Object.values(o).filter((v) => v !== undefined && v !== null).length;
}

export type KnobPosition = ProfileId | "manual";

/** Положение ручки: пресет или «РУЧ», если на сервер действуют ручные правки (общие или его собственные). */
export function knobPosition(dto: StateDto, serverId: string | null): KnobPosition {
	const scope: Scope = serverId ? { kind: "server", id: serverId } : { kind: "global" };
	const server = serverOf(dto, scope);
	const manual = countOverrides(dto.settings.overrides) + countOverrides(server?.overrides ?? {}) > 0;
	return manual ? "manual" : profileFor(dto, scope);
}
