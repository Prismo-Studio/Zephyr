const STORAGE_KEY = 'zephyr-mods-page-size';

export const PAGE_SIZE_CHOICES = [30, 50, 100, 500] as const;
export const DEFAULT_PAGE_SIZE = 30;

export function loadPageSize(): number {
	if (typeof localStorage === 'undefined') return DEFAULT_PAGE_SIZE;
	const raw = localStorage.getItem(STORAGE_KEY);
	const parsed = raw === null ? NaN : parseInt(raw, 10);
	return (PAGE_SIZE_CHOICES as readonly number[]).includes(parsed) ? parsed : DEFAULT_PAGE_SIZE;
}

export function persistPageSize(value: number): number {
	const valid = (PAGE_SIZE_CHOICES as readonly number[]).includes(value)
		? value
		: DEFAULT_PAGE_SIZE;
	try {
		localStorage.setItem(STORAGE_KEY, String(valid));
	} catch {
		/* localStorage unavailable (private mode, quota) — fall through */
	}
	return valid;
}
