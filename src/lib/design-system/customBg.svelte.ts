type MediaKind = 'image' | 'video';

export type CustomBgMedia = {
	kind: MediaKind;
	url: string;
};

const STORAGE_KEY = 'zephyr-custom-bg';

function load(): CustomBgMedia | null {
	if (typeof localStorage === 'undefined') return null;
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (!raw) return null;
		const parsed = JSON.parse(raw) as Partial<CustomBgMedia>;
		if (
			(parsed.kind === 'image' || parsed.kind === 'video') &&
			typeof parsed.url === 'string' &&
			parsed.url.length > 0
		) {
			return { kind: parsed.kind, url: parsed.url };
		}
		return null;
	} catch {
		return null;
	}
}

function save(value: CustomBgMedia | null) {
	if (typeof localStorage === 'undefined') return;
	if (value) {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
	} else {
		localStorage.removeItem(STORAGE_KEY);
	}
}

function syncAttr() {
	if (typeof document === 'undefined') return;
	const themeIsCustom = document.documentElement.getAttribute('data-theme') === 'custom';
	if (customBgState.media && themeIsCustom) {
		document.documentElement.setAttribute('data-bg-media', '1');
	} else {
		document.documentElement.removeAttribute('data-bg-media');
	}
}

export const customBgState = $state<{ media: CustomBgMedia | null; themeActive: boolean }>({
	media: load(),
	themeActive: false
});

let themeObserver: MutationObserver | null = null;

export function initCustomBg() {
	if (typeof document === 'undefined') return;
	customBgState.themeActive = document.documentElement.getAttribute('data-theme') === 'custom';
	syncAttr();
	if (themeObserver) return;
	themeObserver = new MutationObserver(() => {
		const active = document.documentElement.getAttribute('data-theme') === 'custom';
		if (active !== customBgState.themeActive) {
			customBgState.themeActive = active;
		}
		syncAttr();
	});
	themeObserver.observe(document.documentElement, {
		attributes: true,
		attributeFilter: ['data-theme']
	});
}

export function setCustomBg(media: CustomBgMedia) {
	customBgState.media = media;
	save(media);
	syncAttr();
}

export function clearCustomBg() {
	customBgState.media = null;
	save(null);
	syncAttr();
}
