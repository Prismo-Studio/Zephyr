<script lang="ts">
	import Icon from '@iconify/svelte';
	import { gameIconSrc } from '$lib/util';
	import type { Game } from '$lib/types';

	type Props = {
		game: Game;
		isActive: boolean;
		onselect: (slug: string) => Promise<void>;
		ontoggleFav: (slug: string, e: MouseEvent) => Promise<void>;
	};

	let { game, isActive, onselect, ontoggleFav }: Props = $props();

	function swallow(e: Event) {
		e.stopPropagation();
		e.preventDefault();
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="z-game-tile"
	class:active={isActive}
	onclick={() => onselect(game.slug)}
	ondblclick={async () => {
		await onselect(game.slug);
		window.location.href = '/';
	}}
	role="button"
	tabindex="0"
>
	<img src={gameIconSrc(game)} alt={game.name} class="z-game-tile-img" />
	<div class="z-game-tile-overlay">
		<button
			class="z-game-fav-btn"
			onclick={(e) => ontoggleFav(game.slug, e)}
			ondblclick={swallow}
			onpointerdown={swallow}
			aria-label="Toggle favorite"
		>
			<Icon icon="mdi:star" class="z-fav-icon fav" />
		</button>
		<div class="z-game-tile-info">
			<span class="z-game-tile-name">{game.name}</span>
			<span class="z-game-tile-loader">
				<Icon icon="mdi:package-variant" />
				{game.modLoader}
			</span>
		</div>
	</div>
</div>

<style>
	.z-game-tile {
		position: relative;
		aspect-ratio: 4 / 3;
		border-radius: var(--radius-lg);
		overflow: hidden;
		border: 1px solid var(--border-subtle);
		cursor: pointer;
		background: var(--bg-surface);
		padding: 0;
		transition: all var(--transition-fast);
	}

	.z-game-tile:hover {
		transform: translateY(-2px);
		border-color: var(--border-accent);
		box-shadow: var(--shadow-glow);
	}

	.z-game-tile.active {
		border-color: var(--accent-400);
		border-width: 2px;
		box-shadow: var(--shadow-glow-strong);
	}

	.z-game-tile-img {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		object-fit: cover;
		transition: transform 400ms ease;
	}

	.z-game-tile:hover .z-game-tile-img {
		transform: scale(1.05);
	}

	.z-game-tile-overlay {
		position: absolute;
		inset: 0;
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		padding: var(--space-md);
		background: linear-gradient(180deg, rgba(0, 0, 0, 0) 40%, rgba(0, 0, 0, 0.85) 100%);
	}

	.z-game-tile-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		color: #ffffff;
	}

	.z-game-tile-name {
		font-size: 14px;
		font-weight: 700;
		text-shadow: 0 2px 6px rgba(0, 0, 0, 0.5);
	}

	.z-game-tile-loader {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		color: rgba(255, 255, 255, 0.75);
	}

	.z-game-fav-btn {
		position: absolute;
		top: 8px;
		right: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border-radius: 50%;
		background: rgba(0, 0, 0, 0.5);
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		backdrop-filter: blur(4px);
		transition: all var(--transition-fast);
		z-index: 2;
	}

	.z-game-fav-btn:hover {
		background: rgba(0, 0, 0, 0.75);
		transform: scale(1.1);
	}

	:global(.z-fav-icon) {
		font-size: 16px;
	}

	:global(.z-fav-icon.fav) {
		color: #fbbf24;
	}
</style>
