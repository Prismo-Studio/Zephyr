import { PersistedState } from 'runed';
import getPalette from 'tailwindcss-palette-generator';
import {
	DEFAULT_COLORS,
	DEFAULT_FONT,
	BUNDLED_FONTS,
	GOOGLE_FONTS_URL,
	type DefaultColorName
} from './constants/colors.constants';

export { DEFAULT_COLORS, type DefaultColorName };

export const defaultColors = DEFAULT_COLORS;
export type DefaultColor = DefaultColorName;
export type ColorCategory = 'accent' | 'primary';

export type Color =
	| {
			type: 'default';
			name: DefaultColor;
	  }
	| {
			type: 'custom';
			hex: string;
	  };

const getRoot = () =>
	typeof document !== 'undefined' ? (document.querySelector(':root') as HTMLElement) : null;

const fallbacks: Record<ColorCategory, Color> = {
	accent: { type: 'custom', hex: '#2D8CF0' },
	primary: { type: 'custom', hex: '#0B1628' }
};

export function setColor(category: ColorCategory, color: Color) {
	let shades: { [shade: string]: string };

	if (color.type === 'default') {
		shades = DEFAULT_COLORS[color.name];
	} else {
		let palette = getPalette({
			color: color.hex,
			name: 'main'
		});

		shades = palette['main'];
	}

	const root = getRoot();
	if (root) {
		for (const [shade, value] of Object.entries(shades)) {
			root.style.setProperty(`--color-${category}-${shade}`, value);
		}
	}

	localStorage.setItem(category + 'Color', JSON.stringify(color));
}

export function getColor(category: ColorCategory): Color {
	let json = localStorage.getItem(category + 'Color');

	if (json === null) {
		return fallbacks[category];
	}

	try {
		return JSON.parse(json) as Color;
	} catch (e) {
		console.error('Failed to parse saved color', e);
		return fallbacks[category];
	}
}

export function refreshColor(category: ColorCategory) {
	setColor(category, getColor(category));
}

function loadGoogleFont(fontFamily: string) {
	if (typeof document === 'undefined') return;
	if (BUNDLED_FONTS.has(fontFamily)) return;
	const id = `gfont-${fontFamily.replace(/\s+/g, '-')}`;
	if (document.getElementById(id)) return;
	const link = document.createElement('link');
	link.id = id;
	link.rel = 'stylesheet';
	link.href = GOOGLE_FONTS_URL(fontFamily);
	document.head.appendChild(link);
}

export function setFont(fontFamily: string) {
	const root = getRoot();
	if (root) {
		root.style.setProperty('--font-body', `'${fontFamily}', '${DEFAULT_FONT}', sans-serif`);
		root.style.setProperty('--font-display', `'${fontFamily}', '${DEFAULT_FONT}', sans-serif`);
	}
	loadGoogleFont(fontFamily);
	localStorage.setItem('font', fontFamily);
}

export function getFont() {
	return localStorage.getItem('font') ?? DEFAULT_FONT;
}

export function refreshFont() {
	setFont(getFont());
}

export const useNativeMenu = new PersistedState('useNativeMenu', true);
export const useNativeTitlebar = new PersistedState('useNativeTitlebar', false);
