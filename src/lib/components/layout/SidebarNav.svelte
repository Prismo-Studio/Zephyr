<script lang="ts">
	import Icon from '@iconify/svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import { onMount } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type NavItem = { path: string; icon: string; label: () => string };

	const navItems: NavItem[] = [
		{ path: '/dashboard', icon: 'mdi:view-dashboard', label: () => m.navBar_label_home() },
		{ path: '/', icon: 'mdi:package-variant', label: () => m.navBar_label_mods() },
		{ path: '/browse', icon: 'mdi:store-search', label: () => m.navBar_label_browse() },
		{ path: '/profiles', icon: 'mdi:account-group', label: () => m.menuBar_profile_title() },
		{ path: '/config', icon: 'mdi:cog', label: () => m.navBar_label_config() },
		{ path: '/randomizer', icon: 'mdi:dice-multiple', label: () => m.randomizer_title() },
		{ path: '/prefs', icon: 'mdi:tune-vertical', label: () => m.navBar_label_settings() }
	];

	let currentPath = $state(window.location.pathname);

	// Mirror the routing state by watching popstate AND patching pushState — SvelteKit's
	// client navigation goes through history.pushState without firing a popstate event.
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
</script>

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

<style>
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
</style>
