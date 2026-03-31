<script lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import Icon from '@iconify/svelte';

	const appWindow = getCurrentWindow();

	let maximized = $state(false);

	async function checkMaximized() {
		maximized = await appWindow.isMaximized();
	}

	async function minimize() {
		await appWindow.minimize();
	}

	async function toggleMaximize() {
		await appWindow.toggleMaximize();
		await checkMaximized();
	}

	async function close() {
		await appWindow.close();
	}

	// Check on mount
	checkMaximized();
</script>

<div class="z-titlebar" data-tauri-drag-region>
	<div class="z-titlebar-left" data-tauri-drag-region>
		<span class="z-titlebar-brand" data-tauri-drag-region>
			<span class="z-titlebar-logo">Z</span>
			<span class="z-titlebar-name">Zephyr</span>
		</span>
	</div>

	<div class="z-titlebar-center" data-tauri-drag-region></div>

	<div class="z-titlebar-controls">
		<button class="z-titlebar-btn" onclick={minimize} title="Minimize">
			<Icon icon="mdi:minus" />
		</button>
		<button class="z-titlebar-btn" onclick={toggleMaximize} title={maximized ? 'Restore' : 'Maximize'}>
			<Icon icon={maximized ? 'mdi:window-restore' : 'mdi:window-maximize'} />
		</button>
		<button class="z-titlebar-btn z-titlebar-close" onclick={close} title="Close">
			<Icon icon="mdi:close" />
		</button>
	</div>
</div>

<style>
	.z-titlebar {
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

	.z-titlebar-brand {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.z-titlebar-logo {
		font-family: var(--font-display);
		font-size: 14px;
		font-weight: 900;
		width: 20px;
		height: 20px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 5px;
		background: linear-gradient(135deg, var(--accent-400), var(--accent-600));
		color: var(--text-inverse);
		line-height: 1;
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
