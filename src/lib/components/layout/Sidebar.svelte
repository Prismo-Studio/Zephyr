<script lang="ts">
	const { legendActive = false } = $props<{ legendActive?: boolean }>();
	import Icon from '@iconify/svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import VanillaFlowerIcon from '$lib/components/ui/VanillaFlowerIcon.svelte';
	import games from '$lib/state/game.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { gameIconSrc } from '$lib/util';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import LaunchOverlay from '$lib/components/dialogs/LaunchOverlay.svelte';
	import * as api from '$lib/api';
	import { installState } from '$lib/state/misc.svelte';
	import { open } from '@tauri-apps/plugin-shell';
	// import { activeSourceState } from '$lib/state/source.svelte';
	// import type { SourceGame } from '$lib/api/sources';

	// let nexusGames: SourceGame[] = $state([]);
	// let nexusGamesLoaded = $state(false);

	// $effect(() => {
	// 	if (activeSourceState.current === 'nexusmods' && !nexusGamesLoaded) {
	// 		api.sources
	// 			.getNexusmodsGames()
	// 			.then((g) => {
	// 				nexusGames = g;
	// 				nexusGamesLoaded = true;
	// 			})
	// 			.catch(() => {});
	// 	}
	// });
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
	let gameSearchTerm = $state('');
	let gameDropdownEl: HTMLDivElement | null = null;
	let gameSearchInput: HTMLInputElement | null = null;

	$effect(() => {
		if (gameMenuOpen && gameSearchInput) {
			gameSearchInput.focus();
		}
	});

	function handleGameDropdownClickOutside(e: MouseEvent) {
		if (gameMenuOpen && gameDropdownEl && !gameDropdownEl.contains(e.target as Node)) {
			gameMenuOpen = false;
			gameSearchTerm = '';
		}
	}

	$effect(() => {
		if (gameMenuOpen) {
			document.addEventListener('mousedown', handleGameDropdownClickOutside, true);
		} else {
			document.removeEventListener('mousedown', handleGameDropdownClickOutside, true);
		}
		return () => {
			document.removeEventListener('mousedown', handleGameDropdownClickOutside, true);
		};
	});
	let profileMenuOpen = $state(false);

	let launching = $state(false);

	function waitForInstallEnd(): Promise<void> {
		return new Promise((resolve) => {
			const check = () => {
				if (!installState.active) return resolve();
				setTimeout(check, 300);
			};
			setTimeout(check, 500);
		});
	}

	async function installBepInEx(): Promise<boolean> {
		try {
			const results = await api.thunderstore.query({
				searchTerm: 'BepInExPack',
				includeCategories: [],
				excludeCategories: [],
				includeNsfw: false,
				includeDeprecated: false,
				includeDisabled: true,
				includeEnabled: true,
				sortBy: 'downloads',
				sortOrder: 'descending',
				maxCount: 5
			});
			const bepinex = results.find(
				(m) => m.name === 'BepInExPack' || m.name.startsWith('BepInExPack')
			);
			if (bepinex && !bepinex.isInstalled) {
				await api.profile.install.mod({
					packageUuid: bepinex.uuid,
					versionUuid: bepinex.versions[0].uuid
				});
				await waitForInstallEnd();
				return true;
			}
		} catch {}
		return false;
	}

	async function launchGame() {
		launching = true;
		try {
			await api.profile.launch.launchGameSilent();
		} catch (err: any) {
			const msg = String(err).toLowerCase();
			if (msg.includes('bepinex') || msg.includes('preloader not found')) {
				const installed = await installBepInEx();
				if (installed) {
					try {
						await api.profile.launch.launchGame();
						return;
					} catch {}
				}
				launching = false;
				return;
			}
			// Not a BepInEx error — show the toast via normal launch path
			try {
				await api.profile.launch.launchGame();
			} catch {
				launching = false;
			}
		}
	}

	async function launchVanilla() {
		launching = true;
		try {
			await api.profile.launch.launchVanilla();
		} catch {
			launching = false;
		}
	}

	async function switchProfile(id: number) {
		if (id === profiles.activeId) return;
		const index = profiles.list.findIndex((p) => p.id === id);
		if (index === -1) return;
		await profiles.setActive(index);
		await profiles.refresh();
		profileMenuOpen = false;
	}

	function profileIconSrc(icon: string | null): string | null {
		if (!icon) return null;
		if (icon.startsWith('http')) return icon;
		return convertFileSrc(icon);
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
		<div class="z-game-dropdown" bind:this={gameDropdownEl}>
			<div class="z-game-dropdown-search">
				<Icon icon="mdi:magnify" />
				<input
					type="text"
					placeholder={i18nState.locale && m.sidebar_searchGames()}
					bind:value={gameSearchTerm}
					bind:this={gameSearchInput}
					class="z-game-search-input"
					onkeydown={(e) => {
						if (e.key === 'Escape') {
							gameMenuOpen = false;
							gameSearchTerm = '';
						}
					}}
				/>
			</div>
			<div class="z-game-dropdown-list">
				<!-- NexusMods game list commented out
				{#if activeSourceState.current === 'nexusmods'}
					{#each nexusGames.filter((g) => g.name
							.toLowerCase()
							.includes(gameSearchTerm.toLowerCase())) as nxGame}
						<button
							class="z-game-dropdown-item"
							class:active={games.active?.slug === nxGame.slug}
							onclick={async () => {
								const match = games.list.find((g) => g.slug === nxGame.slug);
								if (match) {
									await games.setActive(match.slug);
								}
								gameMenuOpen = false;
								gameSearchTerm = '';
							}}
						>
							<div class="z-game-dropdown-icon z-nx-game-icon">
								<Icon icon="mdi:gamepad-variant" />
							</div>
							<span class="z-game-dropdown-name">{nxGame.name}</span>
							<span class="z-game-dropdown-count">{nxGame.modCount}</span>
						</button>
					{/each}
					{#if nexusGames.filter((g) => g.name
							.toLowerCase()
							.includes(gameSearchTerm.toLowerCase())).length === 0}
						<div class="z-game-dropdown-empty">
							<Icon icon="mdi:magnify" />
							<span>{i18nState.locale && m.sidebar_noGamesFound()}</span>
						</div>
					{/if}
				{:else} -->
				{#each games.list.filter((g) => g.name
						.toLowerCase()
						.includes(gameSearchTerm.toLowerCase())) as game}
					<button
						class="z-game-dropdown-item"
						class:active={game.slug === games.active?.slug}
						onclick={async () => {
							await games.setActive(game.slug);
							gameMenuOpen = false;
							gameSearchTerm = '';
						}}
					>
						<img src={gameIconSrc(game)} alt={game.name} class="z-game-dropdown-icon" />
						<span class="z-game-dropdown-name">{game.name}</span>
						{#if game.favorite}
							<Icon icon="mdi:star" class="text-xs text-amber-400" />
						{/if}
					</button>
				{/each}
				{#if games.list.filter((g) => g.name
						.toLowerCase()
						.includes(gameSearchTerm.toLowerCase())).length === 0}
					<div class="z-game-dropdown-empty">
						<Icon icon="mdi:magnify" />
						<span>{i18nState.locale && m.sidebar_noGamesFound()}</span>
					</div>
				{/if}
				<!-- {/if} NexusMods conditional end -->
			</div>
		</div>
	{/if}

	<!-- Navigation -->
	<nav class="z-sidebar-nav">
		{#each navItems as item}
			<Tooltip text={i18nState.locale && item.label()} position="right" delay={300}>
				<a href={item.path} class="z-nav-item" class:active={isActive(item.path)}>
					<Icon icon={item.icon} class="z-nav-icon" />
					<span class="z-nav-label">{i18nState.locale && item.label()}</span>
					{#if isActive(item.path)}
						<span class="z-nav-indicator"></span>
					{/if}
				</a>
			</Tooltip>
		{/each}
	</nav>

	<!-- Bottom section -->
	<div class="z-sidebar-bottom" class:legend-active={legendActive}>
		<Tooltip text={i18nState.locale && m.toolBar_launchGame_button()} position="right" delay={300}>
			<button class="z-launch-btn" onclick={launchGame}>
				<svg
					width="28"
					height="28"
					viewBox="0 0 24 24"
					fill="currentColor"
					class="z-custom-launch-icon"
				>
					<path d="M8 5v14l11-7z" />
				</svg>
			</button>
		</Tooltip>

		<Tooltip text={i18nState.locale && m.sidebar_launchVanilla()} position="right" delay={300}>
			<button class="z-launch-vanilla-btn" onclick={launchVanilla}>
				<VanillaFlowerIcon />
			</button>
		</Tooltip>

		<!-- Source toggle — disabled for v1 -->

		{#if profiles.active}
			<!-- Profile switcher -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="z-profile-wrapper" onkeydown={handleProfileKeydown}>
				<Tooltip text={profiles.active.name} position="right" delay={300}>
					<button
						class="z-sidebar-profile"
						class:open={profileMenuOpen}
						onclick={() => (profileMenuOpen = !profileMenuOpen)}
						aria-label={i18nState.locale && m.sidebar_switchProfile()}
						aria-expanded={profileMenuOpen}
					>
						{#if profiles.active.icon}
							<img
								src={profileIconSrc(profiles.active.icon)}
								alt={profiles.active.name}
								class="z-profile-img"
							/>
						{:else}
							<Icon icon="mdi:account-circle" class="z-profile-avatar-icon" />
						{/if}
					</button>
				</Tooltip>

				{#if profileMenuOpen}
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div class="z-profile-dropdown" role="menu">
						<div class="z-profile-dropdown-header">
							<Icon icon="mdi:account-switch" />
							<span>{m.sidebar_switchProfile()}</span>
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
										{#if profile.icon}
											<img
												src={profileIconSrc(profile.icon)}
												alt={profile.name}
												class="z-profile-item-img"
											/>
										{:else}
											<Icon icon="mdi:account-circle" />
										{/if}
									</div>
									<span class="z-profile-item-name">{profile.name}</span>
									{#if profile.id === profiles.activeId}
										<Icon icon="mdi:check-circle" class="z-profile-check" />
										<span class="z-profile-active-badge">{m.sidebar_activeProfile()}</span>
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

<LaunchOverlay
	bind:visible={launching}
	onclose={() => {
		launching = false;
	}}
/>

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
		z-index: var(--z-tooltip);
	}

	/* Game selector */
	.z-source-toggle {
		width: 40px;
		height: 40px;
		border-radius: var(--radius-lg);
		border: 2px solid var(--border-default);
		background: var(--bg-elevated);
		cursor: pointer;
		padding: 0;
		overflow: hidden;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all var(--transition-fast);
	}

	.z-source-toggle:hover {
		border-color: var(--accent-400);
		box-shadow: var(--shadow-glow);
	}

	.z-source-toggle-img {
		width: 32px;
		height: 32px;
		object-fit: cover;
		border-radius: calc(var(--radius-lg) - 2px);
	}

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
		padding: 0;
		min-width: 240px;
		max-height: 500px;
		overflow: hidden;
		z-index: var(--z-dropdown);
		box-shadow: var(--shadow-lg);
		animation: scaleIn var(--transition-fast) ease;
		display: flex;
		flex-direction: column;
	}

	.z-game-dropdown-search {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-md);
		border-bottom: 1px solid var(--border-subtle);
		background: var(--bg-surface);
		color: var(--text-muted);
		font-size: 16px;
		flex-shrink: 0;
	}

	.z-game-search-input {
		flex: 1;
		background: transparent;
		border: none;
		color: var(--text-primary);
		font-size: 13px;
		font-family: var(--font-body);
		outline: none;
		padding: 0;
	}

	.z-game-search-input::placeholder {
		color: var(--text-muted);
	}

	.z-game-dropdown-list {
		overflow-y: auto;
		max-height: 420px;
		padding: var(--space-xs);
	}

	.z-game-dropdown-empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-sm);
		padding: var(--space-lg);
		color: var(--text-muted);
		font-size: 13px;
		text-align: center;
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

	.z-nx-game-icon {
		width: 28px;
		height: 28px;
		border-radius: var(--radius-sm);
		background: var(--bg-active);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-muted);
		font-size: 14px;
		flex-shrink: 0;
	}

	.z-game-dropdown-count {
		font-size: 10px;
		color: var(--text-muted);
		margin-left: auto;
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
		overflow-y: auto;
		min-height: 0;
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
		color: var(--text-accent);
		background: var(--bg-active);
	}

	:global(.z-nav-icon) {
		font-size: 20px;
	}

	.z-nav-item.active :global(.z-nav-icon) {
		color: var(--text-accent);
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
		box-shadow: var(--shadow-glow);
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

	/* Only add extra margin if gamepad legend is visible */
	.z-sidebar-bottom.legend-active {
		margin-bottom: 32px;
	}

	.z-launch-btn {
		width: 44px;
		height: 44px;
		border-radius: var(--radius-lg);
		background: transparent;
		border: 2px solid var(--accent-400);
		color: var(--accent-400);
		font-size: 22px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all var(--transition-normal);
	}

	.z-launch-btn:hover {
		background: var(--bg-active);
		transform: translateY(-2px);
		box-shadow: var(--shadow-glow);
	}

	.z-launch-btn:active {
		transform: scale(0.97);
	}

	.z-launch-vanilla-btn {
		width: 38px;
		height: 38px;
		border-radius: var(--radius-lg);
		background: transparent;
		border: 1.5px solid #f5d67b;
		color: #f5d67b;
		font-size: 20px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all var(--transition-normal);
	}

	.z-launch-vanilla-btn:hover {
		background: rgba(245, 214, 123, 0.1);
		transform: translateY(-2px);
		box-shadow: 0 4px 16px rgba(245, 214, 123, 0.15);
		color: #ffe4a0;
	}

	.z-launch-vanilla-btn:active {
		transform: scale(0.97);
	}

	.z-custom-launch-icon {
		width: 28px;
		height: 28px;
	}

	/* Profile switcher */
	.z-profile-wrapper {
		position: relative;
		display: flex;
		justify-content: center;
		width: 100%;
	}

	.z-sidebar-profile {
		width: 40px;
		height: 40px;
		border-radius: var(--radius-lg);
		border: 2px solid var(--border-default);
		background: var(--bg-elevated);
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all var(--transition-fast);
		color: var(--text-muted);
		padding: 0;
		overflow: hidden;
	}

	.z-sidebar-profile:hover {
		border-color: var(--accent-400);
		box-shadow: var(--shadow-glow);
		color: var(--text-secondary);
	}

	.z-sidebar-profile.open {
		border-color: var(--accent-400);
		box-shadow: var(--shadow-glow);
		color: var(--accent-400);
	}

	:global(.z-profile-avatar-icon) {
		font-size: 28px;
	}

	.z-profile-img {
		width: 32px;
		height: 32px;
		object-fit: cover;
		border-radius: calc(var(--radius-lg) - 2px);
	}

	/* Profile dropdown */
	.z-profile-backdrop {
		position: fixed;
		inset: 0;
		z-index: calc(var(--z-dropdown) - 1);
	}

	.z-profile-dropdown {
		position: absolute;
		bottom: 0;
		left: 72px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		padding: var(--space-xs);
		min-width: 200px;
		max-height: 320px;
		overflow-y: auto;
		z-index: var(--z-dropdown);
		box-shadow: var(--shadow-lg), var(--shadow-glow);
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
		background: var(--bg-active);
		color: var(--accent-400);
	}

	.z-profile-item-icon {
		width: 24px;
		height: 24px;
		font-size: 24px;
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-muted);
		overflow: hidden;
		border-radius: var(--radius-full);
	}

	.z-profile-item.active .z-profile-item-icon {
		color: var(--accent-400);
	}

	.z-profile-item-img {
		width: 24px;
		height: 24px;
		border-radius: var(--radius-full);
		object-fit: cover;
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
		background: var(--bg-active);
		padding: 2px 6px;
		border-radius: var(--radius-full);
		flex-shrink: 0;
	}
</style>
