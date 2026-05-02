<script lang="ts">
	import Header from '$lib/components/layout/Header.svelte';
	import ThemeSection from '$lib/components/prefs/ThemeSection.svelte';
	import FontSection from '$lib/components/prefs/FontSection.svelte';
	import LanguageSection from '$lib/components/prefs/LanguageSection.svelte';
	import WindowSection from '$lib/components/prefs/WindowSection.svelte';
	import DisplaySection from '$lib/components/prefs/DisplaySection.svelte';
	import GamepadSection from '$lib/components/prefs/GamepadSection.svelte';
	import SourcesSection from '$lib/components/prefs/SourcesSection.svelte';
	import BehaviorSection from '$lib/components/prefs/BehaviorSection.svelte';
	import PathsSection from '$lib/components/prefs/PathsSection.svelte';
	import ActionsSection from '$lib/components/prefs/ActionsSection.svelte';
	import AboutSection from '$lib/components/prefs/AboutSection.svelte';
	import CustomThemeModal from '$lib/components/prefs/CustomThemeModal.svelte';

	import * as api from '$lib/api';
	import type { Prefs } from '$lib/types';
	import { onMount } from 'svelte';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { m } from '$lib/paraglide/messages';

	let prefs: Prefs | null = $state(null);
	let customModalOpen = $state(false);

	async function savePrefs() {
		if (prefs) await api.prefs.set(prefs);
	}

	onMount(async () => {
		prefs = await api.prefs.get();
	});
</script>

<div class="z-settings-page">
	<div class="z-settings-header-wrapper">
		<Header title={i18nState.locale && m.navBar_label_settings()} />
	</div>

	<div class="z-settings-content">
		<ThemeSection onpickCustom={() => (customModalOpen = true)} />
		<FontSection />
		{#if prefs}<LanguageSection {prefs} onsave={savePrefs} />{/if}
		<WindowSection />
		{#if prefs}<DisplaySection {prefs} />{/if}
		{#if prefs}<GamepadSection {prefs} onsave={savePrefs} />{/if}
		<SourcesSection />
		{#if prefs}
			<BehaviorSection {prefs} onsave={savePrefs} />
			<PathsSection {prefs} onsave={savePrefs} />
		{/if}
		<ActionsSection />
		<AboutSection />
	</div>
</div>

<CustomThemeModal bind:open={customModalOpen} />

<style>
	.z-settings-page {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.z-settings-content {
		flex: 1;
		overflow-y: auto;
		width: 100%;
		padding-bottom: var(--space-3xl);
	}

	.z-settings-header-wrapper {
		width: 100%;
		max-width: 720px;
		margin: 0 auto;
		padding: var(--space-xl) var(--space-xl) 0;
	}

	.z-settings-header-wrapper :global(.z-header) {
		padding-left: 0;
		padding-right: 0;
	}

	.z-settings-header-wrapper :global(.z-header-title) {
		font-size: 28px;
	}

	/* Target only <section> children — section components (WindowSection, etc.)
	   may also render sibling modals/dialogs which must NOT inherit the
	   centered 720px constraint, otherwise their fixed-position backdrops
	   get clipped to that width. Longhand margin so the bottom-margin set
	   by each section component survives the cascade. */
	.z-settings-content > :global(section) {
		max-width: 720px;
		margin-left: auto;
		margin-right: auto;
		width: 100%;
		padding: 0 var(--space-xl);
	}
</style>
