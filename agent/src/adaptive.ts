import type { Levers } from "./types";

export interface AdaptiveConfig {
	min: number;
	max: number;
	step: number;
	targetMs: number;
	/** Кадр «медленный», если avg > targetMs × overRatio. */
	overRatio: number;
	/** Кадр «стабильный», если avg ≤ targetMs × stableRatio. */
	stableRatio: number;
	downAfterMs: number;
	upAfterMs: number;
	cooldownMs: number;
	/** На сколько блокируется масштаб, который не выдержал пробного повышения. */
	probeBackoffMs: number;
}

export interface AdaptiveState {
	scale: number;
	overSince: number | null;
	stableSince: number | null;
	lastChange: number;
	lastUpAt: number;
	blockedScale: number | null;
	blockedUntil: number;
}

export function configFor(l: Levers): AdaptiveConfig {
	return {
		min: l.resMin,
		max: l.resMax,
		step: 0.05,
		targetMs: 1000 / l.maxFps,
		overRatio: 1.15,
		stableRatio: 1.05,
		downAfterMs: 2000,
		upAfterMs: 5000,
		cooldownMs: 1000,
		probeBackoffMs: 30000
	};
}

export function initAdaptive(c: AdaptiveConfig): AdaptiveState {
	return {
		scale: c.max,
		overSince: null,
		stableSince: null,
		lastChange: -Infinity,
		lastUpAt: -Infinity,
		blockedScale: null,
		blockedUntil: 0
	};
}

export function resetTimers(s: AdaptiveState): AdaptiveState {
	return { ...s, overSince: null, stableSince: null };
}

const round2 = (x: number) => Math.round(x * 100) / 100;

/**
 * Когда FPS упирается в лимит, время кадра не может стать меньше цели,
 * поэтому повышение идёт «пробой»: после 5 с стабильности пробуем шаг вверх;
 * если он сразу проваливается — этот масштаб блокируется на probeBackoffMs.
 */
export function stepAdaptive(s: AdaptiveState, c: AdaptiveConfig, avgMs: number, now: number): AdaptiveState {
	const over = avgMs > c.targetMs * c.overRatio;
	const stable = avgMs <= c.targetMs * c.stableRatio;
	const cooled = now - s.lastChange >= c.cooldownMs;

	if (over) {
		const overSince = s.overSince ?? now;
		if (now - overSince >= c.downAfterMs && cooled && s.scale > c.min) {
			const failedProbe = now - s.lastUpAt <= c.downAfterMs * 2;
			return {
				...s,
				scale: round2(Math.max(c.min, s.scale - c.step)),
				overSince: null,
				stableSince: null,
				lastChange: now,
				blockedScale: failedProbe ? s.scale : s.blockedScale,
				blockedUntil: failedProbe ? now + c.probeBackoffMs : s.blockedUntil
			};
		}
		return { ...s, overSince, stableSince: null };
	}

	if (stable) {
		const stableSince = s.stableSince ?? now;
		const next = round2(Math.min(c.max, s.scale + c.step));
		const blocked = s.blockedScale !== null && next >= s.blockedScale && now < s.blockedUntil;
		if (now - stableSince >= c.upAfterMs && cooled && s.scale < c.max && !blocked) {
			return { ...s, scale: next, overSince: null, stableSince: null, lastChange: now, lastUpAt: now };
		}
		return { ...s, stableSince, overSince: null };
	}

	return resetTimers(s);
}
