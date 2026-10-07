import type { Action } from "svelte/action";

/** Кривая Tactile по умолчанию (AIM: CustomEase "tactile"). */
export const TACTILE = "cubic-bezier(0.32, 0.72, 0, 1)";
export const DUR = { micro: 160, short: 240, base: 380, long: 700 } as const;
const SAFETY_MS = 2200;

type Pt = [number, number];
/** Кривая AIM "settle": M0,0 C0.18,0.9 0.3,1.04 0.52,1.02 0.7,1 0.84,1 1,1 — перелёт и осадка. */
const SETTLE_PATH: [Pt, Pt, Pt, Pt][] = [
	[[0, 0], [0.18, 0.9], [0.3, 1.04], [0.52, 1.02]],
	[[0.52, 1.02], [0.7, 1], [0.84, 1], [1, 1]]
];
const bez = (a: number, b: number, c: number, d: number, t: number) => {
	const u = 1 - t;
	return u * u * u * a + 3 * u * u * t * b + 3 * u * t * t * c + t * t * t * d;
};

/** Точки кривой settle для CSS `linear()`: Web Animations не умеют SVG-пути. */
export function settlePoints(perSegment = 12): Pt[] {
	const out: Pt[] = [];
	for (const [p0, p1, p2, p3] of SETTLE_PATH) {
		for (let i = out.length ? 1 : 0; i <= perSegment; i++) {
			const t = i / perSegment;
			out.push([bez(p0[0], p1[0], p2[0], p3[0], t), bez(p0[1], p1[1], p2[1], p3[1], t)]);
		}
	}
	return out;
}

export const SETTLE = `linear(${settlePoints()
	.map(([x, y]) => `${+y.toFixed(4)} ${+(x * 100).toFixed(2)}%`)
	.join(", ")})`;

export const reduceMotion = (): boolean => typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;

const fadeIn = (el: HTMLElement) => el.animate([{ opacity: 0 }, { opacity: 1 }], { duration: 150, easing: "linear" });

/**
 * Появление экрана: корень оседает из 0.965 и 10 px снизу, зоны `[data-part]` всплывают с шагом 40 ms.
 * Страховка 2,2 s: в скрытом окне кадры не идут, и без неё экран остался бы прозрачным.
 */
export const openWindow: Action<HTMLElement> = (root) => {
	const anims: Animation[] = [];
	if (reduceMotion()) anims.push(fadeIn(root));
	else {
		anims.push(
			root.animate([{ opacity: 0, transform: "translateY(10px) scale(0.965)" }, { opacity: 1, transform: "none" }], { duration: DUR.long, easing: SETTLE })
		);
		root.querySelectorAll<HTMLElement>("[data-part]").forEach((el, i) => {
			anims.push(
				el.animate([{ opacity: 0, transform: "translateY(10px)" }, { opacity: 1, transform: "none" }], {
					duration: 450,
					delay: 120 + i * 40,
					easing: TACTILE,
					fill: "backwards"
				})
			);
		});
	}
	const safety = setTimeout(() => anims.forEach((a) => a.finish()), SAFETY_MS);
	return {
		destroy() {
			clearTimeout(safety);
			anims.forEach((a) => a.cancel());
		}
	};
};

/** Нажатие: кнопка уходит на 1 px и 0.96, отпускание возвращает её с осадкой. Клик не ждёт анимацию. */
export const press: Action<HTMLElement> = (el) => {
	let anim: Animation | undefined;
	const pressed = "translateY(1px) scale(0.96)";
	const down = () => {
		if (reduceMotion() || ("disabled" in el && el.disabled)) return;
		anim?.cancel();
		anim = el.animate([{ transform: "none" }, { transform: pressed }], { duration: 120, easing: "ease-out", fill: "forwards" });
	};
	const up = () => {
		if (!anim) return;
		anim.cancel();
		anim = el.animate([{ transform: pressed }, { transform: "none" }], { duration: 450, easing: SETTLE });
		anim.onfinish = () => (anim = undefined);
	};
	el.addEventListener("pointerdown", down);
	for (const e of ["pointerup", "pointerleave", "pointercancel"]) el.addEventListener(e, up);
	return {
		destroy() {
			anim?.cancel();
			el.removeEventListener("pointerdown", down);
			for (const e of ["pointerup", "pointerleave", "pointercancel"]) el.removeEventListener(e, up);
		}
	};
};

/** Смена ключа (например, выбранного сервера): содержимое всплывает на 6 px. */
export const rise: Action<HTMLElement, unknown> = (el, key) => {
	let last = key;
	let anim: Animation | undefined;
	return {
		update(next) {
			if (next === last) return;
			last = next;
			anim?.cancel();
			anim = reduceMotion()
				? fadeIn(el)
				: el.animate([{ opacity: 0, transform: "translateY(6px)" }, { opacity: 1, transform: "none" }], { duration: DUR.base, easing: TACTILE });
		},
		destroy() {
			anim?.cancel();
		}
	};
};

/** Число досчитывается от старого значения за 700 ms; элемент без детей — текст пишет сам action. */
export const countTo: Action<HTMLElement, number> = (el, value) => {
	let shown = value;
	let raf = 0;
	let safety: ReturnType<typeof setTimeout> | undefined;
	const set = (v: number) => (el.textContent = String(Math.round(v)));
	set(value);
	const stop = () => {
		cancelAnimationFrame(raf);
		clearTimeout(safety);
	};
	return {
		update(next) {
			stop();
			const from = shown;
			shown = next;
			if (from === next || reduceMotion()) {
				set(next);
				return;
			}
			const t0 = performance.now();
			const step = (now: number) => {
				const k = Math.min(1, (now - t0) / DUR.long);
				set(from + (next - from) * (1 - (1 - k) ** 3));
				if (k < 1) raf = requestAnimationFrame(step);
			};
			raf = requestAnimationFrame(step);
			// в скрытом окне кадров нет — число всё равно встаёт на место
			safety = setTimeout(() => {
				cancelAnimationFrame(raf);
				set(next);
			}, DUR.long + 100);
		},
		destroy: stop
	};
};
