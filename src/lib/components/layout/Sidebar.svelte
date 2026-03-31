<script lang="ts">
	import Icon from '@iconify/svelte';
	import games from '$lib/state/game.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { gameIconSrc } from '$lib/util';
	import * as api from '$lib/api';
	import { onMount } from 'svelte';

	type NavItem = {
		path: string;
		icon: string;
		label: string;
	};

	const navItems: NavItem[] = [
		{ path: '/dashboard', icon: 'mdi:view-dashboard', label: 'Dashboard' },
		{ path: '/', icon: 'mdi:package-variant', label: 'Mods' },
		{ path: '/browse', icon: 'mdi:store-search', label: 'Browse' },
		{ path: '/profiles', icon: 'mdi:account-group', label: 'Profiles' },
		{ path: '/config', icon: 'mdi:cog', label: 'Config' },
		{ path: '/prefs', icon: 'mdi:tune-vertical', label: 'Settings' }
	];

	let currentPath = $state(window.location.pathname);

	onMount(() => {
		const update = () => { currentPath = window.location.pathname; };
		window.addEventListener('popstate', update);
		// SvelteKit uses history.pushState — observe navigation
		const origPush = history.pushState.bind(history);
		history.pushState = function(data: any, unused: string, url?: string | URL | null) {
			origPush(data, unused, url);
			currentPath = window.location.pathname;
		};
		return () => {
			window.removeEventListener('popstate', update);
			history.pushState = origPush;
		};
	});

	function isActive(path: string): boolean {
		if (path === '/') return currentPath === '/';
		return currentPath.startsWith(path);
	}

	let gameMenuOpen = $state(false);

	async function launchGame() {
		await api.profile.launch.launchGame();
	}
</script>

