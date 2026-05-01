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
		padding: var(--space-3xl);
		gap: var(--space-sm);
	}

	.z-browse-empty-icon {
		width: 64px;
		height: 64px;
		border-radius: var(--radius-xl);
		background: var(--bg-elevated);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 28px;
		color: var(--text-muted);
	}

	.z-browse-empty-title {
		font-size: 15px;
		font-weight: 600;
		color: var(--text-secondary);
	}

	.z-browse-empty-desc {
		font-size: 13px;
		color: var(--text-muted);
	}
</style>
