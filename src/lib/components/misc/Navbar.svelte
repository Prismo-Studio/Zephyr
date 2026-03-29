<script lang="ts">
	import NavbarLink from './NavbarLink.svelte';
	import { m } from '$lib/paraglide/messages';

	let links = $derived([
		{
			to: '/dashboard',
			icon: 'mdi:home-variant-outline',
			label: m.navBar_label_home(),
			tooltip: m.navBar_link_home()
		},
		{
			to: '/',
			icon: 'mdi:view-dashboard',
			label: m.navBar_label_mods(),
			tooltip: m.navBar_link_profile()
		},
		{
			to: '/browse',
			icon: 'mdi:store-search-outline',
			label: m.navBar_label_browse(),
			tooltip: m.navBar_link_browse()
		},
		{
			to: '/config',
			icon: 'mdi:tune-variant',
			label: m.navBar_label_config(),
			tooltip: m.navBar_link_config()
		},
		{
			to: '/modpack',
			icon: 'mdi:package-variant-closed',
			label: m.navBar_label_export(),
			tooltip: m.navBar_link_modpack()
		},
		{
			to: '/prefs',
			icon: 'mdi:cog-outline',
			label: m.navBar_label_settings(),
			tooltip: m.navBar_link_prefs()
		}
	]);
</script>

<nav class="zephyr-sidebar relative flex w-[68px] shrink-0 flex-col items-center gap-1 py-3 px-2">
	<!-- Logo -->
	<div class="mb-3 flex items-center justify-center">
		<img src="logo.png" alt="Zephyr" class="size-9 rounded-lg" />
	</div>

	<div class="mb-2 h-px w-8 bg-[#1A2A42]"></div>

	{#each links as link (link.to)}
		<NavbarLink {...link} />
	{/each}

	<div class="grow"></div>

	<!-- Version badge -->
	<div class="text-[10px] font-medium text-[#556677] select-none">v0.1</div>
</nav>

<style>
	.zephyr-sidebar {
		background: linear-gradient(180deg, #080F1C 0%, #0A1424 100%);
		border-right: 1px solid #1A2A42;
	}
</style>
