<script lang="ts">
	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import type { InstallEvent } from '$lib/types';
	import Icon from '@iconify/svelte';
	import Progress from '$lib/components/ui/Progress.svelte';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import * as api from '$lib/api';
	import Button from '$lib/components/ui/Button.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { installState } from '$lib/state/misc.svelte';

	let visible = $state(false);
	let totalMods = $state(0);
	let totalBytes = $state(0);
	let progressMods = $state(0);
	let progressBytes = $state(0);
	let currentTask = $state('');
	let currentMod = $state('');

	let unlisten: UnlistenFn | undefined;

	function reset() {
		totalMods = 0;
		totalBytes = 0;
		progressMods = 0;
		progressBytes = 0;
		currentTask = '';
		currentMod = '';
	}

	onMount(() => {
		listen<InstallEvent>('install_event', (evt) => {
			const e = evt.payload;
			switch (e.type) {
				case 'show':
					// addCount arrives before show due to async queue,
					// so don't reset here — counters are already set
					visible = true;
					installState.active = true;
					break;
				case 'hide':
					visible = false;
					installState.active = false;
					reset();
					break;
				case 'addCount':
					// If this is the first addCount after a hide, counters
					// are already at 0 from the reset in hide
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
	let remaining = $derived(totalMods - progressMods);

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
				<span>{i18nState.locale && m.installPopover_content()}</span>
			</div>
		</div>

		<Progress value={modsPercent} />

		<div class="z-install-info">
			<div class="z-install-task">
				<Icon icon={taskIcon(currentTask)} class="text-xs" />
				<span class="z-install-task-label">{currentTask}</span>
			</div>
			<span class="z-install-mod">{currentMod}</span>
			<span class="z-install-stats">{progressMods} / {totalMods} mods</span>
		</div>

		<div class="z-install-footer">
			<Button variant="danger" size="sm" onclick={cancelAll}>
				{#snippet icon()}<Icon icon="mdi:cancel" />{/snippet}
				{i18nState.locale && m.installPopover_cancelAll()}
			</Button>
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
		margin-left: auto;
		font-size: 11px;
		color: var(--text-muted);
		white-space: nowrap;
	}

	.z-install-footer {
		display: flex;
		justify-content: flex-end;
	}
</style>
