<script lang="ts">
	import Icon from '@iconify/svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { onMount } from 'svelte';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';
	import { randomizerStore } from './randomizer.store.svelte';
	import GameLogo from './GameLogo.svelte';
	import CustomApworldsPanel from './CustomApworldsPanel.svelte';
	import RuntimeInstallBanner from './RuntimeInstallBanner.svelte';
	import { installApworldFromPath, installApworldsFromFolder } from './api';
	import { pushInfoToast, pushToast } from '$lib/toast';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	const { onSelect }: { onSelect: (gameId: string) => void } = $props();

	let search = $state('');
	let showCustom = $state(false);
	/** Bumped by the header install button so the panel refreshes its list. */
	let panelReloadToken = $state(0);
	let installing = $state(false);

	let installMenuOpen = $state(false);
	let installMenuEl: HTMLDivElement | null = $state(null);

	function closeInstallMenu(e: MouseEvent) {
		if (!installMenuOpen) return;
		if (installMenuEl && !installMenuEl.contains(e.target as Node)) {
			installMenuOpen = false;
		}
	}

	$effect(() => {
		if (installMenuOpen) {
			document.addEventListener('mousedown', closeInstallMenu, true);
		} else {
			document.removeEventListener('mousedown', closeInstallMenu, true);
		}
		return () => document.removeEventListener('mousedown', closeInstallMenu, true);
	});

	async function installApworldFiles() {
		installMenuOpen = false;
		if (installing) return;
		const picked = await openDialog({
			filters: [{ name: 'Archipelago world', extensions: ['apworld'] }],
			multiple: true
		});
		if (!picked) return;
		const paths = Array.isArray(picked) ? picked : [picked];
		installing = true;
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
						ok === 1 ? `Installed ${paths[0].split(/[\\/]/).pop()}` : `Installed ${ok} apworld(s)`
				});
				showCustom = true;
				panelReloadToken++;
			}
			if (failed > 0) {
				pushToast({
					type: 'error',
					name: 'Install failed',
					message: `${failed} file(s) could not be installed.`
				});
			}
		} finally {
			installing = false;
		}
	}

	async function installApworldFolder() {
		installMenuOpen = false;
		if (installing) return;
		const picked = await openDialog({ directory: true, multiple: false });
		if (!picked || Array.isArray(picked)) return;
		installing = true;
		try {
			const res = await installApworldsFromFolder(picked);
			if (res.installed.length > 0) {
				pushInfoToast({
					message:
						res.installed.length === 1
							? `Installed ${res.installed[0].file_name}`
							: `Installed ${res.installed.length} apworld(s)`
				});
				showCustom = true;
				panelReloadToken++;
			} else if (res.failed.length === 0) {
				pushInfoToast({ message: 'No .apworld files found in that folder' });
			}
			if (res.failed.length > 0) {
				pushToast({
					type: 'error',
					name: 'Install failed',
					message: `${res.failed.length} file(s) could not be installed.`
				});
			}
		} catch (err) {
			pushToast({
				type: 'error',
				name: 'Install failed',
				message: err instanceof Error ? err.message : String(err)
			});
		} finally {
			installing = false;
		}
	}

	function normalize(s: string): string {
		return s.toLowerCase().replace(/[^a-z0-9]/g, '');
	}

	const filtered = $derived.by(() => {
		const q = normalize(search.trim());
		if (!q) return randomizerStore.catalog;
		return randomizerStore.catalog.filter(
			(g) => normalize(g.name).includes(q) || normalize(g.id).includes(q)
		);
	});

	onMount(() => {
		if (randomizerStore.catalog.length === 0) {
			randomizerStore.loadCatalog();
		}
	});
</script>

