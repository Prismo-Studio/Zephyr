<script lang="ts">
	import Icon from '@iconify/svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import { gameIconSrc } from '$lib/util';
	import type { Game } from '$lib/types';

	type Props = {
		game: Game;
		isActive: boolean;
		onselect: (slug: string) => Promise<void>;
		ontoggleFav: (slug: string, e: MouseEvent) => Promise<void>;
	};

	let { game, isActive, onselect, ontoggleFav }: Props = $props();

	function truncate(name: string, max = 14): string {
		return name.length > max ? name.slice(0, max) + '...' : name;
	}

	function swallow(e: Event) {
		e.stopPropagation();
		e.preventDefault();
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="z-game-card"
	class:active={isActive}
	onclick={() => onselect(game.slug)}
	ondblclick={async () => {
		await onselect(game.slug);
		window.location.href = '/';
	}}
	role="button"
	tabindex="0"
>
	<img src={gameIconSrc(game)} alt={game.name} class="z-game-card-img" />
	<div class="z-game-card-info">
		{#if game.name.length > 14}
			<Tooltip text={game.name} position="top" delay={200}>
				<span class="z-game-card-name">{truncate(game.name)}</span>
			</Tooltip>
		{:else}
			<span class="z-game-card-name">{game.name}</span>
		{/if}
		<span class="z-game-card-loader">{game.modLoader}</span>
	</div>
	<button
		class="z-game-fav-btn-small"
		onclick={(e) => ontoggleFav(game.slug, e)}
		ondblclick={swallow}
		onpointerdown={swallow}
		aria-label="Toggle favorite"
	>
		<Icon
			icon={game.favorite ? 'mdi:star' : 'mdi:star-outline'}
			class={game.favorite ? 'z-fav-icon fav' : 'z-fav-icon'}
		/>
	</button>
</div>

<style>
	.z-game-card {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-md);
		border-radius: var(--radius-lg);
		background: var(--bg-surface);
		border: 1px solid var(--border-subtle);
		cursor: pointer;
		transition: all var(--transition-fast);
		position: relative;
		text-align: left;
	}

	.z-game-card:hover {
		border-color: var(--border-default);
		background: var(--bg-elevated);
		transform: translateY(-1px);
		box-shadow: var(--shadow-md);
	}

	.z-game-card.active {
		border-color: var(--border-accent);
		background: var(--bg-active);
	}

	.z-game-card-img {
		width: 40px;
		height: 40px;
		border-radius: var(--radius-md);
		object-fit: cover;
		flex-shrink: 0;
	}

	.z-game-card-info {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}

	.z-game-card-name {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.z-game-card-loader {
		font-size: 11px;
		color: var(--text-muted);
	}

	.z-game-fav-btn-small {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: 50%;
		background: var(--bg-overlay);
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.z-game-fav-btn-small:hover {
		background: var(--bg-active);
		transform: scale(1.1);
	}
</style>
