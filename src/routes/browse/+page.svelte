<script lang="ts">
	import * as api from '$lib/api';
	import type { SortBy, Mod, ModId, Dependant } from '$lib/types';
	import type { SourceId, UnifiedMod } from '$lib/api/sources';

	import Loader from '$lib/components/ui/Loader.svelte';
	import ModCard from '$lib/components/mod-list/ModCard.svelte';
	import ModDetails from '$lib/components/mod-list/ModDetails.svelte';
	import ModListFilters from '$lib/components/mod-list/ModListFilters.svelte';
	import InstallModButton from '$lib/components/mod-list/InstallModButton.svelte';
	import RemoveModDialog from '$lib/components/mod-list/RemoveModDialog.svelte';
	import ToggleModDialog from '$lib/components/mod-list/ToggleModDialog.svelte';
	import BatchActionBar from '$lib/components/ui/BatchActionBar.svelte';
	import Header from '$lib/components/layout/Header.svelte';
	import Button from '$lib/components/ui/Button.svelte';

	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { modQuery } from '$lib/state/misc.svelte';
	import ContextMenu from '$lib/components/ui/ContextMenu.svelte';
	import type { ContextMenuItem } from '$lib/components/ui/ContextMenu.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import { open } from '@tauri-apps/plugin-shell';
	import profiles from '$lib/state/profile.svelte';
	import games from '$lib/state/game.svelte';
	import Icon from '@iconify/svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { pushToast } from '$lib/toast';
	import { handleMultiSelect } from '$lib/utils/multiSelect';

	const sortOptions: SortBy[] = ['lastUpdated', 'newest', 'rating', 'downloads'];

	import { activeSourceState } from '$lib/state/source.svelte';
	let activeSource = $derived(activeSourceState.current);

	let mods: Mod[] = $state([]);
	let maxCount: number = $state(30);
	let totalLoadedForCurrentQuery: number = $state(0);

	let selectedModIds: string[] = $state([]);
	let lastClickedIndex = -1;
	let selectedMod = $derived(
		selectedModIds.length === 1 ? (mods.find((m) => m.uuid === selectedModIds[0]) ?? null) : null
	);

	// Multi-select detail navigation
	let multiViewIndex = $state(0);
	let selectedMods = $derived(
		selectedModIds.length > 1
			? selectedModIds.map((id) => mods.find((m) => m.uuid === id)).filter(Boolean) as Mod[]
			: []
	);
	let multiViewMod = $derived(selectedMods.length > 0 ? selectedMods[multiViewIndex] ?? selectedMods[0] : null);

	// Reset index when selection changes
	$effect(() => {
		selectedModIds;
		multiViewIndex = 0;
	});

	let hasRefreshed = $state(false);
	let filtersExpanded = $state(false);

	function handleModClick(evt: MouseEvent, mod: Mod, index: number) {
		const result = handleMultiSelect(
			evt,
			mod.uuid,
			index,
			selectedModIds,
			mods.map((m) => m.uuid),
			lastClickedIndex
		);
		selectedModIds = result.selection;
		lastClickedIndex = result.lastIndex;
	}

	// Remove-mod dialog state
	let removeDialog: { open: boolean; mod: Mod; dependants: Dependant[] } | null = $state(null);

	// Toggle-mod dialog state
	let toggleDialog: {
		open: boolean;
		mod: Mod;
		dependants: Dependant[];
		isDisabling: boolean;
	} | null = $state(null);

	let unlistenFromQuery: UnlistenFn | undefined;

	onMount(() => {
		listen<Mod[]>('mod_query_result', (evt) => {
			mods = evt.payload;
		}).then((unlisten) => {
			unlistenFromQuery = unlisten;
		});

		return () => {
			unlistenFromQuery?.();
			api.thunderstore.stopQuerying();
		};
	});

	let refreshing = false;
	let pendingRefresh = false;

	let lastGameSlug: string | null = null;

	async function refresh() {
		if (refreshing) {
			pendingRefresh = true;
			return;
		}

		const currentSlug = games.active?.slug ?? null;
		if (currentSlug !== lastGameSlug) {
			mods = [];
			hasRefreshed = false;
			lastGameSlug = currentSlug;
		}

		refreshing = true;

		try {
			if (activeSource === 'thunderstore') {
				mods = await api.thunderstore.query({ ...modQuery.current, maxCount });
			} else {
				const sortMap: Record<string, 'downloads' | 'rating' | 'newest' | 'updated' | 'name'> = {
					lastUpdated: 'updated',
					newest: 'newest',
					rating: 'rating',
					downloads: 'downloads',
					name: 'name'
				};

				const results = await api.sources.searchSources({
					searchTerm: modQuery.current.searchTerm,
					categories: modQuery.current.includeCategories,
					sortBy: sortMap[modQuery.current.sortBy] ?? 'downloads',
					sortOrder: modQuery.current.sortOrder === 'ascending' ? 'ascending' : 'descending',
					includeNsfw: modQuery.current.includeNsfw,
					includeDeprecated: modQuery.current.includeDeprecated,
					maxCount,
					sources: [activeSource],
					gameSlug: games.active?.slug
				});

				mods = results.flatMap((r) => r.mods.map((u) => unifiedToMod(u)));
			}
			totalLoadedForCurrentQuery = mods.length;
		} catch {}

		refreshing = false;
		hasRefreshed = true;

		if (pendingRefresh) {
			pendingRefresh = false;
			refresh();
		}
	}

	function cleanDescription(desc: string | null): string | null {
		if (!desc) return null;
		return desc
			.replace(/<br\s*\/?>/gi, ' ')
			.replace(/<[^>]*>/g, '')
			.replace(/\[.*?\]/g, '')
			.trim();
	}

	function unifiedToMod(u: UnifiedMod): Mod {
		return {
			name: u.name,
			description: cleanDescription(u.description),
			categories: u.categories,
			version: u.version,
			author: u.author,
			rating: u.rating ? Number(u.rating) : null,
			downloads: u.downloads ? Number(u.downloads) : null,
			fileSize: u.fileSize,
			websiteUrl: u.websiteUrl,
			donateUrl: null,
			dependencies: u.dependencies,
			isPinned: false,
			isDeprecated: u.isDeprecated,
			isInstalled: undefined,
			containsNsfw: u.isNsfw,
			uuid: u.externalId,
			versionUuid: u.versions[0]?.externalId ?? u.externalId,
			lastUpdated: u.dateUpdated,
			versions: u.versions.map((v) => ({ name: v.version, uuid: v.externalId })),
			type: 'remote' as any,
			enabled: null,
			icon: u.iconUrl,
			configFile: null
		};
	}

	async function installLatest(mod: Mod) {
		await install({
			packageUuid: mod.uuid,
			versionUuid: mod.versions[0].uuid
		});
	}

	async function install(id: ModId) {
		await api.profile.install.mod(id);
		await refresh();
	}

	async function toggleMod(mod: Mod) {
		const response = await api.profile.toggleMod(mod.uuid);
		if (response.type === 'done') {
			await refresh();
		} else if (response.type === 'confirm') {
			toggleDialog = {
				open: true,
				mod,
				dependants: response.dependants,
				isDisabling: mod.enabled !== false
			};
		}
	}

	async function removeMod(mod: Mod) {
		const response = await api.profile.removeMod(mod.uuid);
		if (response.type === 'done') {
			selectedModIds = selectedModIds.filter((id) => id !== mod.uuid);
			await refresh();
		} else if (response.type === 'confirm') {
			// Show cascade-delete dialog
			removeDialog = { open: true, mod, dependants: response.dependants };
		}
	}

	async function doBatchInstall() {
		if (locked) return;
		for (const uuid of selectedModIds) {
			const mod = mods.find((m) => m.uuid === uuid);
			if (mod && !mod.isInstalled && mod.versions.length > 0) {
				await api.profile.install.mod({
					packageUuid: mod.uuid,
					versionUuid: mod.versions[0].uuid
				});
			}
		}
		selectedModIds = [];
		await refresh();
	}

	function selectAll() {
		selectedModIds = mods.map((m) => m.uuid);
	}

	let isAllSelected = $derived(mods.length > 0 && selectedModIds.length === mods.length);
	function toggleSelectAll() {
		if (isAllSelected) {
			selectedModIds = [];
		} else {
			selectAll();
		}
	}

	$effect(() => {
		if (maxCount > 0) {
			JSON.stringify(modQuery.current);
			profiles.active;
			games.active;
			activeSource;
			refresh();
		}
	});

	let locked = $derived(profiles.activeLocked);

	// Context menu state
	let ctxMenu: { items: ContextMenuItem[]; x: number; y: number } | null = $state(null);

	function openModContextMenu(e: MouseEvent, mod: Mod) {
		const items: ContextMenuItem[] = [];

		// Install
		if (!mod.isInstalled && !locked) {
			items.push({
				label: m.mods_contextMenu_install(),
				icon: 'mdi:download',
				onclick: () => installLatest(mod)
			});
		}

		// Toggle enable/disable
		if (mod.isInstalled) {
			items.push({
				label: mod.enabled === false ? m.mods_contextMenu_enable() : m.mods_contextMenu_disable(),
				icon: mod.enabled === false ? 'mdi:eye' : 'mdi:eye-off',
				disabled: locked,
				onclick: () => toggleMod(mod)
			});
		}

		// Open website
		if (mod.websiteUrl) {
			items.push({
				label: m.mods_contextMenu_openThunderstore(),
				icon: 'mdi:open-in-new',
				onclick: () => open(mod.websiteUrl!)
			});
		}

		// Uninstall
		if (mod.isInstalled) {
			items.push({ label: '', separator: true });
			items.push({
				label: m.mods_contextMenu_uninstall(),
				icon: 'mdi:delete',
				danger: true,
				disabled: locked,
				onclick: () => removeMod(mod)
			});
		}

		ctxMenu = { items, x: e.clientX, y: e.clientY };
	}
