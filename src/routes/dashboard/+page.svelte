<script lang="ts">
	import Header from '$lib/components/layout/Header.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Icon from '@iconify/svelte';

	import games from '$lib/state/game.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { gameIconSrc } from '$lib/util';
	import * as api from '$lib/api';

	let favorites = $derived(games.list.filter((g) => g.favorite));
	let recentGames = $derived(games.list.slice(0, 8));

	async function selectGame(slug: string) {
		await games.setActive(slug);
	}

	async function launch() {
		await api.profile.launch.launchGame();
	}
</script>

<div class="z-dashboard">
	<Header title="Dashboard">
		{#snippet actions()}
			{#if games.active}
				<Button variant="primary" onclick={launch}>
					{#snippet icon()}<Icon icon="mdi:rocket-launch" />{/snippet}
					Launch {games.active.name}
				</Button>
			{/if}
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
							<span>View mods</span>
						</a>
						<a href="/browse" class="z-hero-action">
							<Icon icon="mdi:store-search" />
							<span>Browse</span>
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
					<span class="z-stat-label">Games</span>
				</div>
			</div>
			<div class="z-stat-card">
				<div class="z-stat-icon"><Icon icon="mdi:account-group" /></div>
				<div class="z-stat-info">
					<span class="z-stat-value">{profiles.list.length}</span>
					<span class="z-stat-label">Profiles</span>
				</div>
			</div>
			<div class="z-stat-card">
				<div class="z-stat-icon"><Icon icon="mdi:package-variant" /></div>
				<div class="z-stat-info">
					<span class="z-stat-value">{profiles.active?.modCount ?? 0}</span>
					<span class="z-stat-label">Mods installed</span>
				</div>
			</div>
		</div>

		<!-- Games grid -->
		<section class="z-section">
			<div class="z-section-header">
				<h3>Your Games</h3>
				{#if favorites.length > 0}
					<Badge variant="accent">{favorites.length} favorites</Badge>
				{/if}
			</div>

			<div class="z-games-grid">
				{#each recentGames as game (game.slug)}
					<button
						class="z-game-card"
						class:active={game.slug === games.active?.slug}
						onclick={() => selectGame(game.slug)}
					>
						<img src={gameIconSrc(game)} alt={game.name} class="z-game-card-img" />
						<div class="z-game-card-info">
							<span class="z-game-card-name">{game.name}</span>
							<span class="z-game-card-loader">{game.modLoader}</span>
						</div>
						{#if game.favorite}
							<Icon icon="mdi:star" class="z-game-star" />
						{/if}
						{#if game.slug === games.active?.slug}
							<span class="z-game-active-dot"></span>
						{/if}
					</button>
				{/each}
			</div>
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
		padding: 0 var(--space-xl) var(--space-xl);
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
		background: linear-gradient(135deg, rgba(0,0,0,0.4), rgba(0,0,0,0.1));
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
		color: var(--text-secondary);
	}

	.z-hero-mods { color: var(--text-muted); }

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
		background: rgba(255,255,255,0.1);
		color: var(--text-primary);
		text-decoration: none;
		font-size: 13px;
		font-weight: 600;
		transition: all var(--transition-fast);
		backdrop-filter: blur(8px);
	}

	.z-hero-action:hover {
		background: rgba(255,255,255,0.18);
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
		width: 40px;
		height: 40px;
		border-radius: var(--radius-md);
		background: var(--bg-active);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 20px;
		color: var(--text-accent);
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
		box-shadow: 0 0 6px rgba(26, 255, 250, 0.5);
	}
</style>
