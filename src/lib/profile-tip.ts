import type { KnobPosition } from "./levers";

/** Порядок сегментов профиля на главном экране. */
export const PROFILE_POSITIONS: readonly KnobPosition[] = ["quality", "balance", "potato", "manual"];

export const profileNameKey = (p: KnobPosition): string => `profile.${p}.long`;
export const profileTipKey = (p: KnobPosition): string => `tip.${p}`;
