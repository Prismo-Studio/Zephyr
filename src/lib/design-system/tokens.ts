export type ThemeId = 'dark' | 'light' | 'zephyr' | 'midnight' | 'hotdog';

export const themes: { id: ThemeId; label: string; hidden?: boolean }[] = [
	{ id: 'dark', label: 'Dark' },
	{ id: 'light', label: 'Light' },
	{ id: 'zephyr', label: 'Zephyr' },
	{ id: 'midnight', label: 'Midnight Ocean' },
	{ id: 'hotdog', label: 'Hot Dog Stand', hidden: true }
];

const HOTDOG_KEY = 'zephyr-hotdog-unlocked';
const SEQUENCE = 'hotdogstand';

let buffer = '';
let listener: ((e: KeyboardEvent) => void) | null = null;

export function isHotdogUnlocked(): boolean {
	return localStorage.getItem(HOTDOG_KEY) === 'true';
}

export function unlockHotdog() {
	localStorage.setItem(HOTDOG_KEY, 'true');
}

export function getVisibleThemes() {
	return themes.filter((t) => !t.hidden || (t.id === 'hotdog' && isHotdogUnlocked()));
}

export function initEasterEgg() {
	if (listener) return;
	listener = (e: KeyboardEvent) => {
		if (e.key === ' ') return;
		buffer += e.key.toLowerCase();
		if (buffer.length > SEQUENCE.length) {
			buffer = buffer.slice(-SEQUENCE.length);
		}
		if (buffer === SEQUENCE && !isHotdogUnlocked()) {
			unlockHotdog();
			setTheme('hotdog');
			window.dispatchEvent(new CustomEvent('hotdog-unlocked'));
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
