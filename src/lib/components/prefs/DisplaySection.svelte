<script lang="ts">
	import PrefSection from './PrefSection.svelte';
	import PrefRow from './PrefRow.svelte';
	import Dropdown from '$lib/components/ui/Dropdown.svelte';
	import * as api from '$lib/api';
	import type { Prefs } from '$lib/types';
	import { onMount } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		prefs: Prefs;
	};

	let { prefs }: Props = $props();

	const dpiScaleOptions = [
		{ value: '0.75', label: '75%' },
		{ value: '0.85', label: '85%' },
		{ value: '0.9', label: '90%' },
		{ value: '1', label: '100%' },
		{ value: '1.1', label: '110% (default)' },
		{ value: '1.15', label: '115%' },
		{ value: '1.25', label: '125%' },
		{ value: '1.5', label: '150%' },
		{ value: '1.75', label: '175%' },
		{ value: '2', label: '200%' }
	];

	async function changeDpiScale(value: string) {
		const scale = parseFloat(value);
		if (isNaN(scale)) return;
		prefs.dpiScale = await api.prefs.setDpiScale(scale);
	}

	onMount(() => {
		const onScaleChanged = ((e: CustomEvent) => {
			prefs.dpiScale = e.detail;
		}) as EventListener;
		window.addEventListener('dpi-scale-changed', onScaleChanged);
		return () => window.removeEventListener('dpi-scale-changed', onScaleChanged);
	});
</script>

<PrefSection icon="mdi:monitor" title={(i18nState.locale && m.prefs_display_title()) ?? ''}>
	<PrefRow
		title={(i18nState.locale && m.prefs_display_dpiScale_title()) ?? ''}
		description={(i18nState.locale && m.prefs_display_dpiScale_desc()) ?? ''}
	>
		{#snippet control()}
			<Dropdown
				options={dpiScaleOptions}
				value={String(prefs.dpiScale)}
				onchange={changeDpiScale}
				placeholder="100%"
			/>
		{/snippet}
	</PrefRow>
</PrefSection>
