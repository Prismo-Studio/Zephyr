<script lang="ts">
	import PrefSection from './PrefSection.svelte';
	import Dropdown from '$lib/components/ui/Dropdown.svelte';
	import type { Prefs } from '$lib/types';
	import { updateAppLanguage, languageTitle, i18nState } from '$lib/i18nCore.svelte';
	import { getLocale, locales, type Locale } from '$lib/paraglide/runtime';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		prefs: Prefs;
		onsave: () => Promise<void>;
	};

	let { prefs, onsave }: Props = $props();

	let currentLocale: Locale = $state(getLocale() as Locale);
	let languageOptions = $derived(locales.map((l) => ({ value: l, label: languageTitle[l] })));

	async function changeLanguage(locale: string) {
		currentLocale = locale as Locale;
		prefs.language = locale;
		await onsave();
		updateAppLanguage(locale);
	}
</script>

<PrefSection icon="mdi:translate" title={(i18nState.locale && m.languagePref_title()) ?? ''}>
	<Dropdown
		options={languageOptions}
		value={currentLocale}
		onchange={changeLanguage}
		placeholder={m.languagePref_title()}
	/>
</PrefSection>
