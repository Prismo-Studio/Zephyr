<script lang="ts">
	import Icon from '@iconify/svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
</script>

<div class="z-mods-empty">
	<div class="z-empty-icon">
		<Icon icon="mdi:package-variant" />
	</div>
	<p class="z-empty-title">{i18nState.locale && m.mods_noMods()}</p>
	<p class="z-empty-desc">{i18nState.locale && m.mods_noMods_desc()}</p>
	<a href="/browse" class="z-empty-action">
		<Icon icon="mdi:store-search" />
		{i18nState.locale && m.mods_browseMods()}
	</a>
</div>

<style>
	.z-mods-empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: var(--space-3xl) var(--space-xl);
		gap: var(--space-sm);
	}

	.z-empty-icon {
		width: 64px;
		height: 64px;
		border-radius: var(--radius-xl);
		background: var(--bg-elevated);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 28px;
		color: var(--text-muted);
		margin-bottom: var(--space-sm);
	}

	.z-empty-title {
		font-family: var(--font-display);
		font-size: 16px;
		font-weight: 700;
		color: var(--text-primary);
	}

	.z-empty-desc {
		font-size: 13px;
		color: var(--text-muted);
	}

	.z-empty-action {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-top: var(--space-md);
		padding: var(--space-sm) var(--space-lg);
		border-radius: var(--radius-md);
		background: var(--bg-active);
		color: var(--text-accent);
		font-size: 13px;
		font-weight: 600;
		text-decoration: none;
		transition: all var(--transition-fast);
	}

	.z-empty-action:hover {
		background: var(--bg-active);
		box-shadow: var(--shadow-glow);
	}
</style>
