/// <reference types="svelte" />
import { m } from '$lib/paraglide/messages';
import { getLocale, locales, setLocale, type Locale } from '$lib/paraglide/runtime';
import { toSentenceCase as toSentenceCaseLatin } from 'js-convert-case';

// Reactive state for the current locale to trigger Svelte re-renders
// We've renamed this to 'i18nCore' to break all stale Vite caches.
export const i18nState = $state({
	locale: getLocale()
});

export function updateAppLanguage(lang: Locale | string) {
	setLocale(lang as Locale, { reload: false });
	i18nState.locale = lang as Locale;
}

export const languageTitle: Record<Locale, string> = locales.reduce(
	(acc, item) => {
		acc[item] = m.language_name({}, { locale: item });
		return acc;
	},
	{} as Record<Locale, string>
);

export function isLatinAlphabet(str: string): boolean {
	return /^[a-zA-Z\s]*$/.test(str);
}

export function pluralizeOption(
	isPlural: boolean | number,
	origin: string,
	singular: string,
	plural: string
): string {
	if (typeof isPlural === 'number' && isPlural !== 1) {
		return origin;
	}

	if (typeof isPlural === 'boolean' && !isPlural) {
		return origin;
	}

	if (!isLatinAlphabet(origin)) {
		return origin;
	}

	return origin.replace(new RegExp(singular, 'g'), plural);
}

export function toSentenceCase(str: string): string {
	return isLatinAlphabet(str) ? toSentenceCaseLatin(str) : str;
}
