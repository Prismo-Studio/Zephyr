<script lang="ts">
	import Modal from '$lib/components/ui/Modal.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Loader from '$lib/components/ui/Loader.svelte';
	import Icon from '@iconify/svelte';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import * as api from '$lib/api';
	import type { LegacyImportData } from '$lib/types';
	import profiles from '$lib/state/profile.svelte';
	import games from '$lib/state/game.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { pushToast } from '$lib/toast';
	import { writeText, readText } from '@tauri-apps/plugin-clipboard-manager';
	import { maybeSyncAfterImport } from '$lib/state/autoSync.svelte';

	type Props = {
		mode: 'export' | 'import';
		open: boolean;
		onclose: () => void;
	};

	let { mode = $bindable(), open = $bindable(), onclose }: Props = $props();

	// Export state
	let exportCode: string | null = $state(null);
	let exportLoading = $state(false);
	let copied = $state(false);

	// Import state
	let importCode = $state('');
	let importLoading = $state(false);
	let importData: LegacyImportData | null = $state(null);
	let importError: string | null = $state(null);
	let showNameInput = $state(false);
	let newProfileName = $state('');
	let importGameMismatch: string | null = $state(null);

	let nameConflict = $derived(
		showNameInput &&
			newProfileName.trim().length > 0 &&
			profiles.list.some((p) => p.name.toLowerCase() === newProfileName.trim().toLowerCase())
	);

	$effect(() => {
		if (open && mode === 'export' && !exportCode && !exportLoading) {
			doExport();
		}
	});

	$effect(() => {
		if (!open) {
			exportCode = null;
			exportLoading = false;
			copied = false;
			importCode = '';
			importLoading = false;
			importData = null;
			importError = null;
			importGameMismatch = null;
			showNameInput = false;
			newProfileName = '';
		}
	});

	async function doExport() {
		exportLoading = true;
		try {
			exportCode = await api.profile.export.code();
		} catch (e) {
			pushToast({ type: 'error', message: String(e) });
			onclose();
		} finally {
			exportLoading = false;
		}
	}

	async function copyCode() {
		if (!exportCode) return;
		await writeText(exportCode);
		copied = true;
		pushToast({ type: 'info', message: (i18nState.locale && m.share_exportCopied()) || 'Copied!' });
		setTimeout(() => (copied = false), 2000);
	}

	async function handlePaste() {
		try {
			const clip = await readText();
			if (clip) importCode = clip;
		} catch {}
	}

	async function doLoadImport() {
		const code = importCode.trim();
		if (!code) return;
		importLoading = true;
		importError = null;
		importGameMismatch = null;
		try {
			// Pre-read to detect game mismatch before resolving mods
			importData = await api.profile.import.readCode(code);

			const profileCommunity = importData.manifest.community;
			if (profileCommunity && profileCommunity !== games.active?.slug) {
				const targetGame = games.list.find((g) => g.slug === profileCommunity);
				importGameMismatch = targetGame?.name ?? profileCommunity;

				// Switch to target game now so mods resolve correctly and the
				// overwrite button shows the right profile name
				await games.setActive(profileCommunity);
				await profiles.refresh();
				try {
					await api.thunderstore.triggerModFetch();
				} catch {}
				// Re-read with the correct game's packages
				importData = await api.profile.import.readCode(code);
			}
		} catch (e) {
			importError = String(e);
		} finally {
			importLoading = false;
		}
	}

	function startImportCreate() {
		if (!importData) return;
		newProfileName = importData.manifest.profileName;
		showNameInput = true;
	}

	async function doImportCreate() {
		if (!importData) return;
		const name = newProfileName.trim();
		if (!name) return;
		importData.manifest.profileName = name;
		importLoading = true;
		try {
			await api.profile.import.profile(importData, true);
			await profiles.refresh();
			pushToast({
				type: 'info',
				message:
					(i18nState.locale && m.share_importSuccess({ name })) || `Profile "${name}" imported!`
			});
			await maybeSyncAfterImport({ forceFork: true });
			onclose();
		} catch (e) {
			pushToast({ type: 'error', message: String(e) });
		} finally {
			importLoading = false;
		}
	}

	async function doImportOverwrite() {
		if (!importData) return;
		importLoading = true;
		try {
			if (profiles.active) {
				importData.manifest.profileName = profiles.active.name;
			}
			await api.profile.import.profile(importData, false);
			await profiles.refresh();
			pushToast({
				type: 'info',
				message:
					(i18nState.locale && m.share_importOverwriteSuccess()) ||
					'Mods imported into current profile!'
			});
			await maybeSyncAfterImport();
			onclose();
		} catch (e) {
			pushToast({ type: 'error', message: String(e) });
		} finally {
			importLoading = false;
		}
	}
</script>

