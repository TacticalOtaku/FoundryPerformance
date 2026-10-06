/**
 * Следующий выбор в радиогруппе по клавише. `enabled[i] === false` — пункт пропускается
 * стрелками (например, «Ручной»: его выбирают только мышью). `null` — клавиша не наша.
 */
export function nextIndex(current: number, key: string, enabled: boolean[]): number | null {
	const n = enabled.length;
	if (!enabled.some(Boolean)) return null;
	if (key === "Home") return enabled.indexOf(true);
	if (key === "End") return enabled.lastIndexOf(true);
	const step = key === "ArrowRight" || key === "ArrowDown" ? 1 : key === "ArrowLeft" || key === "ArrowUp" ? -1 : 0;
	if (step === 0) return null;
	let i = current < 0 ? (step > 0 ? -1 : n) : current;
	for (let k = 0; k < n; k++) {
		i = (i + step + n) % n;
		if (enabled[i]) return i;
	}
	return null;
}
