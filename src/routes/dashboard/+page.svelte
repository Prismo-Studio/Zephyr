<script lang="ts">
	import Icon from '@iconify/svelte';
	import Header from '$lib/components/layout/Header.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Button from '$lib/components/ui/Button.svelte';

	import DashboardHero from '$lib/components/dashboard/DashboardHero.svelte';
	import DashboardStats from '$lib/components/dashboard/DashboardStats.svelte';
	import GameTile from '$lib/components/dashboard/GameTile.svelte';
	import GameCard from '$lib/components/dashboard/GameCard.svelte';

	import games from '$lib/state/game.svelte';
	import * as api from '$lib/api';
	import { launchGameWithBepInExFallback } from '$lib/launch';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	let searchTerm = $state('');
	let favorites = $derived(games.list.filter((g) => g.favorite));

	let filteredGames = $derived.by(() => {
		const term = searchTerm.toLowerCase().trim();
		if (!term) return games.list;
		return games.list.filter((g) => g.name.toLowerCase().includes(term));
	});

	// Group by mod loader so the long flat list is broken up visually
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
		e.preventDefault();
		await api.profile.favoriteGame(slug);
		await games.refresh();
	}

	let launching = $state(false);
	async function launch() {
		if (launching) return;
		launching = true;
		try {
			await launchGameWithBepInExFallback();
		} finally {
			launching = false;
		}
	}
</script>

<div class="z-dashboard">
	<Header title={i18nState.locale && m.dashboard_title()}>
		{#snippet actions()}
			<Button
				variant="accent"
				onclick={launch}
				disabled={!games.active || launching}
				loading={launching}
			>
				{#snippet icon()}<Icon icon="mdi:rocket-launch" />{/snippet}
				{i18nState.locale && m.dashboard_launch({ name: games.active?.name ?? 'Game' })}
			</Button>
		{/snippet}
	</Header>

	<div class="z-dashboard-content">
		<DashboardHero />
		<DashboardStats />

		{#if favorites.length > 0}
			<section class="z-section">
				<div class="z-section-header">
					<h3>{i18nState.locale && m.dashboard_favoritesTitle()}</h3>
					<Badge variant="accent">{favorites.length}</Badge>
				</div>
				<div class="z-games-grid z-games-grid-hero">
					{#each favorites as game (game.slug)}
						<GameTile
							{game}
							isActive={game.slug === games.active?.slug}
							onselect={selectGame}
							ontoggleFav={toggleFavorite}
						/>
					{/each}
				</div>
			</section>
		{/if}

		<section class="z-section z-section-flex">
			<div class="z-section-header">
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
								<GameCard
									{game}
									isActive={game.slug === games.active?.slug}
									onselect={selectGame}
									ontoggleFav={toggleFavorite}
								/>
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

	.z-games-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
		gap: var(--space-md);
	}

	.z-games-grid-hero {
		grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
		gap: var(--space-md);
	}
</style>
