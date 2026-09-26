export interface Box {
	x: number;
	y: number;
	w: number;
	h: number;
}

export interface TipPlacement {
	x: number;
	y: number;
	side: "top" | "bottom";
	/** Смещение стрелки от левого края таблички — чтобы она указывала на элемент даже после прижатия к краю окна. */
	arrow: number;
}

const GAP = 8;
const MARGIN = 8;
const ARROW_INSET = 12;

export function placeTip(target: Box, tip: { w: number; h: number }, vp: { w: number; h: number }): TipPlacement {
	const above = target.y - GAP - tip.h;
	const side = above >= MARGIN ? "top" : "bottom";
	const y = side === "top" ? above : Math.min(target.y + target.h + GAP, vp.h - tip.h - MARGIN);
	const center = target.x + target.w / 2;
	const x = Math.min(Math.max(center - tip.w / 2, MARGIN), vp.w - tip.w - MARGIN);
	const arrow = Math.min(Math.max(center - x, ARROW_INSET), tip.w - ARROW_INSET);
	return { x, y, side, arrow };
}
