<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import { onMount, onDestroy } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { getCurrentWebview } from '@tauri-apps/api/webview';
	import { pushInfoToast, pushToast } from '$lib/toast';
	import {
		installApworldFromBytes,
		installApworldFromPath,
		listCustomApworlds,
		openCustomWorldsDir,
		refreshApworldSchemas,
		removeCustomApworld
	} from './api';
	import type { CustomApworld } from './types';
	import { randomizerStore } from './randomizer.store.svelte';

	type Props = {
		/** When true, the panel scans & listens for drops the moment it mounts. */
		autoload?: boolean;
		/** Controls whether the panel body is visible. The drop listener stays
		 * active either way so dropping a file auto-opens the panel via
		 * `onAutoOpen`. */
		visible?: boolean;
		/** Bump this from the parent to force a list refresh (e.g. after an
		 * external install action). */
		reloadToken?: number;
		onAutoOpen?: () => void;
	};
	let { autoload = true, visible = true, reloadToken = 0, onAutoOpen }: Props = $props();

	let installed: CustomApworld[] = $state([]);
	let loading = $state(false);
	let busy = $state(false);
	let refreshLog = $state<string | null>(null);
	let dragActive = $state(false);
	let unlisten: (() => void) | null = null;
	let nativeDropHandled = false;
	let dragCounter = 0;

	async function refreshList() {
		loading = true;
		try {
			installed = await listCustomApworlds();
		} catch (err) {
			console.error(err);
		} finally {
			loading = false;
		}
	}

	function formatSize(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
	}

	async function installPaths(paths: string[]) {
		if (!paths.length) return;
		busy = true;
		let ok = 0;
		let failed = 0;
		try {
			for (const p of paths) {
				try {
					await installApworldFromPath(p);
					ok++;
				} catch {
					failed++;
				}
			}
			if (ok > 0) {
				pushInfoToast({
					message:
						ok === 1
							? m.randomizer_customApworlds_installedSingle({
									name: paths[0].split(/[\\/]/).pop() ?? ''
								})
							: m.randomizer_customApworlds_installedMulti({ count: String(ok) })
				});
			}
			if (failed > 0) {
				pushToast({
					type: 'error',
					name: m.randomizer_customApworlds_installFailed(),
					message: m.randomizer_customApworlds_installFailedMsg({ count: String(failed) })
				});
			}
			await refreshList();
		} finally {
			busy = false;
		}
	}

	let pendingRemove: CustomApworld | null = $state(null);

	function remove(file: CustomApworld) {
		pendingRemove = file;
	}

	async function confirmRemove() {
		const file = pendingRemove;
		pendingRemove = null;
		if (!file) return;
		try {
			await removeCustomApworld(file.file_name);
			await refreshList();
		} catch (err) {
			console.error(err);
		}
	}

	async function refreshSchemas() {
		busy = true;
		refreshLog = null;
		try {
			const res = await refreshApworldSchemas();
			refreshLog = res.stdout || res.stderr || m.randomizer_runtime_done();
			if (!res.success) {
				pushToast({
					type: 'error',
					name: m.randomizer_customApworlds_refreshFailed(),
					message: (res.stderr || res.stdout || m.randomizer_customApworlds_unknownError()).slice(
						0,
						300
					)
				});
			} else {
				pushInfoToast({ message: m.randomizer_customApworlds_refreshed() });
				await randomizerStore.loadCatalog();
				await randomizerStore.reloadCurrentSchema();
			}
			await refreshList();
		} finally {
			busy = false;
		}
	}

	$effect(() => {
		// Track the external reload signal. First run is the initial mount, so
		// only trigger when the token actually bumps.
		if (reloadToken > 0) refreshList();
	});

	onMount(async () => {
		if (autoload) await refreshList();
		// Register Tauri OS drag-drop. Payload includes absolute paths.
		try {
			const un = await getCurrentWebview().onDragDropEvent(async (event) => {
				if (event.payload.type === 'enter' || event.payload.type === 'over') {
					dragActive = true;
				} else if (event.payload.type === 'leave') {
					dragActive = false;
				} else if (event.payload.type === 'drop') {
					dragActive = false;
					const paths = (event.payload.paths ?? []).filter((p) =>
						p.toLowerCase().endsWith('.apworld')
					);
					if (paths.length) {
						nativeDropHandled = true;
						onAutoOpen?.();
						await installPaths(paths);
						setTimeout(() => (nativeDropHandled = false), 500);
					}
				}
			});
			unlisten = un;
		} catch {
			// webview event API unavailable. Drag-drop falls back to in-page HTML5 events
		}
	});

	function arrayBufferToBase64(buf: ArrayBuffer): string {
		const bytes = new Uint8Array(buf);
		let binary = '';
		const chunk = 0x8000;
		for (let i = 0; i < bytes.length; i += chunk) {
			binary += String.fromCharCode.apply(null, Array.from(bytes.subarray(i, i + chunk)));
		}
		return btoa(binary);
	}

	async function installFiles(files: File[]) {
		const apworlds = files.filter((f) => f.name.toLowerCase().endsWith('.apworld'));
		if (!apworlds.length) return;
		busy = true;
		let ok = 0;
		let failed = 0;
		try {
			for (const f of apworlds) {
				try {
					const buf = await f.arrayBuffer();
					await installApworldFromBytes(f.name, arrayBufferToBase64(buf));
					ok++;
				} catch (err) {
					console.error('install failed', f.name, err);
					failed++;
				}
			}
			if (ok > 0) {
				pushInfoToast({
					message:
						ok === 1
							? m.randomizer_customApworlds_installedSingle({ name: apworlds[0].name })
							: m.randomizer_customApworlds_installedMulti({ count: String(ok) })
				});
				await randomizerStore.loadCatalog();
				await randomizerStore.reloadCurrentSchema();
			}
			if (failed > 0) {
				pushToast({
					type: 'error',
					name: m.randomizer_customApworlds_installFailed(),
					message: m.randomizer_customApworlds_installFailedMsg({ count: String(failed) })
				});
			}
			await refreshList();
		} finally {
			busy = false;
		}
	}

	function onDragEnter(e: DragEvent) {
		if (!e.dataTransfer) return;
		e.preventDefault();
		dragCounter++;
		dragActive = true;
	}

	function onDragOver(e: DragEvent) {
		if (!e.dataTransfer) return;
		e.preventDefault();
		e.dataTransfer.dropEffect = 'copy';
	}

	function onDragLeave(e: DragEvent) {
		if (!e.dataTransfer) return;
		e.preventDefault();
		dragCounter = Math.max(0, dragCounter - 1);
		if (dragCounter === 0) dragActive = false;
	}

	async function onDrop(e: DragEvent) {
		e.preventDefault();
		dragCounter = 0;
		dragActive = false;
		if (nativeDropHandled) return;
		const files = Array.from(e.dataTransfer?.files ?? []);
		if (!files.length) return;
		onAutoOpen?.();
		await installFiles(files);
	}

	onDestroy(() => {
		unlisten?.();
	});
