<script lang="ts">
	import Header from '$lib/components/layout/Header.svelte';
	import Icon from '@iconify/svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	/* Placeholder — modpack export/import page, minimal for now */
</script>

<div class="z-modpack-page">
	<Header
		title={i18nState.locale && m.modpack_header_title()}
		subtitle={i18nState.locale && m.modpack_header_description()}
	/>

	<div class="z-modpack-content">
		<div class="z-modpack-placeholder">
			<Icon icon="mdi:package-variant-closed" class="z-modpack-icon" />
			<p>Modpack management coming in v0.3</p>
		</div>
	</div>
</div>

<style>
	.z-modpack-page {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.z-modpack-content {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.z-modpack-placeholder {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-md);
		color: var(--text-muted);
		font-size: 14px;
	}

	:global(.z-modpack-icon) {
		font-size: 48px;
		opacity: 0.3;
	}
</style>
