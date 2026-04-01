export type ThemeId = 'dark' | 'light' | 'zephyr' | 'midnight' | 'hotdog';

export const themes: { id: ThemeId; label: string; hidden?: boolean }[] = [
	{ id: 'dark', label: 'Dark' },
	{ id: 'light', label: 'Light' },
	{ id: 'zephyr', label: 'Zephyr' },
	{ id: 'midnight', label: 'Midnight Ocean' },
	{ id: 'hotdog', label: 'Hot Dog Stand', hidden: true }
];

const HOTDOG_KEY = 'zephyr-hotdog-unlocked';
const HOTDOG_SEQUENCE = 'hotdogstand';

// Wingdings easter egg — session-only (cleared on app restart)
const WINGDINGS_SEQUENCE = 'wingdings';

let buffer = '';
let listener: ((e: KeyboardEvent) => void) | null = null;

export function isHotdogUnlocked(): boolean {
	return localStorage.getItem(HOTDOG_KEY) === 'true';
}

export function unlockHotdog() {
	localStorage.setItem(HOTDOG_KEY, 'true');
}

// Wingdings: session-only, not stored in localStorage
export function isWingdingsUnlocked(): boolean {
	return sessionStorage.getItem('zephyr-wingdings-unlocked') === 'true';
}

export function unlockWingdings() {
	sessionStorage.setItem('zephyr-wingdings-unlocked', 'true');
}

export function getVisibleThemes() {
	return themes.filter((t) => !t.hidden || (t.id === 'hotdog' && isHotdogUnlocked()));
}

export function initEasterEgg() {
	if (listener) return;
	const maxLen = Math.max(HOTDOG_SEQUENCE.length, WINGDINGS_SEQUENCE.length);
	listener = (e: KeyboardEvent) => {
		if (e.key === ' ') return;
		buffer += e.key.toLowerCase();
		if (buffer.length > maxLen) {
			buffer = buffer.slice(-maxLen);
		}

		// Hotdog theme unlock
		if (buffer.endsWith(HOTDOG_SEQUENCE) && !isHotdogUnlocked()) {
			unlockHotdog();
			setTheme('hotdog');
			window.dispatchEvent(new CustomEvent('hotdog-unlocked'));
		}

		// Wingdings font unlock (session-only)
		if (buffer.endsWith(WINGDINGS_SEQUENCE) && !isWingdingsUnlocked()) {
			unlockWingdings();
			window.dispatchEvent(new CustomEvent('wingdings-unlocked'));
		}
	};
	window.addEventListener('keydown', listener);
}

export function setTheme(theme: ThemeId) {
	document.documentElement.setAttribute('data-theme', theme);
	localStorage.setItem('zephyr-theme', theme);
}

export function getTheme(): ThemeId {
	const saved = localStorage.getItem('zephyr-theme') as ThemeId | null;
	if (saved && themes.some((t) => t.id === saved)) return saved;
	return 'dark';
}

export function initTheme() {
	setTheme(getTheme());
	initEasterEgg();
}
