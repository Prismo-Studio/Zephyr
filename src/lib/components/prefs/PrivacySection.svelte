<script lang="ts">
	import PrefSection from './PrefSection.svelte';
	import PrefRow from './PrefRow.svelte';
	import Toggle from '$lib/components/ui/Toggle.svelte';
	import { telemetryEnabled, setTelemetryEnabled, telemetryAvailable } from '$lib/telemetry.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	const available = telemetryAvailable();
</script>

{#if available}
	<PrefSection
		icon="mdi:shield-lock-outline"
		title={(i18nState.locale && m.prefs_privacy_title()) ?? ''}
	>
		<PrefRow
			title={(i18nState.locale && m.prefs_privacy_telemetry_title()) ?? ''}
			description={(i18nState.locale && m.prefs_privacy_telemetry_description()) ?? ''}
		>
			{#snippet control()}
				<Toggle checked={telemetryEnabled.current} onchange={(v) => setTelemetryEnabled(v)} />
			{/snippet}
		</PrefRow>
	</PrefSection>
{/if}
