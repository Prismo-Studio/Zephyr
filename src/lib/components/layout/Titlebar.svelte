<script lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { platform } from '@tauri-apps/plugin-os';
	import { useNativeTitlebar } from '$lib/themeSystem';
	import { fullscreenState, setFullscreen } from '$lib/fullscreen.svelte';

	const appWindow = getCurrentWindow();
	const isMac = platform() === 'macos';

	let maximized = $state(false);

	async function checkMaximized() {
		maximized = await appWindow.isMaximized();
	}

	async function minimize() {
		await appWindow.minimize();
	}

	async function toggleMaximize() {
		if (fullscreenState.active) {
			await setFullscreen(false);
			await checkMaximized();
			return;
		}
		await appWindow.toggleMaximize();
		await checkMaximized();
	}

	async function close() {
		await appWindow.close();
	}

	$effect(() => {
		// On macOS we keep native decorations on (with titleBarStyle=Overlay +
		// hiddenTitle) so the OS renders the traffic-lights and supports native
		// fullscreen. Our custom titlebar sits behind them. Don't toggle
		// decorations there — the Native titlebar option is hidden on macOS.
		if (isMac) return;
		appWindow.setDecorations(useNativeTitlebar.current).catch(() => {});
	});

	onMount(() => {
		if (platform() === 'linux') {
			appWindow
				.setDecorations(false)
				.then(async () => {
					const size = await appWindow.innerSize();
					await appWindow.setSize(size);
				})
				.catch(() => {});
		}
		checkMaximized();
	});
</script>

{#if isMac || !useNativeTitlebar.current}
	<div
		class="z-titlebar"
		class:z-titlebar-mac={isMac && !fullscreenState.active}
		data-tauri-drag-region
	>
		<div class="z-titlebar-left" data-tauri-drag-region>
			<span class="z-titlebar-brand" data-tauri-drag-region>
				<img src="/logo.png" alt="Zephyr" class="z-titlebar-logo" />
				<span class="z-titlebar-name">Zephyr</span>
			</span>
		</div>

		<div class="z-titlebar-center" data-tauri-drag-region></div>

		{#if !isMac}
			<div class="z-titlebar-controls">
				<button
					class="z-titlebar-btn"
					onclick={minimize}
					title={i18nState.locale && m.titlebar_minimize()}
				>
					<Icon icon="mdi:minus" />
				</button>
				<button
					class="z-titlebar-btn"
					onclick={toggleMaximize}
					title={i18nState.locale && (maximized ? m.titlebar_restore() : m.titlebar_maximize())}
				>
					<Icon icon={maximized ? 'mdi:window-restore' : 'mdi:window-maximize'} />
				</button>
				<button
					class="z-titlebar-btn z-titlebar-close"
					onclick={close}
					title={i18nState.locale && m.titlebar_close()}
				>
					<Icon icon="mdi:close" />
				</button>
			</div>
		{/if}
	</div>
{/if}

<style>
	.z-titlebar {
		position: relative;
		display: flex;
		align-items: center;
		height: 32px;
		min-height: 32px;
		background: var(--bg-surface);
		border-bottom: 1px solid var(--border-subtle);
		user-select: none;
		-webkit-user-select: none;
		flex-shrink: 0;
		z-index: 999;
	}

	.z-titlebar-left {
		display: flex;
		align-items: center;
		padding-left: 12px;
		gap: 8px;
	}

	/* macOS: make room for the native traffic-light controls on the left */
	.z-titlebar-mac .z-titlebar-left {
		padding-left: 80px;
	}

	.z-titlebar-brand {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.z-titlebar-logo {
		width: 18px;
		height: 18px;
		object-fit: contain;
		border-radius: 4px;
	}

	.z-titlebar-name {
		font-family: var(--font-display);
		font-size: 12px;
		font-weight: 700;
		color: var(--text-secondary);
		letter-spacing: -0.02em;
	}

	.z-titlebar-center {
		flex: 1;
	}

	.z-titlebar-controls {
		display: flex;
		align-items: stretch;
		height: 100%;
	}

	.z-titlebar-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 46px;
		height: 100%;
		background: transparent;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		transition: all 0.1s ease;
		font-size: 16px;
	}

	.z-titlebar-btn:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.z-titlebar-close:hover {
		background: #e81123;
		color: white;
	}
</style>