<div class="rdz-catalog">
	<header class="rdz-catalog-header">
		<div class="rdz-catalog-title">
			<h1>
				{i18nState.locale && m.randomizer_title()}
			</h1>
			<p class="rdz-subtitle">
				{i18nState.locale && m.randomizer_description()}
				<strong>{randomizerStore.catalog.length}</strong>
				{i18nState.locale && m.randomizer_gamesAvailable()}
			</p>
		</div>
		<div class="rdz-catalog-actions">
			<div class="rdz-search" data-tour="rdz-search">
				<Input
					bind:value={search}
					placeholder={i18nState.locale && m.randomizer_searchPlaceholder()}
				>
					{#snippet iconLeft()}
						<Icon icon="mdi:magnify" />
					{/snippet}
				</Input>
			</div>
			<div class="rdz-install-wrap" bind:this={installMenuEl} data-tour="rdz-install-apworld">
				<Button
					size="md"
					variant="primary"
					onclick={() => (installMenuOpen = !installMenuOpen)}
					disabled={installing}
				>
					{#snippet icon()}
						<Icon icon={installing ? 'mdi:loading' : 'mdi:package-variant-plus'} />
					{/snippet}
					<span class="rdz-install-label">
						{i18nState.locale && m.randomizer_installApworld()}
						<Icon icon="mdi:chevron-down" />
					</span>
				</Button>
				{#if installMenuOpen}
					<div class="rdz-install-menu">
						<button class="rdz-install-item" onclick={installApworldFiles}>
							<Icon icon="mdi:file-document-multiple" />
							<div>
								<div class="rdz-install-item-title">
									{i18nState.locale && m.randomizer_installApworld_pickFiles()}
								</div>
								<div class="rdz-install-item-desc">
									{i18nState.locale && m.randomizer_installApworld_pickFilesDesc()}
								</div>
							</div>
						</button>
						<button class="rdz-install-item" onclick={installApworldFolder}>
							<Icon icon="mdi:folder-multiple" />
							<div>
								<div class="rdz-install-item-title">
									{i18nState.locale && m.randomizer_installApworld_pickFolder()}
								</div>
								<div class="rdz-install-item-desc">
									{i18nState.locale && m.randomizer_installApworld_pickFolderDesc()}
								</div>
							</div>
						</button>
					</div>
				{/if}
			</div>
			<div data-tour="rdz-manage" style="display: inline-flex;">
				<Button size="md" variant="ghost" onclick={() => (showCustom = !showCustom)}>
					{#snippet icon()}
						<Icon icon={showCustom ? 'mdi:chevron-up' : 'mdi:chevron-down'} />
					{/snippet}
					{i18nState.locale && m.randomizer_manage()}
				</Button>
			</div>
			<Button
				size="md"
				variant="ghost"
				onclick={() => randomizerStore.loadCatalog()}
				disabled={randomizerStore.catalogLoading}
			>
				{#snippet icon()}
					<Icon icon="mdi:refresh" />
				{/snippet}
				{i18nState.locale && m.randomizer_refresh()}
			</Button>
		</div>
	</header>

	<div data-tour="rdz-runtime-banner">
		<RuntimeInstallBanner />
	</div>

	<CustomApworldsPanel
		visible={showCustom}
		reloadToken={panelReloadToken}
		onAutoOpen={() => (showCustom = true)}
	/>

	{#if randomizerStore.catalogLoading && randomizerStore.catalog.length === 0}
		<div class="rdz-empty">
			<Icon icon="mdi:loading" class="rdz-spin" />
			<p>{i18nState.locale && m.randomizer_loading()}</p>
		</div>
	{:else if randomizerStore.catalog.length === 0}
		<div class="rdz-empty">
			<Icon icon="mdi:dice-multiple-outline" />
			<p>{i18nState.locale && m.randomizer_noSchemas()}</p>
			<small>Drop a JSON schema in <code>data/randomizer/schemas/</code></small>
		</div>
	{:else if filtered.length === 0}
		<div class="rdz-empty">
			<Icon icon="mdi:magnify-close" />
			<p>{i18nState.locale && m.randomizer_noMatch({ search })}</p>
			<button class="rdz-link-btn" onclick={() => (search = '')}
				>{i18nState.locale && m.randomizer_clearSearch()}</button
			>
		</div>
	{:else}
		<div class="rdz-grid" data-tour="rdz-catalog-grid">
			{#each filtered as game (game.id)}
				<button class="rdz-card" onclick={() => onSelect(game.id)}>
					<GameLogo id={game.id} name={game.name} size={64} />
					<div class="rdz-card-body">
						<div class="rdz-card-title-row">
							<h3>{game.name}</h3>
							{#if game.version && game.version !== '0.0.0'}
								<span class="rdz-card-version">v{game.version}</span>
							{/if}
						</div>
						<p class="rdz-card-desc">
							{game.description || (i18nState.locale && m.randomizer_noDescription())}
						</p>
						<div class="rdz-card-badges">
							<span class="rdz-badge">
								<Icon icon="mdi:tune" />
								{game.option_count}
							</span>
							{#if game.preset_count > 0}
								<span class="rdz-badge rdz-badge-soft">
									<Icon icon="mdi:bookmark-multiple" />
									{game.preset_count}
								</span>
							{/if}
						</div>
					</div>
					<div class="rdz-card-arrow">
						<Icon icon="mdi:arrow-right" />
					</div>
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	.rdz-catalog {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg);
		padding: var(--space-xl);
		padding-bottom: var(--space-3xl);
		width: 100%;
		flex: 1 1 0;
		min-height: 0;
		overflow-y: auto;
		overflow-x: hidden;
	}

	.rdz-catalog-header {
		display: flex;
		align-items: flex-end;
		justify-content: space-between;
		gap: var(--space-md);
		flex-wrap: wrap;
	}

	.rdz-catalog-title h1 {
		margin: 0;
		font-size: 28px;
		color: var(--text-primary);
		letter-spacing: 0.01em;
	}

	.rdz-subtitle-inline {
		font-size: 0.6em;
		font-weight: 400;
		color: var(--text-muted);
	}

	.rdz-subtitle {
		margin: var(--space-xs) 0 0;
		color: var(--text-muted);
		font-size: 13px;
	}

	.rdz-subtitle strong {
		color: var(--accent-400);
		font-weight: 700;
	}

	.rdz-catalog-actions {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
	}

	.rdz-search {
		width: 280px;
	}

	.rdz-empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-sm);
		padding: var(--space-3xl);
		color: var(--text-muted);
		font-size: 14px;
	}

	.rdz-empty :global(svg) {
		font-size: 48px;
		opacity: 0.6;
	}

	.rdz-empty :global(.rdz-spin) {
		animation: spin 1s linear infinite;
	}

	.rdz-link-btn {
		background: none;
		border: none;
		color: var(--accent-400);
		cursor: pointer;
		font-size: 13px;
		text-decoration: underline;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.rdz-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
		gap: var(--space-md);
	}

	.rdz-card {
		display: flex;
		gap: var(--space-md);
		padding: var(--space-md);
		border-radius: var(--radius-lg);
		border: 1px solid var(--border-subtle);
		background: var(--bg-surface);
		cursor: pointer;
		text-align: left;
		color: inherit;
		transition: all var(--transition-fast);
		position: relative;
		overflow: hidden;
	}

	.rdz-card::before {
		content: '';
		position: absolute;
		inset: 0;
		background: linear-gradient(135deg, var(--bg-active) 0%, transparent 40%);
		opacity: 0;
		transition: opacity var(--transition-fast);
		pointer-events: none;
	}

	.rdz-card:hover {
		border-color: var(--accent-400);
		transform: translateY(-2px);
		box-shadow:
			0 8px 24px rgba(0, 0, 0, 0.3),
			0 0 0 1px var(--accent-400),
			var(--shadow-glow);
	}

	.rdz-card:hover::before {
		opacity: 1;
	}

	.rdz-card-body {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
	}

	.rdz-card-title-row {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: var(--space-sm);
	}

	.rdz-card-title-row h3 {
		margin: 0;
		font-size: 15px;
		color: var(--text-primary);
		font-weight: 700;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		letter-spacing: 0.01em;
	}

	.rdz-card-version {
		font-size: 10px;
		color: var(--text-muted);
		font-family: var(--font-mono, monospace);
		flex-shrink: 0;
	}

	.rdz-card-desc {
		margin: 0;
		font-size: 12px;
		color: var(--text-muted);
		line-height: 1.4;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}

	.rdz-card-badges {
		display: flex;
		gap: var(--space-xs);
		margin-top: auto;
		padding-top: var(--space-xs);
	}

	.rdz-badge {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		font-size: 10px;
		font-weight: 700;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		padding: 3px 8px;
		border-radius: var(--radius-full);
		background: var(--bg-active);
		color: var(--accent-400);
		border: 1px solid var(--border-accent);
	}

	.rdz-badge :global(svg) {
		font-size: 11px;
	}

	.rdz-badge-soft {
		background: var(--bg-active);
		color: var(--text-secondary);
		border-color: var(--border-default);
	}

	.rdz-card-arrow {
		display: flex;
		align-items: center;
		color: var(--text-muted);
		opacity: 0;
		transform: translateX(-4px);
		transition: all var(--transition-fast);
	}

	.rdz-card:hover .rdz-card-arrow {
		opacity: 1;
		transform: translateX(0);
		color: var(--accent-400);
	}

	.rdz-card-arrow :global(svg) {
		font-size: 18px;
	}

	.rdz-install-wrap {
		position: relative;
	}

	.rdz-install-label {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		white-space: nowrap;
	}

	.rdz-install-label :global(svg) {
		flex-shrink: 0;
	}

	.rdz-install-menu {
		position: absolute;
		top: calc(100% + 4px);
		right: 0;
		min-width: 260px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		padding: 4px;
		box-shadow:
			0 8px 24px rgba(0, 0, 0, 0.45),
			0 2px 6px rgba(0, 0, 0, 0.3);
		z-index: var(--z-dropdown);
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.rdz-install-item {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 12px;
		background: transparent;
		border: none;
		border-radius: var(--radius-md);
		color: var(--text-primary);
		text-align: left;
		cursor: pointer;
		font-family: inherit;
		transition: background var(--transition-fast);
	}

	.rdz-install-item:hover {
		background: var(--bg-hover);
	}

	.rdz-install-item :global(svg) {
		font-size: 20px;
		color: var(--accent-400);
		flex-shrink: 0;
	}

	.rdz-install-item-title {
		font-size: 13px;
		font-weight: 600;
	}

	.rdz-install-item-desc {
		font-size: 11px;
		color: var(--text-muted);
		margin-top: 2px;
	}
</style>
