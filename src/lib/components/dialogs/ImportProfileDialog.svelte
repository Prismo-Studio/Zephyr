<script lang="ts">
	import Modal from '$lib/components/ui/Modal.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import type { ImportData, LegacyImportData, SyncImportData } from '$lib/types';
	import * as api from '$lib/api';

	let open = $state(false);
	let importData: ImportData | null = $state(null);
	let unlisten: UnlistenFn | undefined;

	onMount(() => {
		listen<LegacyImportData | SyncImportData>('import_profile', (evt) => {
			const payload = evt.payload;
			if ('path' in payload) {
				importData = { type: 'legacy', ...payload };
			} else {
				importData = { type: 'sync', ...payload };
			}
			open = true;
		}).then((cb) => (unlisten = cb));

		return () => unlisten?.();
	});

	async function doImport() {
		if (!importData) return;

		if (importData.type === 'legacy') {
			const { type, ...data } = importData;
			await api.profile.import.profile(data, true);
		} else {
			const { type, ...data } = importData;
			await api.profile.sync.clone(data.id, data.manifest.profileName);
		}

		open = false;
	}
</script>

<Modal bind:open title="Import Profile">
	{#if importData}
		<div class="z-import-info">
			<div class="z-import-name">
				<Icon icon="mdi:account-circle" />
				<span>{importData.manifest.profileName}</span>
			</div>
			<p class="z-import-mods">
				{importData.manifest.mods.length} mod{importData.manifest.mods.length !== 1 ? 's' : ''}
			</p>
			{#if importData.type === 'legacy' && importData.missingMods.length > 0}
				<p class="z-import-warning">
					<Icon icon="mdi:alert" />
					{importData.missingMods.length} mod(s) could not be found
				</p>
			{/if}
		</div>
	{/if}

	{#snippet actions()}
		<Button variant="ghost" onclick={() => (open = false)}>Cancel</Button>
		<Button variant="primary" onclick={doImport}>
			{#snippet icon()}<Icon icon="mdi:import" />{/snippet}
			Import
		</Button>
	{/snippet}
</Modal>

<style>
	.z-import-info {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.z-import-name {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-size: 16px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.z-import-mods {
		font-size: 13px;
		color: var(--text-secondary);
	}

	.z-import-warning {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-size: 12px;
		color: var(--warning);
	}
</style>
