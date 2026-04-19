export type ThemeId =
	| 'dark'
	| 'light'
	| 'zephyr'
	| 'midnight'
	| 'crimson'
	| 'amethyst'
	| 'pinky'
	| 'custom'
	| 'hotdog';

export const themes: { id: ThemeId; label: string; hidden?: boolean }[] = [
	{ id: 'dark', label: 'Dark' },
	{ id: 'light', label: 'Light' },
	{ id: 'zephyr', label: 'Zephyr' },
	{ id: 'midnight', label: 'Midnight Ocean' },
	{ id: 'crimson', label: 'Crimson' },
	{ id: 'amethyst', label: 'Amethyst' },
	{ id: 'pinky', label: 'Pinky' },
	{ id: 'custom', label: 'Custom' },
	{ id: 'hotdog', label: 'Hot Dog Stand', hidden: true }
];

export type CustomThemeColors = {
	accent: string;
	bg: string;
	bgCard: string;
	text: string;
};

export const DEFAULT_CUSTOM_COLORS: CustomThemeColors = {
	accent: '#1afffa',
	bg: '#080e1a',
	bgCard: '#0f1a2e',
	text: '#f0f4f8'
};

const CUSTOM_KEY = 'zephyr-custom-theme';

function hexToRgb(hex: string): { r: number; g: number; b: number } {
	const m = hex.replace('#', '').match(/^([\da-f]{2})([\da-f]{2})([\da-f]{2})$/i);
	if (!m) return { r: 0, g: 0, b: 0 };
	return { r: parseInt(m[1], 16), g: parseInt(m[2], 16), b: parseInt(m[3], 16) };
}

function rgba(hex: string, alpha: number): string {
	const { r, g, b } = hexToRgb(hex);
	return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

function mix(hex: string, mixWith: string, ratio: number): string {
	const a = hexToRgb(hex);
	const b = hexToRgb(mixWith);
	const r = Math.round(a.r * (1 - ratio) + b.r * ratio);
	const g = Math.round(a.g * (1 - ratio) + b.g * ratio);
	const bl = Math.round(a.b * (1 - ratio) + b.b * ratio);
	return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${bl.toString(16).padStart(2, '0')}`;
}

export function getCustomColors(): CustomThemeColors {
	if (typeof localStorage === 'undefined') return DEFAULT_CUSTOM_COLORS;
	try {
		const raw = localStorage.getItem(CUSTOM_KEY);
		if (!raw) return DEFAULT_CUSTOM_COLORS;
		const parsed = JSON.parse(raw);
		return { ...DEFAULT_CUSTOM_COLORS, ...parsed };
	} catch {
		return DEFAULT_CUSTOM_COLORS;
	}
}

export function setCustomColors(colors: CustomThemeColors) {
	localStorage.setItem(CUSTOM_KEY, JSON.stringify(colors));
	applyCustomTheme(colors);
}

export function applyCustomTheme(colors: CustomThemeColors) {
	if (typeof document === 'undefined') return;
	const id = 'zephyr-custom-theme-style';
	let style = document.getElementById(id) as HTMLStyleElement | null;
	if (!style) {
		style = document.createElement('style');
		style.id = id;
		document.head.appendChild(style);
	}
	const accentDark = mix(colors.accent, '#000000', 0.15);
	const accentLight = mix(colors.accent, '#ffffff', 0.2);
	const bgElevated = mix(colors.bgCard, '#ffffff', 0.04);
	const bgOverlay = mix(colors.bgCard, '#ffffff', 0.08);
	const textSecondary = mix(colors.text, colors.bg, 0.4);
	const textMuted = mix(colors.text, colors.bg, 0.65);
	style.textContent = `
		[data-theme='custom'] {
			--bg-base: ${colors.bg};
			--bg-surface: ${colors.bgCard};
			--bg-elevated: ${bgElevated};
			--bg-overlay: ${bgOverlay};
			--bg-hover: ${rgba(colors.accent, 0.06)};
			--bg-active: ${rgba(colors.accent, 0.12)};
			--border-subtle: ${rgba(colors.text, 0.06)};
			--border-default: ${rgba(colors.text, 0.1)};
			--border-strong: ${rgba(colors.text, 0.16)};
			--border-accent: ${rgba(colors.accent, 0.35)};
			--text-primary: ${colors.text};
			--text-secondary: ${textSecondary};
			--text-muted: ${textMuted};
			--text-accent: ${colors.accent};
			--text-inverse: ${colors.bg};
			--accent-300: ${accentLight};
			--accent-400: ${colors.accent};
			--accent-500: ${accentDark};
			--accent-600: ${mix(colors.accent, '#000000', 0.25)};
			--accent-700: ${mix(colors.accent, '#000000', 0.35)};
			--shadow-glow: 0 0 20px ${rgba(colors.accent, 0.18)};
			--shadow-glow-strong: 0 0 40px ${rgba(colors.accent, 0.28)};
		}
	`;
}

const HOTDOG_KEY = 'zephyr-hotdog-unlocked';
const HOTDOG_SEQUENCE = 'hotdogstand';

const WINGDINGS_SEQUENCE = 'wingdings';

// Dev-only easter egg. Uncomment the const + the detection block in
// initEasterEgg() AND the <DoomEasterEgg /> mount in +layout.svelte to
// re-enable. Trigger: type the Quake III fast inverse square root magic
// number `5f3759df` anywhere in the app to launch DOOM.
// const DOOM_SEQUENCE = '5f3759df';

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

		if (buffer.endsWith(HOTDOG_SEQUENCE) && !isHotdogUnlocked()) {
			unlockHotdog();
			setTheme('hotdog');
			window.dispatchEvent(new CustomEvent('hotdog-unlocked'));
		}

		if (buffer.endsWith(WINGDINGS_SEQUENCE) && !isWingdingsUnlocked()) {
			unlockWingdings();
			window.dispatchEvent(new CustomEvent('wingdings-unlocked'));
		}

		// if (buffer.endsWith(DOOM_SEQUENCE)) {
		// 	window.dispatchEvent(new CustomEvent('doom-unlocked'));
		// }
	};
	window.addEventListener('keydown', listener);
}

export function setTheme(theme: ThemeId) {
	if (theme === 'custom') {
		applyCustomTheme(getCustomColors());
	}
	document.documentElement.setAttribute('data-theme', theme);
	localStorage.setItem('zephyr-theme', theme);
}

export function getTheme(): ThemeId {
	const saved = localStorage.getItem('zephyr-theme') as ThemeId | null;
	if (saved && themes.some((t) => t.id === saved)) return saved;
	return 'dark';
}

export function initTheme() {
	applyCustomTheme(getCustomColors());
	setTheme(getTheme());
	initEasterEgg();
}
