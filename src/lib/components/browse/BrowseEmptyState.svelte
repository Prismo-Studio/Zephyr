<script lang="ts">
	import Icon from '@iconify/svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		curseForgeOnly: boolean;
	};

	let { curseForgeOnly }: Props = $props();
</script>

<div class="z-browse-empty">
	<div class="z-browse-empty-icon">
		<Icon icon="mdi:package-variant-remove" />
	</div>
	<p class="z-browse-empty-title">{i18nState.locale && m.browse_noMods()}</p>
	<p class="z-browse-empty-desc">
		{curseForgeOnly
			? 'No CurseForge mods available for this game.'
			: i18nState.locale && m.browse_noMods_desc()}
	</p>
</div>

<style>
	.z-browse-empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: var(--space-3xl) var(--space-xl);
		max-width: 360px;
		margin: 0 auto;
	}

	.z-browse-empty-icon {
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

	.z-browse-empty-title {
		font-family: var(--font-display);
		font-size: 15px;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0 0 var(--space-xs);
		text-align: center;
	}

	.z-browse-empty-desc {
		font-size: 13px;
		color: var(--text-muted);
		line-height: 1.5;
		text-align: center;
		margin: 0;
	}
</style>