<Modal
	bind:open
	title={mode === 'export'
		? i18nState.locale && m.share_exportTitle()
		: i18nState.locale && m.share_importTitle()}
	{onclose}
>
	{#snippet children()}
		{#if mode === 'export'}
			<!-- EXPORT VIEW -->
			<div class="z-share-body">
				<p class="z-share-desc">{i18nState.locale && m.share_exportDesc()}</p>

				{#if exportLoading}
					<div class="z-share-loading">
						<Spinner size={20} />
						<span>{i18nState.locale && m.share_exportLoading()}</span>
					</div>
				{:else if exportCode}
					<div class="z-share-code-box">
						<code class="z-share-code">{exportCode}</code>
						<button class="z-share-copy-btn" class:copied onclick={copyCode}>
							<Icon icon={copied ? 'mdi:check' : 'mdi:content-copy'} />
							<span
								>{copied
									? i18nState.locale && m.share_exportCopied()
									: i18nState.locale && m.share_exportCopy()}</span
							>
						</button>
					</div>
				{/if}
			</div>
		{:else}
			<!-- IMPORT VIEW -->
			<div class="z-share-body">
				<p class="z-share-desc">{i18nState.locale && m.share_importDesc()}</p>

				{#if !importData}
					<div class="z-share-input-row">
						<input
							class="z-share-input"
							type="text"
							placeholder={i18nState.locale && m.share_importPlaceholder()}
							bind:value={importCode}
							onkeydown={(e) => e.key === 'Enter' && doLoadImport()}
						/>
						<Button variant="ghost" size="sm" onclick={handlePaste}>
							{#snippet icon()}<Icon icon="mdi:clipboard-text" />{/snippet}
						</Button>
					</div>

					{#if importError}
						<div class="z-share-error">
							<Icon icon="mdi:alert-circle" />
							<span>{importError}</span>
						</div>
					{/if}

					{#if importLoading}
						<div class="z-share-loading">
							<Spinner size={20} />
							<span>{i18nState.locale && m.share_importLoading()}</span>
						</div>
					{/if}
				{:else}
					<!-- Preview -->
					<div class="z-share-preview">
						<div class="z-share-preview-header">
							<Icon icon="mdi:account-circle" />
							<span
								>{i18nState.locale &&
									m.share_importFrom({ name: importData.manifest.profileName })}</span
							>
						</div>
						{#if importGameMismatch}
							<div class="z-share-game-warning">
								<Icon icon="mdi:swap-horizontal" />
								<span>
									{i18nState.locale && m.share_importGameSwitch
										? m.share_importGameSwitch({ game: importGameMismatch })
										: `This profile is for ${importGameMismatch}. The game will be switched automatically.`}
								</span>
							</div>
						{/if}

						<div class="z-share-preview-stats">
							<span class="z-share-stat">
								<Icon icon="mdi:package-variant" />
								{i18nState.locale &&
									m.share_importMods({ count: importData.manifest.mods.length.toString() })}
							</span>
							{#if importData.missingMods.length > 0}
								<span class="z-share-stat z-share-stat-warn">
									<Icon icon="mdi:alert" />
									{i18nState.locale &&
										m.share_importMissing({ count: importData.missingMods.length.toString() })}
								</span>
							{/if}
						</div>
						<div class="z-share-mod-list">
							{#each importData.manifest.mods as mod}
								<div class="z-share-mod-item">
									<Icon icon="mdi:puzzle" />
									<span>{mod.name}</span>
									<span class="z-share-mod-ver"
										>{mod.version.major}.{mod.version.minor}.{mod.version.patch}</span
									>
								</div>
							{/each}
						</div>
					</div>

					{#if showNameInput}
						<div class="z-share-name-input">
							<label class="z-share-name-label">{i18nState.locale && m.share_profileName()}</label>
							<input
								class="z-share-input"
								class:z-share-input-error={nameConflict}
								type="text"
								bind:value={newProfileName}
								onkeydown={(e) => e.key === 'Enter' && !nameConflict && doImportCreate()}
							/>
							{#if nameConflict}
								<div class="z-share-name-conflict">
									<Icon icon="mdi:alert-circle" />
									<span>
										{i18nState.locale && m.share_importNameConflict
											? m.share_importNameConflict({ name: newProfileName.trim() })
											: `A profile named "${newProfileName.trim()}" already exists.`}
									</span>
								</div>
							{/if}
						</div>
					{/if}
				{/if}
			</div>
		{/if}
	{/snippet}

	{#snippet actions()}
		{#if mode === 'export'}
			<Button variant="ghost" onclick={onclose}>{i18nState.locale && m.share_importCancel()}</Button
			>
		{:else if !importData}
			<Button variant="ghost" onclick={onclose}>{i18nState.locale && m.share_importCancel()}</Button
			>
			<Button
				variant="primary"
				onclick={doLoadImport}
				disabled={!importCode.trim() || importLoading}
			>
				{#snippet icon()}<Icon icon="mdi:magnify" />{/snippet}
				{i18nState.locale && m.share_importButton()}
			</Button>
		{:else if showNameInput}
			<Button
				variant="ghost"
				onclick={() => {
					showNameInput = false;
				}}>{i18nState.locale && m.share_importCancel()}</Button
			>
			<Button
				variant="primary"
				onclick={doImportCreate}
				disabled={!newProfileName.trim() || importLoading || nameConflict}
			>
				{#snippet icon()}<Icon icon="mdi:plus" />{/snippet}
				{i18nState.locale && m.share_importCreate()}
			</Button>
		{:else}
			<Button variant="ghost" onclick={onclose}>{i18nState.locale && m.share_importCancel()}</Button
			>
			<Button variant="secondary" onclick={doImportOverwrite} disabled={importLoading}>
				{#snippet icon()}<Icon icon="mdi:swap-horizontal" />{/snippet}
				{i18nState.locale && m.share_importOverwriteProfile
					? m.share_importOverwriteProfile({ name: profiles.active?.name ?? 'Default' })
					: `Overwrite "${profiles.active?.name ?? 'Default'}"`}
			</Button>
			<Button variant="primary" onclick={startImportCreate} disabled={importLoading}>
				{#snippet icon()}<Icon icon="mdi:plus" />{/snippet}
				{i18nState.locale && m.share_importCreate()}
			</Button>
		{/if}
	{/snippet}
</Modal>

<style>
	.z-share-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.z-share-desc {
		font-size: 13px;
		color: var(--text-secondary);
		line-height: 1.5;
	}

	.z-share-loading {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		color: var(--text-muted);
		font-size: 13px;
		padding: var(--space-md) 0;
	}

	.z-share-code-box {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		background: var(--bg-base);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		padding: var(--space-sm) var(--space-md);
	}

	.z-share-code {
		flex: 1;
		font-family: var(--font-mono, monospace);
		font-size: 13px;
		color: var(--accent-400);
		word-break: break-all;
		user-select: all;
		line-height: 1.6;
	}

	.z-share-copy-btn {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		flex-shrink: 0;
		padding: var(--space-xs) var(--space-md);
		border-radius: var(--radius-full);
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		color: var(--text-secondary);
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
		transition: all var(--transition-fast);
		font-family: var(--font-body);
	}

	.z-share-copy-btn:hover {
		border-color: var(--accent-400);
		color: var(--accent-400);
	}

	.z-share-copy-btn.copied {
		border-color: var(--success);
		color: var(--success);
	}

	.z-share-input-row {
		display: flex;
		gap: var(--space-xs);
		align-items: center;
	}

	.z-share-input {
		flex: 1;
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-md);
		border: 1px solid var(--border-default);
		background: var(--bg-base);
		color: var(--text-primary);
		font-size: 13px;
		font-family: var(--font-mono, monospace);
		outline: none;
		transition: border-color var(--transition-fast);
	}

	.z-share-input:focus {
		border-color: var(--accent-400);
	}

	.z-share-input::placeholder {
		color: var(--text-muted);
	}

	.z-share-error {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-size: 12px;
		color: var(--danger);
	}

	.z-share-input-error {
		border-color: var(--danger) !important;
		box-shadow: 0 0 0 1px var(--danger);
	}

	.z-share-name-conflict {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		font-size: 12px;
		color: var(--danger);
		margin-top: var(--space-xs);
	}

	.z-share-game-warning {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-md);
		background: rgba(59, 130, 246, 0.08);
		border: 1px solid rgba(59, 130, 246, 0.25);
		color: var(--accent-400);
		font-size: 12px;
		line-height: 1.4;
	}

	.z-share-preview {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		background: var(--bg-base);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		padding: var(--space-md);
	}

	.z-share-preview-header {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-size: 15px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.z-share-preview-stats {
		display: flex;
		gap: var(--space-md);
	}

	.z-share-stat {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 12px;
		color: var(--text-secondary);
	}

	.z-share-stat-warn {
		color: var(--warning);
	}

	.z-share-mod-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
		max-height: 200px;
		overflow-y: auto;
		margin-top: var(--space-xs);
	}

	.z-share-mod-item {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-xs) var(--space-sm);
		border-radius: var(--radius-sm);
		font-size: 12px;
		color: var(--text-secondary);
	}

	.z-share-mod-item:hover {
		background: var(--bg-hover);
	}

	.z-share-mod-ver {
		margin-left: auto;
		color: var(--text-muted);
		font-family: var(--font-mono, monospace);
		font-size: 11px;
	}

	.z-share-name-input {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
		margin-top: var(--space-sm);
	}

	.z-share-name-label {
		font-size: 12px;
		font-weight: 600;
		color: var(--text-secondary);
	}
</style>
