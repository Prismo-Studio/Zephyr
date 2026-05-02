<script lang="ts">
	import * as api from '$lib/api';
	import type { SortBy, Mod, ModId, Dependant } from '$lib/types';
	import { curseForgeEnabled } from '$lib/themeSystem';
	import * as zephyrServer from '$lib/api/zephyrServer';
	import { zephyrServerState } from '$lib/state/zephyrServer.svelte';

	import Loader from '$lib/components/ui/Loader.svelte';
	import ScrollToTop from '$lib/components/ui/ScrollToTop.svelte';
	import ModCard from '$lib/components/mod-list/ModCard.svelte';
	import ModDetails from '$lib/components/mod-list/ModDetails.svelte';
	import InstallModButton from '$lib/components/mod-list/InstallModButton.svelte';
	import RemoveModDialog from '$lib/components/mod-list/RemoveModDialog.svelte';
	import ToggleModDialog from '$lib/components/mod-list/ToggleModDialog.svelte';
	import MultiViewNav from '$lib/components/mod-list/MultiViewNav.svelte';
	import BatchActionBar from '$lib/components/ui/BatchActionBar.svelte';
	import Header from '$lib/components/layout/Header.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import ContextMenu from '$lib/components/ui/ContextMenu.svelte';

	import BrowseFilterBar from '$lib/components/browse/BrowseFilterBar.svelte';
	import BrowseEmptyState from '$lib/components/browse/BrowseEmptyState.svelte';
	import BrowseLockedBanner from '$lib/components/browse/BrowseLockedBanner.svelte';

	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { modQuery, installState, viewMode } from '$lib/state/misc.svelte';
	import { activeSourceState } from '$lib/state/source.svelte';
	import profiles from '$lib/state/profile.svelte';
	import games from '$lib/state/game.svelte';
	import Icon from '@iconify/svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { pushToast } from '$lib/toast.svelte';
	import { handleMultiSelect } from '$lib/utils/multiSelect';
	import { gamepadState } from '$lib/gamepad.svelte';
	import { curseForgeModToMod, unifiedToMod, isServerMod } from '$lib/utils/sourceMappers';
	import { buildBrowseContextMenu } from '$lib/utils/browseContextMenu';

	const sortOptions: SortBy[] = ['lastUpdated', 'newest', 'rating', 'downloads'];
	const CF_PAGE_SIZE = 25;
	const SERVER_PAGE_SIZE = 50;

	type SourceSortBy = 'downloads' | 'rating' | 'newest' | 'updated' | 'name';
	const sourceSortMap: Record<string, SourceSortBy> = {
		lastUpdated: 'updated',
		newest: 'newest',
		rating: 'rating',
		downloads: 'downloads',
		name: 'name'
	};

	let activeSource = $derived(activeSourceState.current);

	let browseContentEl: HTMLDivElement | undefined = $state();
	let mods: Mod[] = $state([]);
	let maxCount: number = $state(30);

	let selectedModIds: string[] = $state([]);
	let lastClickedIndex = -1;
	let cachedSelectedMods: Map<string, Mod> = new Map();

	type PaginatedSource = {
		items: Mod[];
		offset: number;
		hasMore: boolean;
		loading: boolean;
	};
	const emptyPage = (): PaginatedSource => ({
		items: [],
		offset: 0,
		hasMore: true,
		loading: false
	});

	let thunderstoreMods: Mod[] = $state([]);
	let cfPage: PaginatedSource = $state(emptyPage());
	let serverPage: PaginatedSource = $state(emptyPage());
	let zephyrServerReachable: boolean | null = $state(null);
	let communityMods: Mod[] = $state([]);

	let showCurseForgeOnly = $state(false);
	let hasRefreshed = $state(false);
	let filtersExpanded = $state(false);

	let displayedMods = $derived(
		showCurseForgeOnly
			? mods.filter((mm) => mm.uuid.startsWith('curseforge:') || isServerMod(mm))
			: mods
	);

	let prevInstallActive = false;
	$effect(() => {
		if (prevInstallActive && !installState.active) {
			const cachedIds = [...cachedSelectedMods.entries()].filter(
				([uuid]) => !mods.find((mm) => mm.uuid === uuid)
			);
			if (cachedIds.length > 0) {
				Promise.all(
					cachedIds.map(([uuid, cached]) =>
						api.thunderstore
							.query({
								searchTerm: cached.name,
								includeCategories: [],
								excludeCategories: [],
								includeNsfw: true,
								includeDeprecated: true,
								includeDisabled: true,
								includeEnabled: true,
								sortBy: 'downloads',
								sortOrder: 'descending',
								maxCount: 5
							})
							.then((results) => {
								const updated = results.find((mm) => mm.uuid === uuid);
								if (updated) cachedSelectedMods.set(uuid, updated);
							})
					)
				).then(() => {
					selectedModIds = [...selectedModIds];
				});
			}
			refresh();
		}
		prevInstallActive = installState.active;
	});

	function getSelectedMod(uuid: string): Mod | null {
		const fresh = mods.find((mm) => mm.uuid === uuid);
		if (fresh) {
			cachedSelectedMods.set(uuid, fresh);
			return fresh;
		}
		return cachedSelectedMods.get(uuid) ?? null;
	}

	let selectedMod = $derived.by(() => {
		mods;
		return selectedModIds.length === 1 ? getSelectedMod(selectedModIds[0]) : null;
	});

	let multiViewIndex = $state(0);
	let selectedMods = $derived(
		selectedModIds.length > 1
			? (selectedModIds.map((id) => getSelectedMod(id)).filter(Boolean) as Mod[])
			: []
	);

	$effect(() => {
		if (multiViewIndex >= selectedMods.length && selectedMods.length > 0) {
			multiViewIndex = selectedMods.length - 1;
		}
	});

	let multiViewMod = $derived(
		selectedMods.length > 0 ? (selectedMods[multiViewIndex] ?? selectedMods[0]) : null
	);

	$effect(() => {
		selectedModIds;
		multiViewIndex = 0;
	});

	function toggleCategoryFilter(category: string, multi = false) {
		const cats = modQuery.current.includeCategories;
		if (multi) {
			modQuery.current.includeCategories = cats.includes(category)
				? cats.filter((c) => c !== category)
				: [...cats, category];
		} else {
			if (cats.includes(category) && cats.length === 1) {
				modQuery.current.includeCategories = [];
			} else {
				modQuery.current.includeCategories = [category];
			}
		}
		filtersExpanded = true;
	}

	function shouldUseZephyrServer(): boolean {
		return zephyrServerState.current.enabled && zephyrServerReachable !== false;
	}

	function canInstallDirectly(mod: Mod): boolean {
		return isServerMod(mod) || !mod.uuid.includes(':');
	}

	function syncBrowseMods() {
		if (activeSource !== 'thunderstore') return;
		mods = [
			...thunderstoreMods,
			...(shouldUseZephyrServer() ? serverPage.items : cfPage.items),
			...communityMods
		];
	}

	async function ensureZephyrServerAvailable(force = false): Promise<boolean> {
		if (!zephyrServerState.current.enabled) {
			zephyrServerReachable = null;
			return false;
		}
		if (!force && zephyrServerReachable !== null) {
			return zephyrServerReachable;
		}
		const ok = await zephyrServer.healthCheck();
		zephyrServerReachable = ok;
		return ok;
	}

	$effect(() => {
		zephyrServerState.current.enabled;
		zephyrServerState.current.baseUrl;
		zephyrServerState.current.apiKey;
		zephyrServerReachable = null;
	});

	function handleModClick(evt: MouseEvent, mod: Mod, index: number) {
		cachedSelectedMods.set(mod.uuid, mod);
		const result = handleMultiSelect(
			evt,
			mod.uuid,
			index,
			selectedModIds,
			mods.map((mm) => mm.uuid),
			lastClickedIndex
		);
		selectedModIds = result.selection;
		lastClickedIndex = result.lastIndex;
	}

	async function handleDepClick(author: string, name: string): Promise<boolean> {
		let found = mods.find(
			(mm) =>
				mm.name.toLowerCase() === name.toLowerCase() &&
				mm.author?.toLowerCase() === author.toLowerCase()
		);
		if (!found) found = mods.find((mm) => mm.name.toLowerCase() === name.toLowerCase());
		if (!found) {
			try {
				const results = await api.thunderstore.query({
					searchTerm: name,
					includeCategories: [],
					excludeCategories: [],
					includeNsfw: true,
					includeDeprecated: true,
					includeDisabled: true,
					includeEnabled: true,
					sortBy: 'downloads',
					sortOrder: 'descending',
					maxCount: 10
				});
				found =
					results.find(
						(mm) =>
							mm.name.toLowerCase() === name.toLowerCase() &&
							mm.author?.toLowerCase() === author.toLowerCase()
					) ?? results.find((mm) => mm.name.toLowerCase() === name.toLowerCase());
			} catch {}
		}
		if (found) {
			cachedSelectedMods.set(found.uuid, found);
			selectedModIds = [found.uuid];
			return true;
		}
		return false;
	}

	let removeDialog: { open: boolean; mod: Mod; dependants: Dependant[] } | null = $state(null);
	let toggleDialog: {
		open: boolean;
		mod: Mod;
		dependants: Dependant[];
		isDisabling: boolean;
	} | null = $state(null);

	let unlistenFromQuery: UnlistenFn | undefined;

	onMount(() => {
		listen<Mod[]>('mod_query_result', (evt) => {
			if (activeSource !== 'thunderstore') return;
			thunderstoreMods = evt.payload;
			syncBrowseMods();
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
	let lastQueryHash: string = '';

	function resetState() {
		mods = [];
		thunderstoreMods = [];
		cfPage = emptyPage();
		serverPage = emptyPage();
		communityMods = [];
		hasRefreshed = false;
		maxCount = 30;
		showCurseForgeOnly = false;
		selectedModIds = [];
		cachedSelectedMods.clear();
		multiViewIndex = 0;
	}

	function resetPagination() {
		thunderstoreMods = [];
		cfPage = emptyPage();
		serverPage = emptyPage();
	}

	async function refresh() {
		if (refreshing) {
			pendingRefresh = true;
			return;
		}

		const currentSlug = games.active?.slug ?? null;
		if (currentSlug !== lastGameSlug) {
			resetState();
			lastGameSlug = currentSlug;
		}

		const queryHash = JSON.stringify(modQuery.current);
		if (queryHash !== lastQueryHash) {
			lastQueryHash = queryHash;
			resetPagination();
		}

		refreshing = true;
		try {
			if (activeSource === 'thunderstore') {
				await refreshThunderstore();
			} else {
				await refreshExternalSource();
			}
		} catch {}
		refreshing = false;
		hasRefreshed = true;

		if (pendingRefresh) {
			pendingRefresh = false;
			refresh();
		}
	}

	async function refreshThunderstore() {
		thunderstoreMods = await api.thunderstore.query({ ...modQuery.current, maxCount });

		if (zephyrServerState.current.enabled) {
			const serverAvailable = await ensureZephyrServerAvailable();
			if (serverAvailable) {
				cfPage.items = [];
				if (serverPage.items.length === 0) {
					serverPage.offset = 0;
					await loadMoreFromServer();
				}
			} else {
				serverPage.items = [];
				if (cfPage.items.length === 0) {
					cfPage.offset = 0;
					await loadMoreCF();
				}
			}
		} else if (curseForgeEnabled.current) {
			serverPage.items = [];
			if (cfPage.items.length === 0) {
				cfPage.offset = 0;
				await loadMoreCF();
			}
		} else {
			cfPage.items = [];
			serverPage.items = [];
		}

		syncBrowseMods();
	}

	async function refreshExternalSource() {
		const results = await api.sources.searchSources({
			searchTerm: modQuery.current.searchTerm,
			categories: modQuery.current.includeCategories,
			sortBy: sourceSortMap[modQuery.current.sortBy] ?? 'downloads',
			sortOrder: modQuery.current.sortOrder === 'ascending' ? 'ascending' : 'descending',
			includeNsfw: modQuery.current.includeNsfw,
			includeDeprecated: modQuery.current.includeDeprecated,
			maxCount,
			sources: [activeSource],
			gameSlug: games.active?.slug
		});
		thunderstoreMods = [];
		mods = results.flatMap((r) => r.mods.map((u) => unifiedToMod(u)));
	}

	async function loadMoreCF() {
		cfPage.loading = true;
		try {
			const cfResults = await api.sources.searchSources({
				searchTerm: modQuery.current.searchTerm,
				categories: [],
				sortBy: sourceSortMap[modQuery.current.sortBy] ?? 'downloads',
				sortOrder: modQuery.current.sortOrder === 'ascending' ? 'ascending' : 'descending',
				includeNsfw: modQuery.current.includeNsfw,
				includeDeprecated: modQuery.current.includeDeprecated,
				maxCount: CF_PAGE_SIZE,
				sources: ['curseforge'],
				gameSlug: games.active?.slug,
				offset: cfPage.offset
			});
			const newMods = cfResults.flatMap((r) => r.mods.map((u) => unifiedToMod(u, 'curseforge')));
			cfPage.items = [...cfPage.items, ...newMods];
			cfPage.offset += CF_PAGE_SIZE;
			cfPage.hasMore = newMods.length >= CF_PAGE_SIZE;
			syncBrowseMods();
		} catch {}
		cfPage.loading = false;
	}

	async function loadMoreFromServer() {
		if (!zephyrServerState.current.enabled) return;
		serverPage.loading = true;
		try {
			const searchTerm = modQuery.current.searchTerm || '';
			const cfGameId = zephyrServer.getCurseForgeGameId(games.active?.slug);
			if (cfGameId === null) {
				serverPage.hasMore = false;
				return;
			}
			const page = Math.floor(serverPage.offset / SERVER_PAGE_SIZE);
			const result = await zephyrServer.searchMods(searchTerm, cfGameId, page, SERVER_PAGE_SIZE);
			const newMods = result.data.map(curseForgeModToMod);
			serverPage.items = [...serverPage.items, ...newMods];
			serverPage.offset += SERVER_PAGE_SIZE;
			serverPage.hasMore = serverPage.offset < result.pagination.totalCount;
			syncBrowseMods();
		} catch (err) {
			zephyrServerReachable = false;
			console.warn('[ZephyrServer] Failed to load mods:', err);
		}
		serverPage.loading = false;
	}

	async function installFromServer(mod: Mod, versionUuid?: string) {
		if (!isServerMod(mod)) return;
		const modId = parseInt(mod.uuid.replace('zephyr-server:', ''), 10);
		const fileId = parseInt(versionUuid ?? mod.versionUuid ?? mod.versions[0]?.uuid ?? '0', 10);
		if (!modId || !fileId) return;

		try {
			let gameDir: string | undefined;
			try {
				gameDir = await api.profile.launch.getGameDir();
			} catch {
				// Fall back to the server's default install path
			}
			await zephyrServer.installMod(modId, fileId, gameDir);
			pushToast({ type: 'info', name: mod.name, message: 'Mod installed via Zephyr Server' });
		} catch (err) {
			pushToast({
				type: 'error',
				name: 'Install failed',
				message: err instanceof Error ? err.message : 'Unknown error'
			});
		}
	}

	async function installLatest(mod: Mod) {
		if (isServerMod(mod)) {
			await installFromServer(mod);
			return;
		}
		await install({ packageUuid: mod.uuid, versionUuid: mod.versions[0].uuid });
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
			await refresh();
		} else if (response.type === 'confirm') {
			removeDialog = { open: true, mod, dependants: response.dependants };
		}
	}

	async function doBatchInstall() {
		if (locked) return;
		for (const uuid of selectedModIds) {
			const mod = mods.find((mm) => mm.uuid === uuid);
			if (mod && !mod.isInstalled && mod.versions.length > 0) {
				if (isServerMod(mod)) {
					await installFromServer(mod);
				} else {
					await api.profile.install.mod({
						packageUuid: mod.uuid,
						versionUuid: mod.versions[0].uuid
					});
				}
			}
		}
		selectedModIds = [];
		await refresh();
	}

	function selectAll() {
		selectedModIds = displayedMods.map((mm) => mm.uuid);
	}

	let isAllSelected = $derived(
		displayedMods.length > 0 && selectedModIds.length === displayedMods.length
	);

	function toggleSelectAll() {
		selectedModIds = isAllSelected ? [] : displayedMods.map((mm) => mm.uuid);
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

	let ctxMenu: { items: ReturnType<typeof buildBrowseContextMenu>; x: number; y: number } | null =
		$state(null);

	function openModContextMenu(e: MouseEvent, mod: Mod) {
		ctxMenu = {
			items: buildBrowseContextMenu({
				mod,
				locked,
				mods,
				selectedModIds,
				handlers: { installLatest, toggleMod, removeMod, doBatchInstall, refresh }
			}),
			x: e.clientX,
			y: e.clientY
		};
	}

	function loadMore() {
		if (showCurseForgeOnly) {
			if (shouldUseZephyrServer()) {
				loadMoreFromServer();
			} else {
				loadMoreCF();
			}
			return;
		}
		maxCount += 30;
		if (shouldUseZephyrServer() && serverPage.hasMore) {
			loadMoreFromServer();
		} else if (curseForgeEnabled.current && cfPage.hasMore) {
			loadMoreCF();
		} else if (zephyrServerState.current.enabled && cfPage.hasMore) {
			loadMoreCF();
		}
	}

	let externalHasMore = $derived(shouldUseZephyrServer() ? serverPage.hasMore : cfPage.hasMore);
	let externalLoading = $derived(shouldUseZephyrServer() ? serverPage.loading : cfPage.loading);
	let showCfToggle = $derived(
		(curseForgeEnabled.current && cfPage.items.length > 0) ||
			(zephyrServerState.current.enabled && serverPage.items.length > 0)
	);
	let showLoadMore = $derived(
		showCurseForgeOnly
			? externalHasMore
			: displayedMods.length >= maxCount ||
					(shouldUseZephyrServer()
						? serverPage.hasMore
						: curseForgeEnabled.current && cfPage.hasMore)
	);
</script>

<div class="z-browse-page">
	<div class="z-browse-main">
		<Header title={i18nState.locale && m.navBar_label_browse()} subtitle={'Thunderstore'}>
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

		<div class="z-browse-content" bind:this={browseContentEl}>
			<BrowseFilterBar
				bind:expanded={filtersExpanded}
				bind:viewMode={viewMode.current}
				{sortOptions}
				{isAllSelected}
				visibleCount={displayedMods.length}
				bind:showCurseForgeOnly
				showCurseForgeToggle={showCfToggle}
				ontoggleSelectAll={toggleSelectAll}
			/>

			{#if locked}
				<BrowseLockedBanner />
			{/if}

			<div class="z-browse-list" class:z-grid-layout={viewMode.current === 'grid'}>
				{#if mods.length === 0 && !hasRefreshed}
					<Loader />
				{:else if cfPage.loading && showCurseForgeOnly && cfPage.items.length === 0}
					<Loader />
				{:else if displayedMods.length === 0 && hasRefreshed}
					<BrowseEmptyState curseForgeOnly={showCurseForgeOnly} />
				{:else}
					{#each displayedMods as mod, index (mod.uuid)}
						<ModCard
							{mod}
							isSelected={selectedModIds.includes(mod.uuid)}
							{locked}
							showInstallBtn={canInstallDirectly(mod)}
							viewMode={viewMode.current}
							onclick={(evt: MouseEvent) => handleModClick(evt, mod, index)}
							oninstall={() => installLatest(mod)}
							oncontextmenu={openModContextMenu}
							oncategoryclick={toggleCategoryFilter}
							activeCategories={modQuery.current.includeCategories}
						/>
					{/each}

					{#if showLoadMore}
						<button
							class="z-load-more"
							onclick={loadMore}
							disabled={showCurseForgeOnly && externalLoading}
						>
							{showCurseForgeOnly && externalLoading
								? 'Loading...'
								: i18nState.locale && m.browse_loadMore()}
						</button>
					{/if}
				{/if}
			</div>
		</div>
		<ScrollToTop target={browseContentEl} lifted={selectedModIds.length >= 2} />
	</div>

	{#if selectedMod}
		<ModDetails
			mod={selectedMod}
			{locked}
			showVersionSelector={false}
			onclose={() => (selectedModIds = [])}
			ontoggle={!selectedMod.uuid.includes(':') ? () => toggleMod(selectedMod!) : undefined}
			onremove={!selectedMod.uuid.includes(':') ? () => removeMod(selectedMod!) : undefined}
			oncategoryclick={toggleCategoryFilter}
			ondepclick={handleDepClick}
			activeCategories={modQuery.current.includeCategories}
		>
			{#if isServerMod(selectedMod)}
				<InstallModButton mod={selectedMod} {locked} onInstall={installFromServer} />
			{:else if !selectedMod.uuid.includes(':')}
				<InstallModButton mod={selectedMod} {install} {locked} />
			{/if}
		</ModDetails>
	{:else if multiViewMod}
		<ModDetails
			mod={multiViewMod}
			{locked}
			showVersionSelector={false}
			onclose={() => (selectedModIds = [])}
			ontoggle={!multiViewMod.uuid.includes(':') ? () => toggleMod(multiViewMod!) : undefined}
			onremove={!multiViewMod.uuid.includes(':') ? () => removeMod(multiViewMod!) : undefined}
			oncategoryclick={toggleCategoryFilter}
			ondepclick={handleDepClick}
			activeCategories={modQuery.current.includeCategories}
		>
			{#if isServerMod(multiViewMod)}
				<InstallModButton mod={multiViewMod} {locked} onInstall={installFromServer} />
			{:else if !multiViewMod.uuid.includes(':')}
				<InstallModButton mod={multiViewMod} {install} {locked} />
			{/if}
			<MultiViewNav
				index={multiViewIndex}
				total={selectedMods.length}
				onprev={() => multiViewIndex--}
				onnext={() => multiViewIndex++}
			/>
		</ModDetails>
	{/if}

	<BatchActionBar
		count={selectedModIds.length}
		total={displayedMods.length}
		onclear={() => (selectedModIds = [])}
		onselectAll={selectAll}
		legendActive={!!(gamepadState.enabled && gamepadState.connected)}
	>
		{#snippet actions()}
			<Button variant="accent" size="sm" onclick={doBatchInstall} disabled={locked}>
				{#snippet icon()}<Icon icon="mdi:download" />{/snippet}
				{i18nState.locale && m.batch_installSelected()}
			</Button>
		{/snippet}
	</BatchActionBar>
</div>

{#if removeDialog}
	<RemoveModDialog
		bind:open={removeDialog.open}
		modName={removeDialog.mod.name}
		dependants={removeDialog.dependants}
		oncancel={() => (removeDialog = null)}
		onremoveOne={async () => {
			await api.profile.forceRemoveMods([removeDialog!.mod.uuid]);
			removeDialog = null;
			await refresh();
		}}
		onremoveAll={async () => {
			const uuids = [removeDialog!.mod.uuid, ...removeDialog!.dependants.map((d) => d.uuid)];
			await api.profile.forceRemoveMods(uuids);
			removeDialog = null;
			await refresh();
		}}
	/>
{/if}

{#if ctxMenu}
	<ContextMenu items={ctxMenu.items} x={ctxMenu.x} y={ctxMenu.y} onclose={() => (ctxMenu = null)} />
{/if}

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
		position: relative;
	}

	.z-browse-content {
		flex: 1;
		overflow-y: auto;
		padding: 0 var(--space-xl);
		padding-bottom: var(--space-xl);
		display: flex;
		flex-direction: column;
		min-height: 0;
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

	.z-browse-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
		flex: 1;
	}

	.z-browse-list.z-grid-layout {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
		grid-auto-rows: min-content;
		gap: var(--space-md);
	}

	.z-browse-list.z-grid-layout > :global(.z-loader) {
		grid-column: 1 / -1;
	}

	.z-browse-list > :global(.z-loader) {
		flex: 1;
		min-height: 50vh;
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
</style>
