<script lang="ts">
	import Icon from '@iconify/svelte';
	import Header from '$lib/components/layout/Header.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';

	function truncate(name: string, max = 14): string {
		return name.length > max ? name.slice(0, max) + '...' : name;
	}
	import games from '$lib/state/game.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { gameIconSrc } from '$lib/util';
	import * as api from '$lib/api';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	let searchTerm = $state('');
	let favorites = $derived(games.list.filter((g) => g.favorite));

	let filteredGames = $derived.by(() => {
		const term = searchTerm.toLowerCase().trim();
		if (!term) return games.list;
		return games.list.filter((g) => g.name.toLowerCase().includes(term));
	});

	// Group by mod loader for variety
	let groupedByLoader = $derived.by(() => {
		const groups = new Map<string, typeof games.list>();
		for (const g of filteredGames) {
			const key = g.modLoader || 'Other';
			if (!groups.has(key)) groups.set(key, []);
			groups.get(key)!.push(g);
		}
		return Array.from(groups.entries()).sort((a, b) => b[1].length - a[1].length);
	});

	async function selectGame(slug: string) {
		await games.setActive(slug);
	}

	async function toggleFavorite(slug: string, e: MouseEvent) {
		e.stopPropagation();
		await api.profile.favoriteGame(slug);
		await games.refresh();
	}

	async function launch() {
		await api.profile.launch.launchGame();
	}
</script>

