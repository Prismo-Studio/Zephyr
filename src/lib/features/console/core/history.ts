/**
 * Persisted command-history helpers. Both client and server sessions cap
 * history at 100 entries, most-recent first, with entries deduped on push.
 * localStorage failures (quota, private mode, disabled storage) are silently
 * ignored — history is a nicety, not critical state.
 */

const MAX_HISTORY = 100;

export function loadCommandHistory(key: string): string[] {
	try {
		const raw = localStorage.getItem(key);
		if (!raw) return [];
		const parsed = JSON.parse(raw);
		if (Array.isArray(parsed)) {
			return parsed.filter((x): x is string => typeof x === 'string');
		}
	} catch {
		// ignore
	}
	return [];
}

export function pushCommandHistory(key: string, history: string[], line: string): string[] {
	const next = [line, ...history.filter((h) => h !== line)].slice(0, MAX_HISTORY);
	try {
		localStorage.setItem(key, JSON.stringify(next));
	} catch {
		// ignore
	}
	return next;
}
