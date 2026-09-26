export class FrameStats {
	private buf: Float64Array;
	private next = 0;
	private count = 0;

	constructor(capacity: number) {
		this.buf = new Float64Array(capacity);
	}

	push(ms: number): void {
		this.buf[this.next] = ms;
		this.next = (this.next + 1) % this.buf.length;
		if (this.count < this.buf.length) this.count++;
	}

	get size(): number {
		return this.count;
	}

	recent(count: number): number[] {
		const n = Math.min(count, this.count);
		const out = new Array<number>(n);
		for (let i = 0; i < n; i++) {
			const idx = (this.next - n + i + this.buf.length) % this.buf.length;
			out[i] = this.buf[idx];
		}
		return out;
	}

	avgMs(count: number): number {
		const r = this.recent(count);
		if (r.length === 0) return 0;
		let sum = 0;
		for (const x of r) sum += x;
		return sum / r.length;
	}

	clear(): void {
		this.next = 0;
		this.count = 0;
	}
}

/** FPS по самым медленным 1% кадров. */
export function low1(samplesMs: number[]): number {
	if (samplesMs.length === 0) return 0;
	const sorted = [...samplesMs].sort((a, b) => b - a);
	const k = Math.max(1, Math.floor(sorted.length * 0.01));
	let sum = 0;
	for (let i = 0; i < k; i++) sum += sorted[i];
	return 1000 / (sum / k);
}

export function summarize(samplesMs: number[]): { avg: number; low1: number; min: number } {
	if (samplesMs.length === 0) return { avg: 0, low1: 0, min: 0 };
	let sum = 0;
	let worst = 0;
	for (const x of samplesMs) {
		sum += x;
		if (x > worst) worst = x;
	}
	return { avg: 1000 / (sum / samplesMs.length), low1: low1(samplesMs), min: 1000 / worst };
}
