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
		max-width: 360px;
		margin: 0 auto;
	}

	.z-empty-icon {
		width: 88px;
		height: 88px;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 40px;
		color: var(--text-secondary);
		margin-bottom: var(--space-md);
		background: radial-gradient(
			circle at center,
			color-mix(in srgb, var(--text-primary) 6%, transparent) 0%,
			transparent 70%
		);
	}

	.z-empty-title {
		font-family: var(--font-display);
		font-size: 15px;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0 0 var(--space-xs);
		text-align: center;
	}

	.z-empty-desc {
		font-size: 13px;
		color: var(--text-muted);
		line-height: 1.5;
		text-align: center;
		margin: 0 0 var(--space-xl);
	}

	.z-empty-action {
		display: inline-flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm) var(--space-lg);
		border-radius: var(--radius-md);
		background: var(--bg-active);
		border: 1px solid var(--border-accent);
		color: var(--text-accent);
		font-size: 13px;
		font-weight: 600;
		text-decoration: none;
		transition: all var(--transition-fast);
	}

	.z-empty-action:hover {
		box-shadow: var(--shadow-glow);
	}
</style>
