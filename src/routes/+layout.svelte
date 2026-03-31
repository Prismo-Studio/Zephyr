<script lang="ts">
	import '../app.css';

	import Titlebar from '$lib/components/layout/Titlebar.svelte';
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import Statusbar from '$lib/components/layout/Statusbar.svelte';
	import Toasts from '$lib/components/ui/Toasts.svelte';
	import InstallPopover from '$lib/components/toolbar/InstallPopover.svelte';
	import InstallModDialog from '$lib/components/dialogs/InstallModDialog.svelte';
	import WelcomeDialog from '$lib/components/dialogs/WelcomeDialog.svelte';
	import ImportProfileDialog from '$lib/components/dialogs/ImportProfileDialog.svelte';

	import { onMount, type Snippet } from 'svelte';
	import { refreshColor, refreshFont } from '$lib/theme';
	import { initTheme } from '$lib/design-system/tokens';
	import profiles from '$lib/state/profile.svelte';
	import { updateBanner } from '$lib/state/misc.svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import type { ProfileInfo, ManagedGameInfo } from '$lib/types';
	import { refreshLanguage } from '$lib/i18n';

	type Props = {
		children?: Snippet;
	};

	let { children }: Props = $props();

	let unlistenProfiles: UnlistenFn | null;
	let unlistenGames: UnlistenFn | null;

	onMount(() => {
		initTheme();
		refreshFont();
		refreshColor('accent');
		refreshColor('primary');
		refreshLanguage();

		$effect(() => {
			profiles.active;
			updateBanner.threshold = 0;
		});

		listen<ProfileInfo>('profile_changed', (evt) => {
			profiles.updateOne(evt.payload);
		}).then((callback) => (unlistenProfiles = callback));

		listen<ManagedGameInfo>('game_changed', (evt) => {
			profiles.update(evt.payload);
		}).then((callback) => (unlistenGames = callback));

		return () => {
			unlistenProfiles?.();
			unlistenGames?.();
		};
	});
</script>

<svelte:body
	oncontextmenu={(evt) => {
		if (window.location.hostname === 'tauri.localhost') {
			evt.preventDefault();
		}
	}}
/>

<main class="z-app">
	<Titlebar />

	<div class="z-app-body">
		<Sidebar />

		<div class="z-main">
			<div class="z-content">
				{@render children?.()}
			</div>
			<Statusbar />
		</div>
	</div>

	<Toasts />
	<InstallPopover />
	<InstallModDialog />
	<WelcomeDialog />
	<ImportProfileDialog />
</main>

<style>
	.z-app {
		display: flex;
		flex-direction: column;
		height: 100vh;
		width: 100vw;
		overflow: hidden;
		background: var(--bg-base);
		color: var(--text-primary);
	}

	.z-app-body {
		display: flex;
		flex: 1;
		min-height: 0;
		overflow: hidden;
	}

	.z-main {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-width: 0;
		overflow: hidden;
	}

	.z-content {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
	}
</style>