<aside class="z-sidebar">
	<!-- Game icon / selector -->
	<div class="z-sidebar-game">
		{#if games.active}
			<button
				class="z-game-btn"
				onclick={() => (gameMenuOpen = !gameMenuOpen)}
				title={games.active.name}
			>
				<img
					src={gameIconSrc(games.active)}
					alt={games.active.name}
					class="z-game-icon"
				/>
			</button>
		{:else}
			<div class="z-game-btn z-game-placeholder">
				<Icon icon="mdi:gamepad-variant" />
			</div>
		{/if}
	</div>

	<!-- Game selector dropdown -->
	{#if gameMenuOpen && games.list.length > 0}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="z-game-dropdown" onmouseleave={() => (gameMenuOpen = false)}>
			{#each games.list as game}
				<button
					class="z-game-dropdown-item"
					class:active={game.slug === games.active?.slug}
					onclick={async () => {
						await games.setActive(game.slug);
						gameMenuOpen = false;
					}}
				>
					<img src={gameIconSrc(game)} alt={game.name} class="z-game-dropdown-icon" />
					<span class="z-game-dropdown-name">{game.name}</span>
					{#if game.favorite}
						<Icon icon="mdi:star" class="text-amber-400 text-xs" />
					{/if}
				</button>
			{/each}
		</div>
	{/if}

	<!-- Navigation -->
	<nav class="z-sidebar-nav">
		{#each navItems as item}
			<a
				href={item.path}
				class="z-nav-item"
				class:active={isActive(item.path)}
				title={item.label}
			>
				<Icon icon={item.icon} class="z-nav-icon" />
				<span class="z-nav-label">{item.label}</span>
				{#if isActive(item.path)}
					<span class="z-nav-indicator"></span>
				{/if}
			</a>
		{/each}
	</nav>

	<!-- Bottom section -->
	<div class="z-sidebar-bottom">
		<!-- Launch button -->
		<button class="z-launch-btn" onclick={launchGame} title="Launch Game">
			<Icon icon="mdi:rocket-launch" />
		</button>

		<!-- Profile name -->
		{#if profiles.active}
			<div class="z-sidebar-profile" title={profiles.active.name}>
				<Icon icon="mdi:account-circle" class="text-xs" />
				<span>{profiles.active.name}</span>
			</div>
		{/if}
	</div>
</aside>

<style>
	.z-sidebar {
		display: flex;
		flex-direction: column;
		width: 68px;
		min-width: 68px;
		height: 100%;
		background: var(--bg-surface);
		border-right: 1px solid var(--border-subtle);
		padding: var(--space-md) 0;
		gap: var(--space-sm);
		position: relative;
		z-index: var(--z-sticky);
	}

	/* Game selector */
	.z-sidebar-game {
		display: flex;
		justify-content: center;
		padding: 0 var(--space-md);
		margin-bottom: var(--space-sm);
	}

	.z-game-btn {
		width: 44px;
		height: 44px;
		border-radius: var(--radius-lg);
		border: 2px solid var(--border-default);
		background: var(--bg-elevated);
		cursor: pointer;
		overflow: hidden;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all var(--transition-fast);
		color: var(--text-muted);
		font-size: 20px;
	}

	.z-game-btn:hover {
		border-color: var(--accent-400);
		box-shadow: var(--shadow-glow);
	}

	.z-game-icon {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	/* Game dropdown */
	.z-game-dropdown {
		position: absolute;
		top: var(--space-md);
		left: 72px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		padding: var(--space-xs);
		min-width: 220px;
		max-height: 400px;
		overflow-y: auto;
		z-index: var(--z-dropdown);
		box-shadow: var(--shadow-lg);
		animation: scaleIn var(--transition-fast) ease;
	}

	.z-game-dropdown-item {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		width: 100%;
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-md);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		transition: all var(--transition-fast);
		font-family: var(--font-body);
		font-size: 13px;
	}

	.z-game-dropdown-item:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.z-game-dropdown-item.active {
		background: var(--bg-active);
		color: var(--text-accent);
	}

	.z-game-dropdown-icon {
		width: 28px;
		height: 28px;
		border-radius: var(--radius-sm);
		object-fit: cover;
		flex-shrink: 0;
	}

	.z-game-dropdown-name {
		flex: 1;
		text-align: left;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	/* Navigation */
	.z-sidebar-nav {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 0 var(--space-sm);
		flex: 1;
	}

	.z-nav-item {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 3px;
		padding: var(--space-sm) 0;
		border-radius: var(--radius-md);
		color: var(--text-muted);
		text-decoration: none;
		transition: all var(--transition-fast);
		position: relative;
	}

	.z-nav-item:hover {
		color: var(--text-secondary);
		background: var(--bg-hover);
	}

	.z-nav-item.active {
		color: var(--text-accent);
		background: var(--bg-active);
	}

	:global(.z-nav-icon) {
		font-size: 20px;
	}

	.z-nav-label {
		font-size: 9px;
		font-weight: 600;
		letter-spacing: 0.03em;
		text-transform: uppercase;
	}

	.z-nav-indicator {
		position: absolute;
		left: -8px;
		top: 50%;
		transform: translateY(-50%);
		width: 3px;
		height: 20px;
		background: var(--accent-400);
		border-radius: 0 var(--radius-full) var(--radius-full) 0;
		box-shadow: 0 0 8px rgba(26, 255, 250, 0.4);
	}

	/* Bottom */
	.z-sidebar-bottom {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-sm);
		padding: 0 var(--space-sm);
		margin-top: auto;
	}

	.z-launch-btn {
		width: 44px;
		height: 44px;
		border-radius: var(--radius-lg);
		background: linear-gradient(135deg, var(--accent-400), var(--accent-600));
		border: none;
		color: var(--text-inverse);
		font-size: 20px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all var(--transition-normal);
		box-shadow: 0 0 12px rgba(26, 255, 250, 0.2);
	}

	.z-launch-btn:hover {
		box-shadow: 0 0 28px rgba(26, 255, 250, 0.4);
		transform: scale(1.05);
	}

	.z-launch-btn:active {
		transform: scale(0.97);
	}

	.z-sidebar-profile {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 2px;
		font-size: 9px;
		color: var(--text-muted);
		text-align: center;
		padding: var(--space-xs);
		max-width: 100%;
		overflow: hidden;
	}

	.z-sidebar-profile span {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 56px;
	}
</style>
