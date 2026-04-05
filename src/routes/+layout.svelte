<script lang="ts">
	import '../app.css';

	import Titlebar from '$lib/components/layout/Titlebar.svelte';
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import Statusbar from '$lib/components/layout/Statusbar.svelte';
	import Toasts from '$lib/components/ui/Toasts.svelte';
	import InstallPopover from '$lib/components/toolbar/InstallPopover.svelte';
	import InstallModDialog from '$lib/components/dialogs/InstallModDialog.svelte';
	import ImportProfileDialog from '$lib/components/dialogs/ImportProfileDialog.svelte';

	import { onMount, type Snippet } from 'svelte';
	import { refreshColor, refreshFont } from '$lib/themeSystem';
	import { initTheme } from '$lib/design-system/tokens';
	import profiles from '$lib/state/profile.svelte';
	import games from '$lib/state/game.svelte';
	import auth from '$lib/state/auth.svelte';
	import { updateBanner } from '$lib/state/misc.svelte';
	import updates from '$lib/state/update.svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { PersistedState } from 'runed';
	import type { ProfileInfo, ManagedGameInfo } from '$lib/types';
	import { updateAppLanguage, i18nState } from '$lib/i18nCore.svelte';
	import { getLocale, locales, type Locale } from '$lib/paraglide/runtime';
	import * as api from '$lib/api';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { initErrorListener } from '$lib/invoke';
	import { open } from '@tauri-apps/plugin-shell';
	import { relaunch } from '@tauri-apps/plugin-process';
	import { getVersion } from '@tauri-apps/api/app';
	import { pushToast, pushInfoToast } from '$lib/toast';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Icon from '@iconify/svelte';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		children?: Snippet;
	};

	let { children }: Props = $props();

	let updateInstalling = $state(false);
	let appVersion = $state('');

	async function installUpdate() {
		if (!updates.next || updateInstalling) return;
		updateInstalling = true;
		try {
			await updates.next.downloadAndInstall();
			pushInfoToast({ message: m.updater_update_message() });
			await relaunch();
		} catch (err) {
			pushToast({
				type: 'error',
				name: m.updater_installUpdate_message_name(),
				message: String(err)
			});
			updateInstalling = false;
		}
	}

	let unlistenProfiles: UnlistenFn | null;
	let unlistenGames: UnlistenFn | null;

	onMount(() => {
		getCurrentWindow()
			.isVisible()
			.then((visible) => {
				if (!visible) {
					getCurrentWindow().show();
				}
			});
		initErrorListener();
		// Kick off data loading now that Tauri IPC is ready
		profiles.refresh().catch(() => {});
		games.refresh().catch(() => {});
		auth.refresh().catch(() => {});
		updates.refresh().catch(() => {});
		getVersion().then((v) => (appVersion = v));
		initTheme();
		refreshFont();
		refreshColor('accent');
		refreshColor('primary');
		// Initialize language
		(async () => {
			let prefs = await api.prefs.get();
			let lang: string;

			if (await api.state.isFirstRun()) {
				const { locale } = await import('@tauri-apps/plugin-os');
				let systemLocale = await locale();
				if (systemLocale && locales.includes(systemLocale as Locale)) {
					lang = systemLocale;
					prefs.language = lang;
					await api.prefs.set(prefs);
				} else {
					lang = prefs.language;
				}
			} else {
				lang = prefs.language;
			}

			// Fallback to base locale if stored language was removed
			if (!locales.includes(lang as Locale)) {
				lang = 'en';
				prefs.language = lang;
				await api.prefs.set(prefs);
			}

			if (lang !== getLocale()) {
				updateAppLanguage(lang);
			}
		})();

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

<svelte:window
	onkeydown={(evt) => {
		const k = evt.key.toLowerCase();
		// Block F12 (devtools)
		if (k === 'f12') {
			evt.preventDefault();
			return;
		}
		// Block F5 (refresh via F5)
		if (k === 'f5') {
			evt.preventDefault();
			return;
		}
		// Ctrl+R: soft refresh (reload data without full page reload)
		if (evt.ctrlKey && !evt.shiftKey && !evt.altKey && k === 'r') {
			evt.preventDefault();
			profiles.refresh().catch(() => {});
			games.refresh().catch(() => {});
			return;
		}
		// Block Ctrl+shortcuts except Ctrl+C/V/X/A/Z (standard editing)
		if (evt.ctrlKey && !evt.shiftKey && !evt.altKey) {
			const allowed = ['c', 'v', 'x', 'a', 'z'];
			if (!allowed.includes(k)) {
				evt.preventDefault();
				return;
			}
		}
		// Block Ctrl+Shift+I/J/C (devtools variants)
		if (evt.ctrlKey && evt.shiftKey) {
			if (['i', 'j', 'c'].includes(k)) {
				evt.preventDefault();
				return;
			}
		}
	}}
/>

<svelte:body
	oncontextmenu={(evt) => {
		evt.preventDefault();
	}}
	onclick={(evt) => {
		const anchor = (evt.target as HTMLElement).closest('a[href]') as HTMLAnchorElement | null;
		if (!anchor) return;
		const href = anchor.href;
		if (
			href &&
			(href.startsWith('http://') || href.startsWith('https://')) &&
			new URL(href).origin !== window.location.origin
		) {
			evt.preventDefault();
			open(href);
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
	<!-- WelcomeDialog removed -->
	<ImportProfileDialog />

	{#if updates.next?.available}
		<Modal
			open={true}
			onclose={() => (updates.next = null)}
			title={i18nState.locale && m.updater_confirmDialog_title()}
		>
			{#snippet children()}
				<div class="z-update-modal">
					<p>
						{updates.next!.version
							? m.updater_confirmDialog_content_next({
									next: updates.next!.version,
									current: appVersion
								})
							: m.updater_confirmDialog_content_available()}
					</p>
					<p>{m.updater_confirmDialog_content()}</p>
				</div>
			{/snippet}
			{#snippet actions()}
				<Button variant="primary" onclick={installUpdate} disabled={updateInstalling}>
					{#snippet icon()}
						<Icon
							icon={updateInstalling ? 'mdi:loading' : 'mdi:download'}
							class={updateInstalling ? 'z-spin' : ''}
						/>
					{/snippet}
					{i18nState.locale && m.updater_confirmDialog_button()}
				</Button>
			{/snippet}
		</Modal>
	{/if}
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
