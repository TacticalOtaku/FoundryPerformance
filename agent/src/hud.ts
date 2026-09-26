import type { Strings } from "./i18n";
import type { ProfileId } from "./types";

export interface HudData {
	fps: number;
	low1: number;
	ms: number;
	res: number;
	/** null — базовый замер (профиль не применён). */
	profile: ProfileId | null;
	spark: number[];
}

const POS_KEY = "fp.hud.pos";
const PROFILES: ProfileId[] = ["quality", "balance", "potato"];

const CSS = `
:host{all:initial}
.hud{position:fixed;z-index:2147483000;width:184px;background:#1E2A1E;color:#B8F28A;border:2px solid #D8D6D0;
 font:500 11px/1.35 'Martian Mono',ui-monospace,Consolas,monospace;padding:8px 10px;user-select:none;cursor:grab;
 font-variant-numeric:tabular-nums}
.hud[hidden]{display:none}
.row{display:flex;justify-content:space-between;gap:8px}
.big{font-size:26px;line-height:1}
.dim{color:#7FA866}
canvas{display:block;width:100%;height:18px;margin:5px 0}
.menu{margin-top:8px;border-top:1px solid #3C5A3C;padding-top:6px;display:grid;gap:4px;cursor:default}
.menu[hidden]{display:none}
button{all:unset;cursor:pointer;padding:3px 6px;border:1px solid #3C5A3C;text-align:center}
button[aria-pressed=true]{background:#B8F28A;color:#1E2A1E}
button:focus-visible{outline:2px solid #FF5A1F}
.bench{border-color:#FF5A1F;color:#FF5A1F}
.status{color:#FF5A1F;min-height:1em;white-space:normal}
@media (prefers-reduced-motion:no-preference){button{transition:background .12s}}
`;

export class Hud {
	private host = document.createElement("fp-hud");
	private box: HTMLDivElement;
	private el: Record<"fps" | "low" | "res" | "ms" | "prof" | "status", HTMLElement>;
	private menu: HTMLDivElement;
	private spark: HTMLCanvasElement;
	private buttons = new Map<ProfileId, HTMLButtonElement>();

	constructor(
		private t: Strings,
		h: { onProfile(id: ProfileId): void; onBench(): void }
	) {
		const root = this.host.attachShadow({ mode: "closed" });
		root.innerHTML = `<style>${CSS}</style>
<div class="hud" hidden>
 <div class="row"><span class="big" data-k="fps">---</span><span>FPS</span></div>
 <canvas width="160" height="18"></canvas>
 <div class="row"><span>1%LOW <b data-k="low">--</b></span><span>RES <b data-k="res">--</b></span></div>
 <div class="row dim"><span><b data-k="ms">--</b> MS</span><span data-k="prof"></span></div>
 <div class="status" data-k="status"></div>
 <div class="menu" hidden><div class="dim">${t.menuTitle}</div></div>
</div>`;
		this.box = root.querySelector(".hud")!;
		this.menu = root.querySelector(".menu")!;
		this.spark = root.querySelector("canvas")!;
		const q = (k: string) => root.querySelector<HTMLElement>(`[data-k=${k}]`)!;
		this.el = { fps: q("fps"), low: q("low"), res: q("res"), ms: q("ms"), prof: q("prof"), status: q("status") };

		for (const id of PROFILES) {
			const b = document.createElement("button");
			b.textContent = t.profileLong[id];
			b.onclick = () => h.onProfile(id);
			this.buttons.set(id, b);
			this.menu.appendChild(b);
		}
		const bench = document.createElement("button");
		bench.className = "bench";
		bench.textContent = t.bench;
		bench.onclick = () => h.onBench();
		this.menu.appendChild(bench);
		const hint = document.createElement("div");
		hint.className = "dim";
		hint.textContent = t.hint;
		this.menu.appendChild(hint);

		this.restorePosition();
		this.enableDrag();
		(document.body ?? document.documentElement).appendChild(this.host);
	}

	toggle(): void {
		this.box.hidden = !this.box.hidden;
	}

	toggleMenu(): void {
		this.box.hidden = false;
		this.menu.hidden = !this.menu.hidden;
	}

	status(text: string): void {
		this.el.status.textContent = text;
		this.box.hidden = false;
	}

	update(d: HudData): void {
		if (this.box.hidden) return;
		this.el.fps.textContent = String(Math.round(d.fps)).padStart(3, "0");
		this.el.low.textContent = String(Math.round(d.low1));
		this.el.res.textContent = `${Math.round(d.res * 100)}%`;
		this.el.ms.textContent = d.ms.toFixed(1);
		this.el.prof.textContent = d.profile ? this.t.profile[d.profile] : this.t.base;
		for (const [id, b] of this.buttons) b.setAttribute("aria-pressed", String(id === d.profile));
		this.drawSpark(d.spark);
	}

	private drawSpark(fpsValues: number[]): void {
		const ctx = this.spark.getContext("2d");
		if (!ctx) return;
		const { width, height } = this.spark;
		ctx.clearRect(0, 0, width, height);
		const n = fpsValues.length;
		if (n === 0) return;
		const w = width / n;
		const top = Math.max(60, ...fpsValues);
		for (let i = 0; i < n; i++) {
			const v = fpsValues[i];
			const bh = Math.max(1, (v / top) * height);
			ctx.fillStyle = v < 30 ? "#FF5A1F" : "#B8F28A";
			ctx.fillRect(i * w, height - bh, Math.max(1, w - 1), bh);
		}
	}

	private restorePosition(): void {
		let pos = { right: 12, top: 12 };
		try {
			const raw = localStorage.getItem(POS_KEY);
			if (raw) pos = JSON.parse(raw);
		} catch {
			/* хранилище недоступно — позиция по умолчанию */
		}
		this.box.style.right = `${pos.right}px`;
		this.box.style.top = `${pos.top}px`;
	}

	private enableDrag(): void {
		this.box.addEventListener("pointerdown", (e) => {
			if ((e.target as HTMLElement).closest("button")) return;
			const startX = e.clientX;
			const startY = e.clientY;
			const startRight = parseFloat(this.box.style.right);
			const startTop = parseFloat(this.box.style.top);
			this.box.setPointerCapture(e.pointerId);
			const move = (ev: PointerEvent) => {
				this.box.style.right = `${Math.max(0, startRight - (ev.clientX - startX))}px`;
				this.box.style.top = `${Math.max(0, startTop + (ev.clientY - startY))}px`;
			};
			const up = () => {
				this.box.removeEventListener("pointermove", move);
				this.box.removeEventListener("pointerup", up);
				try {
					localStorage.setItem(POS_KEY, JSON.stringify({ right: parseFloat(this.box.style.right), top: parseFloat(this.box.style.top) }));
				} catch {
					/* не критично */
				}
			};
			this.box.addEventListener("pointermove", move);
			this.box.addEventListener("pointerup", up);
		});
	}
}
