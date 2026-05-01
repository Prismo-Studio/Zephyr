<script lang="ts">
	import Icon from '@iconify/svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
</script>

<div class="z-locked-banner">
	<Icon icon="mdi:lock" />
	<span>{i18nState.locale && m.browse_lockedBanner()}</span>
</div>

<style>
	.z-locked-banner {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-md);
		background: rgba(255, 179, 71, 0.08);
		border: 1px solid rgba(255, 179, 71, 0.2);
		color: var(--warning);
		font-size: 12px;
		margin-bottom: var(--space-sm);
	}
</style>