</script>

<section class="apw-panel" class:apw-hidden={!visible}>
	<header class="apw-header">
		<div>
			<h2>
				<Icon icon="mdi:package-variant-plus" />
				{i18nState.locale && m.randomizer_customApworlds_title()}
			</h2>
			<p class="apw-subtitle">
				{i18nState.locale && m.randomizer_customApworlds_intro()}
			</p>
		</div>
		<div class="apw-actions">
			<Button size="sm" variant="ghost" onclick={openCustomWorldsDir}>
				{#snippet icon()}<Icon icon="mdi:folder-open" />{/snippet}
				{i18nState.locale && m.randomizer_customApworlds_openFolder()}
			</Button>
			<Button size="sm" variant="ghost" onclick={refreshList} disabled={loading}>
				{#snippet icon()}<Icon icon="mdi:refresh" />{/snippet}
				{i18nState.locale && m.randomizer_customApworlds_reload()}
			</Button>
		</div>
	</header>

	<div
		class="apw-drop"
		class:is-active={dragActive}
		class:is-busy={busy}
		role="region"
		aria-label={i18nState.locale && m.randomizer_customApworlds_dropTitle()}
		ondragenter={onDragEnter}
		ondragover={onDragOver}
		ondragleave={onDragLeave}
		ondrop={onDrop}
	>
		<Icon icon={busy ? 'mdi:loading' : 'mdi:tray-arrow-down'} class={busy ? 'apw-spin' : ''} />
		<div class="apw-drop-body">
			<strong>{i18nState.locale && m.randomizer_customApworlds_dropTitle()}</strong>
			<small>{i18nState.locale && m.randomizer_customApworlds_dropSub()}</small>
		</div>
	</div>

	{#if installed.length > 0}
		<div class="apw-list">
			{#each installed as file (file.file_name)}
				<div class="apw-item">
					<Icon icon="mdi:puzzle" class="apw-item-icon" />
					<div class="apw-item-body">
						<div class="apw-item-title">
							{file.display_name || file.world_id || file.file_name}
							{#if file.world_version}
								<span class="apw-version">v{file.world_version}</span>
							{/if}
						</div>
						<div class="apw-item-meta">
							<code>{file.file_name}</code>
							<span>{formatSize(file.size)}</span>
							{#if file.has_schema}
								<span class="apw-chip apw-chip-ok">
									<Icon icon="mdi:check-circle" />
									{i18nState.locale && m.randomizer_customApworlds_hasSchema()}
								</span>
							{:else}
								<span class="apw-chip apw-chip-warn">
									<Icon icon="mdi:alert-circle" />
									{i18nState.locale && m.randomizer_customApworlds_noSchema()}
								</span>
							{/if}
						</div>
					</div>
					<button
						class="apw-remove"
						aria-label="Remove {file.file_name}"
						onclick={() => remove(file)}
						disabled={busy}
					>
						<Icon icon="mdi:trash-can-outline" />
					</button>
				</div>
			{/each}
		</div>

		<div class="apw-footer">
			<Button size="md" variant="primary" onclick={refreshSchemas} disabled={busy}>
				{#snippet icon()}<Icon icon="mdi:database-refresh" />{/snippet}
				{i18nState.locale && m.randomizer_customApworlds_refreshSchemas()}
			</Button>
			<small class="apw-hint">
				{i18nState.locale && m.randomizer_customApworlds_refreshHint()}
			</small>
		</div>

		{#if refreshLog}
			<details class="apw-log">
				<summary>{i18nState.locale && m.randomizer_customApworlds_lastOutput()}</summary>
				<pre>{refreshLog}</pre>
			</details>
		{/if}
	{:else if !loading}
		<div class="apw-empty">
			<Icon icon="mdi:package-variant-closed" />
			<span>{i18nState.locale && m.randomizer_customApworlds_empty()}</span>
		</div>
	{/if}
</section>

<Modal
	open={pendingRemove !== null}
	title={i18nState.locale && m.randomizer_customApworlds_removeTitle()}
	onclose={() => (pendingRemove = null)}
>
	<p>
		{i18nState.locale &&
			m.randomizer_customApworlds_removeConfirm({ name: pendingRemove?.file_name ?? '' })}
	</p>
	{#snippet actions()}
		<Button variant="ghost" onclick={() => (pendingRemove = null)}>
			{i18nState.locale && m.randomizer_customApworlds_cancel()}
		</Button>
		<Button variant="danger" onclick={confirmRemove}>
			{i18nState.locale && m.randomizer_customApworlds_remove()}
		</Button>
	{/snippet}
</Modal>

<style>
	.apw-panel {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		padding: var(--space-sm) var(--space-md);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-lg);
		background: var(--bg-surface);
	}

	.apw-hidden {
		display: none;
	}

	.apw-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: var(--space-md);
		flex-wrap: wrap;
	}

	.apw-header h2 {
		margin: 0;
		font-size: 16px;
		display: inline-flex;
		align-items: center;
		gap: var(--space-xs);
		color: var(--text-primary);
	}

	.apw-subtitle {
		margin: var(--space-xs) 0 0;
		font-size: 12px;
		color: var(--text-muted);
		max-width: 60ch;
	}

	.apw-subtitle code,
	.apw-item-meta code {
		background: var(--bg-active);
		padding: 1px 6px;
		border-radius: var(--radius-sm);
		font-size: 11px;
	}

	.apw-actions {
		display: flex;
		gap: var(--space-xs);
	}

	.apw-drop {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-md) var(--space-lg);
		border: 2px dashed var(--border-default);
		border-radius: var(--radius-md);
		background: var(--bg-base);
		transition:
			border-color var(--transition-fast),
			background var(--transition-fast);
	}

	.apw-drop code {
		background: var(--bg-active);
		padding: 1px 6px;
		border-radius: var(--radius-sm);
		font-size: 11px;
	}

	.apw-drop-body {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.apw-drop-body strong {
		color: var(--text-primary);
		font-size: 13px;
	}

	.apw-drop-body small {
		color: var(--text-muted);
		font-size: 11px;
	}

	.apw-drop.is-active {
		border-color: var(--accent-400);
		background: var(--bg-active);
	}

	.apw-drop.is-busy {
		opacity: 0.7;
		pointer-events: none;
	}

	.apw-drop :global(svg) {
		font-size: 30px;
		color: var(--accent-400);
		flex-shrink: 0;
	}

	:global(.apw-spin) {
		animation: apw-spin 1s linear infinite;
	}

	@keyframes apw-spin {
		to {
			transform: rotate(360deg);
		}
	}

	.apw-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
	}

	.apw-item {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		background: var(--bg-base);
	}

	.apw-item :global(.apw-item-icon) {
		font-size: 22px;
		color: var(--accent-400);
		flex-shrink: 0;
	}

	.apw-item-body {
		flex: 1;
		min-width: 0;
	}

	.apw-item-title {
		font-weight: 600;
		color: var(--text-primary);
		font-size: 13px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.apw-version {
		margin-left: var(--space-xs);
		font-size: 10px;
		font-family: var(--font-mono, monospace);
		color: var(--text-muted);
	}

	.apw-item-meta {
		margin-top: 2px;
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-size: 11px;
		color: var(--text-muted);
		flex-wrap: wrap;
	}

	.apw-chip {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		padding: 1px 6px;
		border-radius: var(--radius-full);
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
	}
	.apw-chip :global(svg) {
		font-size: 11px;
	}
	.apw-chip-ok {
		background: rgba(80, 200, 120, 0.12);
		color: #65c88a;
	}
	.apw-chip-warn {
		background: rgba(240, 180, 60, 0.12);
		color: #f0b43c;
	}

	.apw-remove {
		background: transparent;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		border-radius: var(--radius-sm);
		padding: 6px;
		display: flex;
		align-items: center;
		justify-content: center;
		transition:
			color var(--transition-fast),
			background var(--transition-fast);
	}

	.apw-remove:hover {
		color: #ef5555;
		background: var(--bg-active);
	}

	.apw-remove :global(svg) {
		font-size: 18px;
	}

	.apw-footer {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		flex-wrap: wrap;
	}

	.apw-hint {
		color: var(--text-muted);
		font-size: 11px;
	}

	.apw-log {
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		background: var(--bg-base);
		padding: var(--space-sm);
		font-size: 11px;
	}

	.apw-log summary {
		cursor: pointer;
		color: var(--text-muted);
	}

	.apw-log pre {
		margin: var(--space-xs) 0 0;
		max-height: 200px;
		overflow: auto;
		white-space: pre-wrap;
		word-break: break-word;
		font-family: var(--font-mono, monospace);
		font-size: 11px;
		color: var(--text-secondary);
	}

	.apw-empty {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-md);
		color: var(--text-muted);
		font-size: 12px;
	}

	.apw-empty :global(svg) {
		font-size: 20px;
		opacity: 0.7;
	}
</style>
