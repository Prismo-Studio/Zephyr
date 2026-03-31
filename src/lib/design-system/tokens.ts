export type ThemeId = 'dark' | 'light' | 'zephyr' | 'midnight';

export const themes: { id: ThemeId; label: string }[] = [
	{ id: 'dark', label: 'Dark' },
	{ id: 'light', label: 'Light' },
	{ id: 'zephyr', label: 'Zephyr' },
	{ id: 'midnight', label: 'Midnight Ocean' }
];

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
}
