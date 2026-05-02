<script lang="ts">
	import PrefSection from './PrefSection.svelte';
	import Dropdown from '$lib/components/ui/Dropdown.svelte';
	import * as api from '$lib/api';
	import { onMount } from 'svelte';
	import { setFont, getFont } from '$lib/themeSystem';
	import { isWingdingsUnlocked } from '$lib/design-system/tokens';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	let systemFonts: string[] = $state([]);
	let wingdingsUnlocked = $state(isWingdingsUnlocked());
	let currentFont = $state(getFont());

	let fontOptions = $derived([
		{ value: 'Inter', label: 'Inter (default)' },
		{ value: 'Outfit', label: 'Outfit' },
		{ value: 'DM Sans', label: 'DM Sans' },
		...systemFonts.slice(0, 50).map((f: string) => ({ value: f, label: f })),
		...(wingdingsUnlocked ? [{ value: 'Wingdings', label: 'Wingdings' }] : [])
	]);

	function changeFont(font: string) {
		currentFont = font;
		setFont(font);
	}

	onMount(() => {
		api.prefs.getSystemFonts().then((fonts) => (systemFonts = fonts));
		const onWingdings = () => (wingdingsUnlocked = true);
		window.addEventListener('wingdings-unlocked', onWingdings);
		return () => window.removeEventListener('wingdings-unlocked', onWingdings);
	});
</script>

<PrefSection icon="mdi:format-font" title={(i18nState.locale && m.fontFamilyPref_title()) ?? ''}>
	<Dropdown
		options={fontOptions}
		value={currentFont}
		onchange={changeFont}
		placeholder={m.fontFamilyPref_placeholder()}
	/>
</PrefSection>
