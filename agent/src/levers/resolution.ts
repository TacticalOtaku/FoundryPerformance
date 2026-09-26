/**
 * Меняет разрешение рендерера PIXI (только канвас; DOM-интерфейс Foundry не затрагивается)
 * и шлёт `resize`, чтобы Foundry пересчитал свои render-текстуры штатным путём.
 * Работоспособность на v14 проверяется в Task 10.
 */
export class CanvasResolution {
	private current = 1;

	constructor(
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		private getRenderer: () => any,
		private baseResolution: () => number,
		private win: { dispatchEvent(e: Event): boolean }
	) {}

	get scale(): number {
		return this.current;
	}

	targetFor(scale: number): number {
		return Math.max(0.25, Math.round(this.baseResolution() * scale * 100) / 100);
	}

	apply(scale: number): boolean {
		const r = this.getRenderer();
		if (!r) return false;
		this.current = scale;
		const target = this.targetFor(scale);
		if (Math.abs(r.resolution - target) < 0.005) return true;
		r.resolution = target;
		this.win.dispatchEvent(new Event("resize"));
		return true;
	}
}
