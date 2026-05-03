import { isLatinAlphabet } from '$lib/i18nCore.svelte';
import { m } from '$lib/paraglide/messages';

export function shortenFileSize(size: number): string {
	const i = size === 0 ? 0 : Math.floor(Math.log(size) / Math.log(1024));
	return (size / Math.pow(1024, i)).toFixed(1) + ['B', 'kB', 'MB', 'GB', 'TB'][i];
}

function pluralize(str: string): string {
	return isLatinAlphabet(str) ? str + 's' : str;
}

export function formatModName(name: string): string {
	return name.replace(/_/g, ' ');
}

export function formatTime(seconds: number): string {
	if (seconds < 60) {
		return m.util_formatTime_seconds({ seconds: Math.round(seconds) });
	}

	if (seconds < 3600) {
		const minutes = Math.floor(seconds / 60);
		if (minutes > 1) {
			return pluralize(m.util_formatTime_minute({ minutes }));
		}
		return m.util_formatTime_minute({ minutes });
	}

	const hours = Math.floor(seconds / 3600);
	if (hours > 1) {
		return pluralize(m.util_formatTime_hour({ hours }));
	}
	return m.util_formatTime_hour({ hours });
}

export function shortenNum(value: number): string {
	const i = value === 0 ? 0 : Math.floor(Math.log(value) / Math.log(1000));
	if (i === 0) {
		return value.toString();
	}
	return (value / Math.pow(1000, i)).toFixed(1) + ['', 'k', 'M', 'G', 'T'][i];
}

export function timeSince(date: Date | string): string {
	const seconds = Math.floor((Date.now() - new Date(date).getTime()) / 1000);

	const [interval, singular, plural] = (() => {
		let i = Math.floor(seconds / (60 * 60 * 24 * 365.25));
		if (i >= 1) return [i, m.util_timeSince_year(), m.util_timeSince_year_plural()];

		i = Math.floor(seconds / (60 * 60 * 24 * 30));
		if (i >= 1) return [i, m.util_timeSince_month(), m.util_timeSince_month_plural()];

		i = Math.floor(seconds / (60 * 60 * 24 * 7));
		if (i >= 1) return [i, m.util_timeSince_week(), m.util_timeSince_week_plural()];

		i = Math.floor(seconds / (60 * 60 * 24));
		if (i >= 1) return [i, m.util_timeSince_day(), m.util_timeSince_day_plural()];

		i = Math.floor(seconds / (60 * 60));
		if (i >= 1) return [i, m.util_timeSince_hour(), m.util_timeSince_hour_plural()];

		i = Math.floor(seconds / 60);
		if (i >= 1) return [i, m.util_timeSince_minute(), m.util_timeSince_minute_plural()];

		return [null, null, null];
	})();

	if (interval === null) {
		return m.util_timeSince_interval_null();
	}
	if (interval === 1) {
		return m.util_timeSince_interval_1({ str: singular! });
	}
	return m.util_timeSince_interval_default({ interval, str: plural! });
}

export function capitalize(str: string): string {
	if (!isLatinAlphabet(str)) return str;
	return str.charAt(0).toUpperCase() + str.slice(1);
}
