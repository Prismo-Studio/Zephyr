<script lang="ts">
	import PrefSection from './PrefSection.svelte';
	import PrefRow from './PrefRow.svelte';
	import Toggle from '$lib/components/ui/Toggle.svelte';
	import type { Prefs } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		prefs: Prefs;
		onsave: () => Promise<void>;
	};

	let { prefs, onsave }: Props = $props();
</script>

<PrefSection icon="mdi:cog" title={(i18nState.locale && m.prefs_miscellaneous_title()) ?? ''}>
	<PrefRow
		title={(i18nState.locale && m.prefs_miscellaneous_fetchMods_title()) ?? ''}
		description={(i18nState.locale && m.prefs_miscellaneous_fetchMods_content_1()) ?? ''}
	>
		{#snippet control()}
			<Toggle
				checked={prefs.fetchModsAutomatically}
				onchange={(v) => {
					prefs.fetchModsAutomatically = v;
					onsave();
				}}
			/>
		{/snippet}
	</PrefRow>

	<PrefRow
		title={(i18nState.locale && m.prefs_miscellaneous_pullBeforeLaunch_title()) ?? ''}
		description={(i18nState.locale && m.prefs_miscellaneous_pullBeforeLaunch_content()) ?? ''}
	>
		{#snippet control()}
			<Toggle
				checked={prefs.pullBeforeLaunch}
				onchange={(v) => {
					prefs.pullBeforeLaunch = v;
					onsave();
				}}
			/>
		{/snippet}
	</PrefRow>
</PrefSection>
