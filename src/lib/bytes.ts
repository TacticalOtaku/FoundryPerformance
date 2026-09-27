/** Размер для людей: «87 МБ», «1,3 ГБ». Единицы — из словаря, чтобы совпадали с остальным интерфейсом. */
export function formatBytes(n: number, units: { mb: string; gb: string }, locale: string): string {
	const mb = n / 1024 ** 2;
	if (mb < 1024) return `${Math.round(mb)} ${units.mb}`;
	const gb = (mb / 1024).toLocaleString(locale, { maximumFractionDigits: 1 });
	return `${gb} ${units.gb}`;
}
