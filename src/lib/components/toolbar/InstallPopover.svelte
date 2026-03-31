<script lang="ts">
	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import type { InstallEvent } from '$lib/types';
	import Icon from '@iconify/svelte';
	import Progress from '$lib/components/ui/Progress.svelte';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import * as api from '$lib/api';

	let visible = $state(false);
	let totalMods = $state(0);
	let totalBytes = $state(0);
	let progressMods = $state(0);
	let progressBytes = $state(0);
	let currentTask = $state('');
	let currentMod = $state('');

	let unlisten: UnlistenFn | undefined;

	onMount(() => {
		listen<InstallEvent>('install_event', (evt) => {
			const e = evt.payload;
			switch (e.type) {
				case 'show':
					visible = true;
					totalMods = 0;
					totalBytes = 0;
					progressMods = 0;
					progressBytes = 0;
					currentTask = '';
					currentMod = '';
					break;
				case 'hide':
					visible = false;
					break;
				case 'addCount':
					totalMods += e.mods;
					totalBytes += e.bytes;
					break;
				case 'addProgress':
					progressMods += e.mods;
					progressBytes += e.bytes;
					break;
				case 'setTask':
					currentMod = e.name;
					currentTask = e.task;
					break;
			}
		}).then((cb) => (unlisten = cb));

		return () => unlisten?.();
	});

	let modsPercent = $derived(totalMods > 0 ? (progressMods / totalMods) * 100 : 0);

	async function cancelAll() {
		await api.profile.install.cancelAll();
	}

	function taskIcon(task: string) {
		switch (task) {
			case 'download':
				return 'mdi:download';
			case 'extract':
				return 'mdi:zip-box';
			case 'install':
				return 'mdi:package-variant-plus';
			default:
				return 'mdi:progress-wrench';
		}
	}
</script>

{#if visible}
	<div class="z-install-popover">
		<div class="z-install-header">
			<div class="z-install-title">
				<Spinner size={14} />
				<span>Installing mods...</span>
			</div>
			<button class="z-install-cancel" onclick={cancelAll} title="Cancel">
				<Icon icon="mdi:close" />
			</button>
		</div>

		<Progress value={modsPercent} />

		<div class="z-install-info">
			<div class="z-install-task">
				<Icon icon={taskIcon(currentTask)} class="text-xs" />
				<span class="z-install-task-label">{currentTask}</span>
			</div>
			<span class="z-install-mod">{currentMod}</span>
		</div>

		<div class="z-install-stats">
			<span>{progressMods} / {totalMods} mods</span>
		</div>
	</div>
{/if}

<style>
	.z-install-popover {
		position: fixed;
		bottom: 32px;
		right: var(--space-xl);
		width: 320px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
		padding: var(--space-lg);
		box-shadow: var(--shadow-lg);
		z-index: var(--z-modal);
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		animation: slideUp var(--transition-normal) ease;
	}

	.z-install-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.z-install-title {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.z-install-cancel {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.z-install-cancel:hover {
		background: rgba(255, 92, 92, 0.1);
		color: var(--error);
	}

	.z-install-info {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-size: 11px;
	}

	.z-install-task {
		display: flex;
		align-items: center;
		gap: 4px;
		color: var(--text-accent);
		text-transform: capitalize;
	}

	.z-install-task-label {
		font-weight: 600;
	}

	.z-install-mod {
		color: var(--text-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.z-install-stats {
		font-size: 11px;
		color: var(--text-muted);
		text-align: right;
	}
</style>