<div class="z-dashboard">
	<Header title={i18nState.locale && m.dashboard_title()}>
		{#snippet actions()}
			<Button variant="accent" onclick={launch} disabled={!games.active}>
				{#snippet icon()}<Icon icon="mdi:rocket-launch" />{/snippet}
				{i18nState.locale && m.dashboard_launch({ name: games.active?.name ?? 'Game' })}
			</Button>
		{/snippet}
	</Header>

	<div class="z-dashboard-content">
		<!-- Active game hero -->
		{#if games.active}
			<div class="z-hero-card glass">
				<img src={gameIconSrc(games.active)} alt={games.active.name} class="z-hero-bg" />
				<div class="z-hero-overlay">
					<div class="z-hero-info">
						<h2 class="z-hero-title">{games.active.name}</h2>
						<div class="z-hero-meta">
							<Badge variant="accent">{games.active.modLoader}</Badge>
							{#if profiles.active}
								<span class="z-hero-profile">
									<Icon icon="mdi:account-circle" />
									{profiles.active.name}
									<span class="z-hero-mods">· {profiles.active.modCount} mods</span>
								</span>
							{/if}
						</div>
					</div>
					<div class="z-hero-actions">
						<a href="/" class="z-hero-action">
							<Icon icon="mdi:package-variant" />
							<span>{i18nState.locale && m.dashboard_viewMods()}</span>
						</a>
						<a href="/browse" class="z-hero-action">
							<Icon icon="mdi:store-search" />
							<span>{i18nState.locale && m.navBar_label_browse()}</span>
						</a>
					</div>
				</div>
			</div>
		{/if}

		<!-- Quick stats -->
		<div class="z-stats-row">
			<div class="z-stat-card">
				<div class="z-stat-icon"><Icon icon="mdi:gamepad-variant" /></div>
				<div class="z-stat-info">
					<span class="z-stat-value">{games.list.length}</span>
					<span class="z-stat-label">{i18nState.locale && m.dashboard_games()}</span>
				</div>
			</div>
			<div class="z-stat-card">
				<div class="z-stat-icon"><Icon icon="mdi:account-group" /></div>
				<div class="z-stat-info">
					<span class="z-stat-value">{profiles.list.length}</span>
					<span class="z-stat-label">{i18nState.locale && m.dashboard_stats_profiles()}</span>
				</div>
			</div>
			<div class="z-stat-card">
				<div class="z-stat-icon"><Icon icon="mdi:puzzle" /></div>
				<div class="z-stat-info">
					<span class="z-stat-value">{profiles.active?.modCount ?? 0}</span>
					<span class="z-stat-label">{i18nState.locale && m.dashboard_stats_modsInstalled()}</span>
				</div>
			</div>
		</div>

		<!-- Favorites section -->
		{#if favorites.length > 0}
			<section class="z-section">
				<div class="z-section-header">
					<Icon icon="mdi:star" class="z-section-icon" />
					<h3>{i18nState.locale && m.dashboard_yourGames()}</h3>
					<Badge variant="accent">
						{favorites.length}
					</Badge>
				</div>
				<div class="z-games-grid z-games-grid-hero">
					{#each favorites as game (game.slug)}
						<!-- svelte-ignore a11y_click_events_have_key_events -->
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div
							class="z-game-tile"
							class:active={game.slug === games.active?.slug}
							onclick={() => selectGame(game.slug)}
							ondblclick={async () => {
								await selectGame(game.slug);
								window.location.href = '/';
							}}
							role="button"
							tabindex="0"
						>
							<img src={gameIconSrc(game)} alt={game.name} class="z-game-tile-img" />
							<div class="z-game-tile-overlay">
								<button
									class="z-game-fav-btn"
									onclick={(e) => toggleFavorite(game.slug, e)}
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
							{#if game.slug === games.active?.slug}
								<span class="z-game-active-pulse"></span>
							{/if}
						</div>
					{/each}
				</div>
			</section>
		{/if}

		<!-- All games grouped by loader -->
		<section class="z-section z-section-flex">
			<div class="z-section-header">
				<Icon icon="mdi:gamepad-variant" class="z-section-icon" />
				<h3>{i18nState.locale && m.dashboard_yourGames()}</h3>
				<Badge variant="accent">{games.list.length}</Badge>
				<div class="z-section-search">
					<Icon icon="mdi:magnify" />
					<input
						type="text"
						placeholder={i18nState.locale && m.randomizer_searchPlaceholder()}
						bind:value={searchTerm}
					/>
				</div>
			</div>

			{#if filteredGames.length === 0}
				<div class="z-no-games">
					<Icon icon="mdi:gamepad-variant-outline" />
					<p>{i18nState.locale && m.randomizer_noMatch({ search: searchTerm })}</p>
				</div>
			{:else}
				{#each groupedByLoader as [loader, loaderGames] (loader)}
					<div class="z-loader-group">
						<div class="z-loader-group-title">
							<Icon icon="mdi:package-variant" />
							<span>{loader}</span>
							<small>{loaderGames.length}</small>
						</div>
						<div class="z-games-grid">
							{#each loaderGames as game (game.slug)}
								<!-- svelte-ignore a11y_click_events_have_key_events -->
								<!-- svelte-ignore a11y_no_static_element_interactions -->
								<div
									class="z-game-card"
									class:active={game.slug === games.active?.slug}
									onclick={() => selectGame(game.slug)}
									ondblclick={async () => {
										await selectGame(game.slug);
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
										class="z-game-fav-btn z-game-fav-btn-small"
										onclick={(e) => toggleFavorite(game.slug, e)}
										aria-label="Toggle favorite"
									>
										<Icon
											icon={game.favorite ? 'mdi:star' : 'mdi:star-outline'}
											class={game.favorite ? 'z-fav-icon fav' : 'z-fav-icon'}
										/>
									</button>
								</div>
							{/each}
						</div>
					</div>
				{/each}
			{/if}
		</section>
	</div>
</div>

<style>
	.z-dashboard {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.z-dashboard-content {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-md) var(--space-xl) var(--space-xl);
		display: flex;
		flex-direction: column;
		gap: var(--space-xl);
	}

	/* Hero */
	.z-hero-card {
		position: relative;
		border-radius: var(--radius-xl);
		overflow: hidden;
		min-height: 180px;
	}

	.z-hero-bg {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		object-fit: cover;
		filter: blur(40px) brightness(0.3);
		transform: scale(1.2);
	}

	.z-hero-overlay {
		position: relative;
		z-index: 1;
		padding: var(--space-xl);
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		min-height: 180px;
		background: linear-gradient(135deg, rgba(0, 0, 0, 0.6), rgba(0, 0, 0, 0.2));
		color: #ffffff;
	}

	.z-hero-title {
		font-family: var(--font-display);
		font-size: 28px;
		font-weight: 800;
		letter-spacing: -0.03em;
	}

	.z-hero-meta {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		margin-top: var(--space-sm);
	}

	.z-hero-profile {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 13px;
		color: rgba(255, 255, 255, 0.8);
	}

	.z-hero-mods {
		color: rgba(255, 255, 255, 0.5);
	}

	.z-hero-actions {
		display: flex;
		gap: var(--space-sm);
		margin-top: var(--space-lg);
	}

	.z-hero-action {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: var(--space-sm) var(--space-lg);
		border-radius: var(--radius-md);
		background: rgba(255, 255, 255, 0.1);
		color: #ffffff;
		text-decoration: none;
		font-size: 13px;
		font-weight: 600;
		transition: all var(--transition-fast);
		backdrop-filter: blur(8px);
	}

	.z-hero-action:hover {
		background: rgba(255, 255, 255, 0.18);
	}

	/* Stats */
	.z-stats-row {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: var(--space-md);
	}

	.z-stat-card {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-lg);
		border-radius: var(--radius-lg);
		background: var(--bg-surface);
		border: 1px solid var(--border-subtle);
	}

	.z-stat-icon {
		font-size: 24px;
		color: var(--text-accent);
		flex-shrink: 0;
	}

	.z-stat-info {
		display: flex;
		flex-direction: column;
	}

	.z-stat-value {
		font-family: var(--font-display);
		font-size: 22px;
		font-weight: 800;
		color: var(--text-primary);
	}

	.z-stat-label {
		font-size: 11px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	/* Section */
	.z-section {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.z-section-flex {
		flex: 1;
		min-height: 0;
	}

	.z-section-header {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		margin-bottom: var(--space-md);
	}

	.z-section-header h3 {
		font-family: var(--font-display);
		font-size: 16px;
		font-weight: 700;
	}

	:global(.z-section-icon) {
		font-size: 20px;
		color: var(--text-accent);
	}

	.z-section-search {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-left: auto;
		padding: 6px 12px;
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		border: 1px solid var(--border-subtle);
		color: var(--text-muted);
		transition: border-color var(--transition-fast);
		min-width: 240px;
	}

	.z-section-search:focus-within {
		border-color: var(--border-accent);
		color: var(--text-accent);
	}

	.z-section-search input {
		flex: 1;
		background: transparent;
		border: none;
		outline: none;
		color: var(--text-primary);
		font-size: 13px;
	}

	.z-section-search input::placeholder {
		color: var(--text-muted);
	}

	/* Grouped */
	.z-loader-group {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		margin-bottom: var(--space-lg);
	}

	.z-loader-group-title {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 11px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--text-muted);
		padding: 4px 2px;
		border-bottom: 1px solid var(--border-subtle);
	}

	.z-loader-group-title small {
		margin-left: auto;
		font-size: 11px;
		color: var(--text-muted);
		font-weight: 600;
	}

	.z-no-games {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-3xl);
		color: var(--text-muted);
	}

	.z-no-games :global(svg) {
		font-size: 48px;
		opacity: 0.4;
	}

	/* Hero tiles for favorites */
	.z-games-grid-hero {
		grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
		gap: var(--space-md);
	}

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

	.z-game-fav-btn-small {
		width: 24px;
		height: 24px;
		background: var(--bg-overlay);
	}

	.z-game-active-pulse {
		position: absolute;
		top: 12px;
		left: 12px;
		width: 10px;
		height: 10px;
		border-radius: 50%;
		background: var(--accent-400);
		box-shadow: 0 0 0 4px var(--bg-active);
		animation: pulse 2s ease-in-out infinite;
	}

	@keyframes pulse {
		0%,
		100% {
			box-shadow: 0 0 0 4px var(--bg-active);
		}
		50% {
			box-shadow: 0 0 0 8px transparent;
		}
	}

	/* Games grid */
	.z-games-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
		gap: var(--space-md);
	}

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

	:global(.z-game-star) {
		position: absolute;
		top: 8px;
		right: 8px;
		font-size: 14px;
		color: #fbbf24;
	}

	.z-game-active-dot {
		position: absolute;
		bottom: 8px;
		right: 8px;
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--accent-400);
		box-shadow: 0 0 6px var(--accent-400);
	}
</style>
