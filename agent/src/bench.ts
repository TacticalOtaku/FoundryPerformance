import { summarize } from "./fps-meter";

/** 30 с панорамирования и зума по сцене; фиксирует времена кадров через rec. */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export async function runBench(canvas: any, rec: { start(): void; stop(): number[] }, durationMs = 30000) {
	const r = canvas.dimensions.sceneRect ?? canvas.dimensions.rect;
	const at = (fx: number, fy: number, scale: number) => ({ x: r.x + r.width * fx, y: r.y + r.height * fy, scale });
	const route = [at(0.5, 0.5, 0.5), at(0.2, 0.2, 1), at(0.8, 0.2, 1.5), at(0.8, 0.8, 1), at(0.2, 0.8, 0.75), at(0.5, 0.5, 1)];
	const segment = durationMs / route.length;
	rec.start();
	for (const p of route) await canvas.animatePan({ ...p, duration: segment });
	return summarize(rec.stop());
}
