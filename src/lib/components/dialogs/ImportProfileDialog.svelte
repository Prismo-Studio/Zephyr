<script lang="ts">
	import Modal from '$lib/components/ui/Modal.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import type { ImportData, LegacyImportData, SyncImportData } from '$lib/types';
	import * as api from '$lib/api';
	import profiles from '$lib/state/profile.svelte';
	import { pushToast } from '$lib/toast.svelte';
	import { maybeSyncAfterImport } from '$lib/state/autoSync.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	let open = $state(false);
	let importData: ImportData | null = $state(null);
	let busy = $state(false);
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

	async function importAsNew() {
		if (!importData) return;
		busy = true;
		try {
			if (importData.type === 'legacy') {
				const { type, ...data } = importData;
				await api.profile.import.profile(data, true);
				await profiles.refresh();
			} else {
				const { type, ...data } = importData;
				await api.profile.sync.clone(data.id, data.manifest.profileName);
				await profiles.refresh();
			}
			await maybeSyncAfterImport({ forceFork: true });
			open = false;
		} catch (e: any) {
			pushToast({ type: 'error', name: m.import_failed(), message: e?.message ?? String(e) });
		} finally {
			busy = false;
		}
	}

	async function replaceActive() {
		if (!importData || importData.type !== 'sync') return;
		const active = profiles.active;
		if (!active) {
			await importAsNew();
			return;
		}
		busy = true;
		try {
			const { type, ...data } = importData;
			const targetName = active.name;
			const activeId = active.id;
			await api.profile.sync.clone(data.id, data.manifest.profileName);
			await api.profile.deleteProfile(activeId);
			await profiles.refresh();
			await maybeSyncAfterImport({ forceFork: true });
			pushToast({
				type: 'info',
				name: m.import_replaced(),
				message: m.import_replacedDesc({
					source: targetName,
					target: data.manifest.profileName
				})
			});
			open = false;
		} catch (e: any) {
			pushToast({ type: 'error', name: m.import_failed(), message: e?.message ?? String(e) });
		} finally {
			busy = false;
		}
	}
</script>

<Modal bind:open title={(i18nState.locale && m.import_title()) || ''}>
	{#if importData}
		<div class="z-import-info">
			<div class="z-import-name">
				<Icon icon="mdi:account-circle" />
				<span>{importData.manifest.profileName}</span>
			</div>
			<p class="z-import-mods">
				{i18nState.locale && m.profiles_mods({ count: importData.manifest.mods.length.toString() })}
			</p>
			{#if importData.type === 'legacy' && importData.missingMods.length > 0}
				<p class="z-import-warning">
					<Icon icon="mdi:alert" />
					{i18nState.locale &&
						m.import_modsNotFound({ count: importData.missingMods.length.toString() })}
				</p>
			{/if}
			{#if importData.type === 'sync'}
				<p class="z-import-hint">
					{i18nState.locale &&
						(profiles.active
							? m.import_syncHint({ name: profiles.active.name })
							: m.import_syncHintNoActive())}
				</p>
			{/if}
		</div>
	{/if}

	{#snippet actions()}
		<Button variant="ghost" onclick={() => (open = false)} disabled={busy}>
			{i18nState.locale && m.import_cancel()}
		</Button>
		{#if importData?.type === 'sync' && profiles.active}
			<Button variant="secondary" onclick={replaceActive} loading={busy} disabled={busy}>
				{#snippet icon()}<Icon icon="mdi:swap-horizontal" />{/snippet}
				{i18nState.locale && m.import_replaceActive()}
			</Button>
		{/if}
		<Button variant="primary" onclick={importAsNew} loading={busy} disabled={busy}>
			{#snippet icon()}<Icon icon="mdi:import" />{/snippet}
			{i18nState.locale && m.import_newProfile()}
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

	.z-import-hint {
		font-size: 12px;
		color: var(--text-muted);
		line-height: 1.5;
	}
</style>