</script>

<div class="z-browse-page">
	<div class="z-browse-main">
		<Header
			title={i18nState.locale && m.navBar_label_browse()}
			subtitle={'Thunderstore' /* activeSource === 'nexusmods' ? 'NexusMods' : 'Thunderstore' */}
		>
			{#snippet actions()}
				<button
					class="z-refresh-btn"
					onclick={() => {
						if (activeSource === 'thunderstore') api.thunderstore.triggerModFetch();
						refresh();
					}}
					title="Refresh"
				>
					<Icon icon="mdi:refresh" />
				</button>
			{/snippet}
		</Header>

		<div class="z-browse-content">
			<div class="z-browse-filters">
				<div class="z-browse-filters-row">
					{#if !filtersExpanded}
						<label class="z-master-checkbox-wrapper">
							<Checkbox checked={isAllSelected} onchange={toggleSelectAll} />
							<span class="z-master-checkbox-label">{i18nState.locale && m.batch_selectAll()}</span>
						</label>
					{/if}
					<div class="flex-1"></div>
					<ModListFilters
						queryArgs={modQuery.current}
						{sortOptions}
						externalPanel
						bind:expanded={filtersExpanded}
					/>
				</div>

				{#if filtersExpanded}
					<div class="z-browse-filter-panel">
						<label class="z-filter-toggle">
							<Checkbox bind:checked={modQuery.current.includeNsfw} />
							<span>{i18nState.locale && m.modListFilters_options_NSFW()}</span>
						</label>
						<label class="z-filter-toggle">
							<Checkbox bind:checked={modQuery.current.includeDeprecated} />
							<span>{i18nState.locale && m.modListFilters_options_deprecated()}</span>
						</label>

						{#if games.categories.length > 0}
							<div class="z-filter-categories">
								{#each games.categories.slice(0, 20) as cat}
									<button
										class="z-category-chip"
										class:active={modQuery.current.includeCategories.includes(cat.slug)}
										onclick={() => {
											const idx = modQuery.current.includeCategories.indexOf(cat.slug);
											if (idx >= 0) {
												modQuery.current.includeCategories =
													modQuery.current.includeCategories.filter((c) => c !== cat.slug);
											} else {
												modQuery.current.includeCategories = [
													...modQuery.current.includeCategories,
													cat.slug
												];
											}
										}}
									>
										{cat.name}
									</button>
								{/each}
							</div>
						{/if}
					</div>

					<div class="z-browse-select-row">
						<label class="z-master-checkbox-wrapper">
							<Checkbox checked={isAllSelected} onchange={toggleSelectAll} />
							<span class="z-master-checkbox-label">{i18nState.locale && m.batch_selectAll()}</span>
						</label>
						<span class="z-browse-count">{mods.length} mods</span>
					</div>
				{/if}
			</div>

			{#if locked}
				<div class="z-locked-banner">
					<Icon icon="mdi:lock" />
					<span>{i18nState.locale && m.browse_lockedBanner()}</span>
				</div>
			{/if}

			<div class="z-browse-list">
				{#if mods.length === 0 && hasRefreshed}
					<div class="z-browse-empty">
						<div class="z-browse-empty-icon">
							<Icon icon="mdi:package-variant-remove" />
						</div>
						<p class="z-browse-empty-title">{i18nState.locale && m.browse_noMods()}</p>
						<p class="z-browse-empty-desc">{i18nState.locale && m.browse_noMods_desc()}</p>
					</div>
				{:else if mods.length === 0}
					<Loader />
				{:else}
					{#each mods as mod, index (mod.uuid)}
						<ModCard
							{mod}
							isSelected={selectedModIds.includes(mod.uuid)}
							{locked}
							onclick={(evt: MouseEvent) => handleModClick(evt, mod, index)}
							oninstall={() => installLatest(mod)}
							oncontextmenu={openModContextMenu}
						/>
					{/each}

					<button
						class="z-load-more"
						onclick={() => (maxCount += 30)}
						disabled={mods.length < maxCount}
						title={mods.length < maxCount ? 'No more mods to load' : 'Load more mods'}
					>
						{i18nState.locale && m.browse_loadMore()}
					</button>
				{/if}
			</div>
		</div>
	</div>

	{#if selectedMod}
		<ModDetails
			mod={selectedMod}
			{locked}
			showVersionSelector={false}
			onclose={() => (selectedModIds = [])}
			ontoggle={activeSource === 'thunderstore' ? () => toggleMod(selectedMod!) : undefined}
			onremove={activeSource === 'thunderstore' ? () => removeMod(selectedMod!) : undefined}
		>
			{#if activeSource === 'thunderstore'}
				<InstallModButton mod={selectedMod} {install} {locked} />
			{/if}
		</ModDetails>
	{:else if multiViewMod}
		<ModDetails
			mod={multiViewMod}
			{locked}
			showVersionSelector={false}
			onclose={() => (selectedModIds = [])}
			ontoggle={activeSource === 'thunderstore' ? () => toggleMod(multiViewMod!) : undefined}
			onremove={activeSource === 'thunderstore' ? () => removeMod(multiViewMod!) : undefined}
		>
			{#if activeSource === 'thunderstore'}
				<InstallModButton mod={multiViewMod} {install} {locked} />
			{/if}
			<div class="z-multi-nav">
				<button
					class="z-multi-nav-btn"
					disabled={multiViewIndex <= 0}
					onclick={() => multiViewIndex--}
					title="Previous mod"
				>
					<Icon icon="mdi:chevron-left" />
				</button>
				<span class="z-multi-nav-label">{multiViewIndex + 1} / {selectedMods.length}</span>
				<button
					class="z-multi-nav-btn"
					disabled={multiViewIndex >= selectedMods.length - 1}
					onclick={() => multiViewIndex++}
					title="Next mod"
				>
					<Icon icon="mdi:chevron-right" />
				</button>
			</div>
		</ModDetails>
	{/if}

	<BatchActionBar
		count={selectedModIds.length}
		total={mods.length}
		onclear={() => (selectedModIds = [])}
		onselectAll={selectAll}
	>
		{#snippet actions()}
			<Button variant="accent" size="sm" onclick={doBatchInstall} disabled={locked}>
				{#snippet icon()}<Icon icon="mdi:download" />{/snippet}
				{i18nState.locale && m.batch_installSelected()}
			</Button>
		{/snippet}
	</BatchActionBar>
</div>

<!-- Remove mod confirmation dialog -->
{#if removeDialog}
	<RemoveModDialog
		bind:open={removeDialog.open}
		modName={removeDialog.mod.name}
		dependants={removeDialog.dependants}
		oncancel={() => (removeDialog = null)}
		onremoveOne={async () => {
			await api.profile.forceRemoveMods([removeDialog!.mod.uuid]);
			selectedModIds = selectedModIds.filter((id) => id !== removeDialog!.mod.uuid);
			removeDialog = null;
			await refresh();
		}}
		onremoveAll={async () => {
			const uuids = [removeDialog!.mod.uuid, ...removeDialog!.dependants.map((d) => d.uuid)];
			await api.profile.forceRemoveMods(uuids);
			selectedModIds = selectedModIds.filter((id) => !uuids.includes(id));
			removeDialog = null;
			await refresh();
		}}
	/>
{/if}

<!-- Context menu -->
{#if ctxMenu}
	<ContextMenu items={ctxMenu.items} x={ctxMenu.x} y={ctxMenu.y} onclose={() => (ctxMenu = null)} />
{/if}

<!-- Toggle mod confirmation dialog -->
{#if toggleDialog}
	<ToggleModDialog
		bind:open={toggleDialog.open}
		modName={toggleDialog.mod.name}
		isDisabling={toggleDialog.isDisabling}
		dependants={toggleDialog.dependants}
		oncancel={() => (toggleDialog = null)}
		ontoggleOne={async () => {
			await api.profile.forceToggleMods([toggleDialog!.mod.uuid]);
			toggleDialog = null;
			await refresh();
		}}
		ontoggleAll={async () => {
			const uuids = [toggleDialog!.mod.uuid, ...toggleDialog!.dependants.map((d) => d.uuid)];
			await api.profile.forceToggleMods(uuids);
			toggleDialog = null;
			await refresh();
		}}
	/>
{/if}

<style>
	.z-browse-page {
		display: flex;
		height: 100%;
		overflow: hidden;
	}

	.z-browse-main {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		min-width: 0;
	}

	.z-browse-content {
		flex: 1;
		overflow-y: auto;
		padding: 0 var(--space-xl);
		padding-bottom: var(--space-xl);
	}

	.z-browse-filters {
		position: sticky;
		top: 0;
		z-index: 10;
		padding-top: var(--space-sm);
		padding-bottom: var(--space-xs);
		background: var(--bg-base);
		border-bottom: 1px solid var(--border-subtle);
		margin-bottom: var(--space-sm);
	}

	.z-browse-filters-row {
		display: flex;
		align-items: center;
		gap: var(--space-md);
	}

	.z-master-checkbox-wrapper {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-xs) var(--space-sm);
		cursor: pointer;
		color: var(--text-muted);
		font-size: 13px;
		font-weight: 500;
		transition: color var(--transition-fast);
	}

	.z-master-checkbox-wrapper:hover {
		color: var(--text-primary);
	}

	.z-master-checkbox-label {
		user-select: none;
	}

	.z-browse-filter-panel {
		display: flex;
		flex-wrap: wrap;
		justify-content: flex-start;
		gap: var(--space-sm);
		padding: var(--space-md);
		border-radius: var(--radius-md);
		background: var(--bg-elevated);
		border: 1px solid var(--border-subtle);
		margin-top: var(--space-sm);
	}

	.z-filter-toggle {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
		color: var(--text-secondary);
		cursor: pointer;
		padding: 4px 10px;
		border-radius: var(--radius-sm);
		transition: background var(--transition-fast);
	}

	.z-filter-toggle:hover {
		background: var(--bg-hover);
	}

	.z-filter-categories {
		display: flex;
		flex-wrap: wrap;
		justify-content: flex-start;
		gap: 4px;
		width: 100%;
		padding-top: var(--space-sm);
		border-top: 1px solid var(--border-subtle);
	}

	.z-category-chip {
		padding: 3px 10px;
		border-radius: var(--radius-full);
		font-size: 11px;
		border: 1px solid var(--border-subtle);
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.z-category-chip:hover {
		border-color: var(--border-default);
		color: var(--text-secondary);
	}

	.z-category-chip.active {
		background: var(--bg-active);
		border-color: var(--border-accent);
		color: var(--text-accent);
	}

	.z-browse-select-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding-top: var(--space-xs);
	}

	.z-browse-count {
		font-size: 11px;
		color: var(--text-muted);
	}

	.z-refresh-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		border-radius: var(--radius-md);
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
		font-size: 18px;
	}

	.z-refresh-btn:hover {
		color: var(--text-accent);
		border-color: var(--border-accent);
	}

	.z-locked-banner {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-md);
		background: rgba(255, 179, 71, 0.08);
		border: 1px solid rgba(255, 179, 71, 0.2);
		color: var(--warning);
		font-size: 12px;
		margin-bottom: var(--space-sm);
	}

	.z-browse-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.z-browse-empty,
	.z-browse-loading {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: var(--space-3xl);
		gap: var(--space-sm);
	}

	.z-browse-empty-icon {
		width: 64px;
		height: 64px;
		border-radius: var(--radius-xl);
		background: var(--bg-elevated);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 28px;
		color: var(--text-muted);
	}

	.z-browse-empty-title {
		font-size: 15px;
		font-weight: 600;
		color: var(--text-secondary);
	}
	.z-browse-empty-desc {
		font-size: 13px;
		color: var(--text-muted);
	}

	.z-browse-loading {
		flex-direction: row;
		gap: var(--space-md);
		color: var(--text-muted);
		font-size: 13px;
	}

	.z-browse-spinner {
		width: 18px;
		height: 18px;
		border: 2px solid var(--text-muted);
		border-top-color: transparent;
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.z-load-more {
		padding: var(--space-md);
		text-align: center;
		font-size: 12px;
		color: var(--text-muted);
		background: transparent;
		border: 1px dashed var(--border-subtle);
		border-radius: var(--radius-md);
		cursor: pointer;
		transition: all var(--transition-fast);
		margin-top: var(--space-sm);
	}

	.z-load-more:hover:not(:disabled) {
		border-color: var(--border-accent);
		color: var(--text-accent);
		background: var(--bg-hover);
	}

	.z-load-more:disabled {
		opacity: 0.5;
		cursor: not-allowed;
		border-style: dotted;
	}

	.z-multi-nav {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-sm);
		padding: 8px 0;
	}

	.z-multi-nav-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		color: var(--text-secondary);
		cursor: pointer;
		transition: all var(--transition-fast);
		font-size: 18px;
	}

	.z-multi-nav-btn:hover:not(:disabled) {
		background: var(--bg-hover);
		color: var(--text-primary);
		border-color: var(--border-strong);
	}

	.z-multi-nav-btn:disabled {
		opacity: 0.35;
		cursor: not-allowed;
	}

	.z-multi-nav-label {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-secondary);
		min-width: 50px;
		text-align: center;
		font-family: var(--font-body);
	}
</style>
