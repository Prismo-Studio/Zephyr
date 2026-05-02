<script lang="ts">
	import Icon from '@iconify/svelte';
	import PrefSection from './PrefSection.svelte';
	import * as api from '$lib/api';
	import type { Prefs } from '$lib/types';
	import { open as selectDirectory } from '@tauri-apps/plugin-dialog';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { pushInfoToast } from '$lib/toast.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		prefs: Prefs;
		onsave: () => Promise<void>;
	};

	let { prefs, onsave }: Props = $props();

	async function copyPath(path: string) {
		await writeText(path);
		pushInfoToast({ message: m.prefs_copyPath_success() });
	}

	async function openPath(path: string) {
		await api.prefs.openDir(path);
	}

	async function changeDir(field: 'dataDir' | 'cacheDir') {
		const selected = await selectDirectory({
			directory: true,
			multiple: false,
			defaultPath: prefs[field]
		});
		if (selected && typeof selected === 'string') {
			prefs[field] = selected;
			await onsave();
		}
	}
</script>

<PrefSection icon="mdi:folder" title={(i18nState.locale && m.prefs_locations_title()) ?? ''}>
	<div class="z-settings-path">
		<div class="z-settings-path-header">
			<span class="z-settings-path-label">
				{i18nState.locale && m.prefs_locations_dataFolder()}
			</span>
			<div class="z-settings-path-actions">
				<button class="z-path-action" onclick={() => copyPath(prefs.dataDir)} title="Copy">
					<Icon icon="mdi:content-copy" />
				</button>
				<button class="z-path-action" onclick={() => openPath(prefs.dataDir)} title="Open">
					<Icon icon="mdi:folder-open" />
				</button>
				<button class="z-path-action" onclick={() => changeDir('dataDir')} title="Change">
					<Icon icon="mdi:folder-edit" />
				</button>
			</div>
		</div>
		<code>{prefs.dataDir}</code>
	</div>

	<div class="z-settings-path">
		<div class="z-settings-path-header">
			<span class="z-settings-path-label">{m.prefs_locations_cacheFolder()}</span>
			<div class="z-settings-path-actions">
				<button class="z-path-action" onclick={() => copyPath(prefs.cacheDir)} title="Copy">
					<Icon icon="mdi:content-copy" />
				</button>
				<button class="z-path-action" onclick={() => openPath(prefs.cacheDir)} title="Open">
					<Icon icon="mdi:folder-open" />
				</button>
				<button class="z-path-action" onclick={() => changeDir('cacheDir')} title="Change">
					<Icon icon="mdi:folder-edit" />
				</button>
			</div>
		</div>
		<code>{prefs.cacheDir}</code>
	</div>
</PrefSection>

<style>
	.z-settings-path {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin-bottom: var(--space-md);
	}

	.z-settings-path-label {
		font-size: 12px;
		color: var(--text-muted);
	}

	.z-settings-path code {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-secondary);
		padding: var(--space-sm) var(--space-md);
		background: var(--bg-elevated);
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-subtle);
		word-break: break-all;
	}

	.z-settings-path-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 4px;
	}

	.z-settings-path-actions {
		display: flex;
		gap: var(--space-xs);
	}

	.z-path-action {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--text-muted);
		border: none;
		cursor: pointer;
		transition: all var(--transition-fast);
		font-size: 14px;
	}

	.z-path-action:hover {
		background: var(--bg-hover);
		color: var(--text-accent);
	}
</style>
