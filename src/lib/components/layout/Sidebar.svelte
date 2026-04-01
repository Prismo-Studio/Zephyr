<script lang="ts">
	import Icon from '@iconify/svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import games from '$lib/state/game.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { gameIconSrc } from '$lib/util';
	import * as api from '$lib/api';
	import { onMount } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type NavItem = {
		path: string;
		icon: string;
		label: () => string;
	};

	const navItems: NavItem[] = [
		{ path: '/dashboard', icon: 'mdi:view-dashboard', label: () => m.navBar_label_home() },
		{ path: '/', icon: 'mdi:package-variant', label: () => m.navBar_label_mods() },
		{ path: '/browse', icon: 'mdi:store-search', label: () => m.navBar_label_browse() },
		{ path: '/profiles', icon: 'mdi:account-group', label: () => m.menuBar_profile_title() },
		{ path: '/config', icon: 'mdi:cog', label: () => m.navBar_label_config() },
		{ path: '/prefs', icon: 'mdi:tune-vertical', label: () => m.navBar_label_settings() }
	];

	let currentPath = $state(window.location.pathname);

	onMount(() => {
		const update = () => {
			currentPath = window.location.pathname;
		};
		window.addEventListener('popstate', update);
		const origPush = history.pushState.bind(history);
		history.pushState = function (data: any, unused: string, url?: string | URL | null) {
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
	let profileMenuOpen = $state(false);

	async function launchGame() {
		await api.profile.launch.launchGame();
	}

	async function switchProfile(id: number) {
		if (id === profiles.activeId) return;
		const index = profiles.list.findIndex((p) => p.id === id);
		if (index === -1) return;
		await profiles.setActive(index);
		await profiles.refresh();
		profileMenuOpen = false;
	}

	function handleProfileKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') profileMenuOpen = false;
	}
</script>

<aside class="z-sidebar">
	<!-- Game icon / selector -->
	<div class="z-sidebar-game">
		{#if games.active}
			<Tooltip text={games.active.name} position="right" delay={300}>
				<button class="z-game-btn" onclick={() => (gameMenuOpen = !gameMenuOpen)}>
					<img src={gameIconSrc(games.active)} alt={games.active.name} class="z-game-icon" />
				</button>
			</Tooltip>
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
						<Icon icon="mdi:star" class="text-xs text-amber-400" />
					{/if}
				</button>
			{/each}
		</div>
	{/if}

	<!-- Navigation -->
	<nav class="z-sidebar-nav">
		{#each navItems as item}
			<Tooltip text={item.label()} position="right" delay={300}>
				<a href={item.path} class="z-nav-item" class:active={isActive(item.path)}>
					<Icon icon={item.icon} class="z-nav-icon" />
					<span class="z-nav-label">{item.label()}</span>
					{#if isActive(item.path)}
						<span class="z-nav-indicator"></span>
					{/if}
				</a>
			</Tooltip>
		{/each}
	</nav>

	<!-- Bottom section -->
	<div class="z-sidebar-bottom">
		<Tooltip text={i18nState.locale && m.toolBar_launchGame_button()} position="right" delay={300}>
			<button class="z-launch-btn" onclick={launchGame}>
				<Icon icon="mdi:rocket-launch" />
			</button>
		</Tooltip>

		{#if profiles.active}
			<!-- Profile switcher -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="z-profile-wrapper" onkeydown={handleProfileKeydown}>
				<button
					class="z-sidebar-profile"
					class:open={profileMenuOpen}
					onclick={() => (profileMenuOpen = !profileMenuOpen)}
					aria-label="Switch profile"
					aria-expanded={profileMenuOpen}
				>
					<div class="z-profile-avatar">
						<Icon icon="mdi:account-circle" class="z-profile-icon" />
					</div>
					<span class="z-profile-name">{profiles.active.name}</span>
					<Icon icon="mdi:chevron-up" class="z-profile-chevron" />
				</button>

				{#if profileMenuOpen}
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div class="z-profile-dropdown" role="menu">
						<div class="z-profile-dropdown-header">
							<Icon icon="mdi:account-switch" />
							<span>Switch Profile</span>
						</div>
						<div class="z-profile-list">
							{#each profiles.list as profile}
								<button
									class="z-profile-item"
									class:active={profile.id === profiles.activeId}
									onclick={() => switchProfile(profile.id)}
									role="menuitem"
								>
									<div class="z-profile-item-icon">
										{#if profile.id === profiles.activeId}
											<Icon icon="mdi:check-circle" class="z-profile-check" />
										{:else}
											<Icon icon="mdi:account-circle-outline" />
										{/if}
									</div>
									<span class="z-profile-item-name">{profile.name}</span>
									{#if profile.id === profiles.activeId}
										<span class="z-profile-active-badge">Active</span>
									{/if}
								</button>
							{/each}
						</div>
					</div>

					<!-- Backdrop to close on outside click -->
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div class="z-profile-backdrop" onclick={() => (profileMenuOpen = false)}></div>
				{/if}
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
		padding: 0;
		flex: 1;
	}

	.z-nav-item {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 3px;
		padding: var(--space-sm) 0;
		border-radius: 0;
		color: var(--text-muted);
		text-decoration: none;
		transition: all var(--transition-fast);
		position: relative;
		width: 100%;
	}

	.z-nav-item:hover {
		color: var(--text-secondary);
		background: var(--bg-hover);
	}

	.z-nav-item.active {
		color: #1afffa;
		background: rgba(26, 255, 250, 0.15);
	}

	:global(.z-nav-icon) {
		font-size: 20px;
	}

	.z-nav-item.active :global(.z-nav-icon) {
		color: #1afffa;
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

	/* Profile switcher */
	.z-profile-wrapper {
		position: relative;
		width: 100%;
	}

	.z-sidebar-profile {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 2px;
		width: 100%;
		padding: var(--space-xs) 2px;
		border: 1px solid transparent;
		border-radius: var(--radius-md);
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
		font-family: var(--font-body);
	}

	.z-sidebar-profile:hover,
	.z-sidebar-profile.open {
		background: var(--bg-hover);
		border-color: var(--border-default);
		color: var(--text-secondary);
	}

	.z-sidebar-profile.open {
		border-color: var(--accent-400);
		color: var(--accent-400);
	}

	.z-profile-avatar {
		width: 24px;
		height: 24px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	:global(.z-profile-icon) {
		font-size: 18px;
	}

	.z-profile-name {
		font-size: 9px;
		font-weight: 600;
		letter-spacing: 0.03em;
		text-transform: uppercase;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 56px;
	}

	:global(.z-profile-chevron) {
		font-size: 12px;
		transition: transform var(--transition-fast);
	}

	.z-sidebar-profile.open :global(.z-profile-chevron) {
		transform: rotate(180deg);
	}

	/* Profile dropdown */
	.z-profile-backdrop {
		position: fixed;
		inset: 0;
		z-index: calc(var(--z-dropdown) - 1);
	}

	.z-profile-dropdown {
		position: absolute;
		bottom: calc(100% + 8px);
		left: 72px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		padding: var(--space-xs);
		min-width: 200px;
		max-height: 320px;
		overflow-y: auto;
		z-index: var(--z-dropdown);
		box-shadow:
			var(--shadow-lg),
			0 0 20px rgba(26, 255, 250, 0.08);
		animation: slideUp 150ms ease;
	}

	@keyframes slideUp {
		from {
			opacity: 0;
			transform: translateY(6px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.z-profile-dropdown-header {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		padding: var(--space-xs) var(--space-sm);
		color: var(--text-muted);
		font-size: 10px;
		font-weight: 700;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		border-bottom: 1px solid var(--border-subtle);
		padding-bottom: var(--space-sm);
		margin-bottom: var(--space-xs);
	}

	.z-profile-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
		max-height: 240px;
		overflow-y: auto;
	}

	.z-profile-item {
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
		text-align: left;
	}

	.z-profile-item:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.z-profile-item.active {
		background: rgba(26, 255, 250, 0.1);
		color: var(--accent-400);
	}

	.z-profile-item-icon {
		font-size: 16px;
		flex-shrink: 0;
		display: flex;
		align-items: center;
	}

	:global(.z-profile-check) {
		color: var(--accent-400);
	}

	.z-profile-item-name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.z-profile-active-badge {
		font-size: 9px;
		font-weight: 700;
		letter-spacing: 0.05em;
		text-transform: uppercase;
		color: var(--accent-400);
		background: rgba(26, 255, 250, 0.12);
		padding: 2px 6px;
		border-radius: var(--radius-full);
		flex-shrink: 0;
	}
</style>
