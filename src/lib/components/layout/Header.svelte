<script lang="ts">
	import type { Snippet } from 'svelte';
	import Icon from '@iconify/svelte';
	import profiles from '$lib/state/profile.svelte';
	import games from '$lib/state/game.svelte';

	type Props = {
		title?: string;
		subtitle?: string;
		children?: Snippet;
		actions?: Snippet;
	};

	let { title = '', subtitle = '', children, actions }: Props = $props();
</script>

<header class="z-header">
	<div class="z-header-left">
		{#if title}
			<div class="z-header-titles">
				<h1 class="z-header-title">{title}</h1>
				{#if subtitle}
					<span class="z-header-subtitle">{subtitle}</span>
				{/if}
			</div>
		{/if}
		{#if children}{@render children()}{/if}
	</div>

	<div class="z-header-right">
		{#if actions}{@render actions()}{/if}
	</div>
</header>

<style>
	.z-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-lg) var(--space-xl);
		min-height: 56px;
		gap: var(--space-lg);
		flex-shrink: 0;
		border-bottom: 1px solid var(--border-subtle);
	}

	.z-header-left {
		display: flex;
		align-items: center;
		gap: var(--space-lg);
		flex: 1;
		min-width: 0;
	}

	.z-header-titles {
		display: flex;
		align-items: baseline;
		gap: var(--space-md);
	}

	.z-header-title {
		font-family: var(--font-display);
		font-size: 20px;
		font-weight: 800;
		color: var(--text-primary);
		letter-spacing: -0.03em;
	}

	.z-header-subtitle {
		font-size: 13px;
		color: var(--text-muted);
		font-weight: 500;
	}

	.z-header-right {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		flex-shrink: 0;
	}
</style>
