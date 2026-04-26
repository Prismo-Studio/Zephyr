<script lang="ts">
	import * as api from '$lib/api';
	import type { SortBy, Mod, ModId, Dependant } from '$lib/types';
	import type { SourceId, UnifiedMod } from '$lib/api/sources';
	import { curseForgeEnabled } from '$lib/themeSystem';
	import * as zephyrServer from '$lib/api/zephyrServer';
	import { zephyrServerState } from '$lib/state/zephyrServer.svelte';

	import Loader from '$lib/components/ui/Loader.svelte';
	import ScrollToTop from '$lib/components/ui/ScrollToTop.svelte';
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
	import { modQuery, installState, viewMode } from '$lib/state/misc.svelte';
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
	import { gamepadState } from '$lib/gamepad.svelte';

	const sortOptions: SortBy[] = ['lastUpdated', 'newest', 'rating', 'downloads'];

	import { activeSourceState } from '$lib/state/source.svelte';
	let activeSource = $derived(activeSourceState.current);

	let browseContentEl: HTMLDivElement | undefined = $state();
	let mods: Mod[] = $state([]);
	let maxCount: number = $state(30);
	let totalLoadedForCurrentQuery: number = $state(0);

	let selectedModIds: string[] = $state([]);
	let lastClickedIndex = -1;
	let cachedSelectedMods: Map<string, Mod> = new Map();

	let prevInstallActive = false;
	$effect(() => {
		if (prevInstallActive && !installState.active) {
			const cachedIds = [...cachedSelectedMods.entries()].filter(
				([uuid]) => !mods.find((m) => m.uuid === uuid)
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
								const updated = results.find((m) => m.uuid === uuid);
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
		const fresh = mods.find((m) => m.uuid === uuid);
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

	// Multi-select detail navigation
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

	// Reset index when selection changes
	$effect(() => {
		selectedModIds;
		multiViewIndex = 0;
	});

	let hasRefreshed = $state(false);
	let filtersExpanded = $state(false);

	function toggleCategoryFilter(category: string, multi = false) {
		const cats = modQuery.current.includeCategories;
		if (multi) {
			if (cats.includes(category)) {
				modQuery.current.includeCategories = cats.filter((c) => c !== category);
			} else {
				modQuery.current.includeCategories = [...cats, category];
			}
		} else {
			if (cats.includes(category) && cats.length === 1) {
				modQuery.current.includeCategories = [];
			} else {
				modQuery.current.includeCategories = [category];
			}
		}
		filtersExpanded = true;
	}
	let showCurseForgeOnly = $state(false);
	type PaginatedSource = {
		items: Mod[];
		offset: number;
		hasMore: boolean;
		loading: boolean;
	};

	function emptyPage(): PaginatedSource {
		return { items: [], offset: 0, hasMore: true, loading: false };
	}

	let thunderstoreMods: Mod[] = $state([]);
	let cfPage: PaginatedSource = $state(emptyPage());
	let serverPage: PaginatedSource = $state(emptyPage());
	let zephyrServerReachable: boolean | null = $state(null);
	let communityMods: Mod[] = $state([]);

	let displayedMods = $derived(
		showCurseForgeOnly ? mods.filter((m) => isCurseForgeMod(m) || isServerMod(m)) : mods
	);

	function shouldUseZephyrServer(): boolean {
		return zephyrServerState.current.enabled && zephyrServerReachable !== false;
	}

	function canInstallDirectly(mod: Mod): boolean {
		return isServerMod(mod) || !mod.uuid.includes(':');
	}

	function syncThunderstoreBrowseMods() {
		if (activeSource !== 'thunderstore') return;
		mods = [
			...thunderstoreMods,
			...(shouldUseZephyrServer() ? serverPage.items : cfPage.items),
			...communityMods
		];
	}

	function externalHasMore(): boolean {
		return shouldUseZephyrServer() ? serverPage.hasMore : cfPage.hasMore;
	}

	function externalLoading(): boolean {
		return shouldUseZephyrServer() ? serverPage.loading : cfPage.loading;
	}

	async function loadMoreExternalMods() {
		if (shouldUseZephyrServer()) {
			await loadMoreFromServer();
			return;
		}

		await loadMoreCF();
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
			mods.map((m) => m.uuid),
			lastClickedIndex
		);
		selectedModIds = result.selection;
		lastClickedIndex = result.lastIndex;
	}

	async function handleDepClick(author: string, name: string): Promise<boolean> {
		let found = mods.find(
			(m) =>
				m.name.toLowerCase() === name.toLowerCase() &&
				m.author?.toLowerCase() === author.toLowerCase()
		);
		if (!found) {
			found = mods.find((m) => m.name.toLowerCase() === name.toLowerCase());
		}
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
						(m) =>
							m.name.toLowerCase() === name.toLowerCase() &&
							m.author?.toLowerCase() === author.toLowerCase()
					) ?? results.find((m) => m.name.toLowerCase() === name.toLowerCase());
			} catch {}
		}
		if (found) {
			cachedSelectedMods.set(found.uuid, found);
			selectedModIds = [found.uuid];
			return true;
		}
		return false;
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
			if (activeSource !== 'thunderstore') return;
			thunderstoreMods = evt.payload;
			syncThunderstoreBrowseMods();
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

	async function refresh() {
		if (refreshing) {
			pendingRefresh = true;
			return;
		}

		const currentSlug = games.active?.slug ?? null;
		if (currentSlug !== lastGameSlug) {
			mods = [];
			thunderstoreMods = [];
			cfPage.items = [];
			cfPage.offset = 0;
			cfPage.hasMore = true;
			serverPage.items = [];
			serverPage.offset = 0;
			serverPage.hasMore = true;
			communityMods = [];
			hasRefreshed = false;
			lastGameSlug = currentSlug;
			maxCount = 30;
			showCurseForgeOnly = false;
			selectedModIds = [];
			cachedSelectedMods.clear();
			multiViewIndex = 0;
		}

		// Detect query changes to reset pagination
		const queryHash = JSON.stringify(modQuery.current);
		if (queryHash !== lastQueryHash) {
			lastQueryHash = queryHash;
			thunderstoreMods = [];
			cfPage.items = [];
			cfPage.offset = 0;
			cfPage.hasMore = true;
			serverPage.items = [];
			serverPage.offset = 0;
			serverPage.hasMore = true;
		}

		refreshing = true;

		try {
			if (activeSource === 'thunderstore') {
				thunderstoreMods = await api.thunderstore.query({ ...modQuery.current, maxCount });

				// CurseForge mods: use Zephyr Server if enabled, otherwise Tauri backend
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

				// Community mods — disabled for now
				// try {
				// 	const communityResults = await api.sources.searchSources({
				// 		searchTerm: modQuery.current.searchTerm,
				// 		categories: [],
				// 		sortBy: 'downloads',
				// 		sortOrder: 'descending',
				// 		includeNsfw: modQuery.current.includeNsfw,
				// 		includeDeprecated: modQuery.current.includeDeprecated,
				// 		maxCount: 50,
				// 		sources: ['github'],
				// 		gameSlug: games.active?.slug
				// 	});
				// 	communityMods = communityResults.flatMap((r) =>
				// 		r.mods.map((u) => unifiedToMod(u, 'zephyr'))
				// 	);
				// } catch {
				// 	communityMods = [];
				// }

				syncThunderstoreBrowseMods();
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

				thunderstoreMods = [];
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

	const CF_PAGE_SIZE = 25;

	async function loadMoreCF() {
		cfPage.loading = true;
		try {
			const sortMap: Record<string, 'downloads' | 'rating' | 'newest' | 'updated' | 'name'> = {
				lastUpdated: 'updated',
				newest: 'newest',
				rating: 'rating',
				downloads: 'downloads',
				name: 'name'
			};
			const cfResults = await api.sources.searchSources({
				searchTerm: modQuery.current.searchTerm,
				categories: [],
				sortBy: sortMap[modQuery.current.sortBy] ?? 'downloads',
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
			syncThunderstoreBrowseMods();
		} catch {}
		cfPage.loading = false;
	}

	function isCurseForgeMod(mod: Mod): boolean {
		return mod.uuid.startsWith('curseforge:');
	}

	function isServerMod(mod: Mod): boolean {
		return mod.uuid.startsWith('zephyr-server:');
	}

	const SERVER_PAGE_SIZE = 50;

	function curseForgeModToMod(cfMod: zephyrServer.CurseForgeMod): Mod {
		return {
			name: cfMod.name,
			description: cleanDescription(cfMod.summary),
			categories: cfMod.categories.map((c) => c.name),
			version: cfMod.latestFiles[0]?.displayName ?? null,
			author: cfMod.authors[0]?.name ?? null,
			rating: cfMod.thumbsUpCount > 0 ? cfMod.thumbsUpCount : null,
			downloads: cfMod.downloadCount,
			fileSize: cfMod.latestFiles[0]?.fileLength ?? 0,
			websiteUrl: cfMod.links.websiteUrl,
			donateUrl: null,
			dependencies:
				cfMod.latestFiles[0]?.dependencies
					?.filter((d) => d.relationType === 3)
					.map((d) => String(d.modId)) ?? [],
			isPinned: false,
			isDeprecated: false,
			isInstalled: undefined,
			containsNsfw: false,
			uuid: `zephyr-server:${cfMod.id}`,
			versionUuid: String(cfMod.latestFiles[0]?.id ?? cfMod.id),
			lastUpdated: cfMod.dateModified,
			versions: cfMod.latestFiles.map((f) => ({
				name: f.displayName,
				uuid: String(f.id)
			})),
			type: 'remote' as any,
			enabled: null,
			icon: cfMod.logo?.thumbnailUrl ?? null,
			configFile: null
		};
	}

	async function loadMoreFromServer() {
		if (!zephyrServerState.current.enabled) return;
		serverPage.loading = true;
		try {
			const searchTerm = modQuery.current.searchTerm || '';
			const cfGameId = zephyrServer.getCurseForgeGameId(games.active?.slug);
			if (cfGameId === null) {
				// Game not supported on CurseForge
				serverPage.hasMore = false;
				serverPage.loading = false;
				return;
			}
			const page = Math.floor(serverPage.offset / SERVER_PAGE_SIZE);
			const result = await zephyrServer.searchMods(searchTerm, cfGameId, page, SERVER_PAGE_SIZE);
			const newMods = result.data.map(curseForgeModToMod);
			serverPage.items = [...serverPage.items, ...newMods];
			serverPage.offset += SERVER_PAGE_SIZE;
			serverPage.hasMore = serverPage.offset < result.pagination.totalCount;
			syncThunderstoreBrowseMods();
		} catch (err) {
			zephyrServerReachable = false;
			console.warn('[ZephyrServer] Failed to load mods:', err);
		}
		serverPage.loading = false;
	}

	async function installFromServer(mod: Mod, versionUuid?: string) {
		if (!mod.uuid.startsWith('zephyr-server:')) return;
		const modId = parseInt(mod.uuid.replace('zephyr-server:', ''), 10);
		const fileId = parseInt(versionUuid ?? mod.versionUuid ?? mod.versions[0]?.uuid ?? '0', 10);
		if (!modId || !fileId) return;

		try {
			let gameDir: string | undefined;
			try {
				gameDir = await api.profile.launch.getGameDir();
			} catch {
				// Could not resolve game dir — let server use its default
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

	function cleanDescription(desc: string | null): string | null {
		if (!desc) return null;
		return desc
			.replace(/<br\s*\/?>/gi, ' ')
			.replace(/<[^>]*>/g, '')
			.replace(/\[.*?\]/g, '')
			.trim();
	}

	function unifiedToMod(u: UnifiedMod, source?: string): Mod {
		const sourceId = source ?? u.sourceId ?? 'thunderstore';
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
			uuid: `${sourceId}:${u.externalId}`,
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
		if (isServerMod(mod)) {
			await installFromServer(mod);
			return;
		}
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
			await refresh();
		} else if (response.type === 'confirm') {
			removeDialog = { open: true, mod, dependants: response.dependants };
		}
	}

	async function doBatchInstall() {
		if (locked) return;
		for (const uuid of selectedModIds) {
			const mod = mods.find((m) => m.uuid === uuid);
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
		selectedModIds = displayedMods.map((m) => m.uuid);
	}

	let isAllSelected = $derived(
		displayedMods.length > 0 && selectedModIds.length === displayedMods.length
	);
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
		const isMulti = selectedModIds.length > 1 && selectedModIds.includes(mod.uuid);

		if (isMulti) {
			const selectedMods = selectedModIds
				.map((id) => mods.find((m) => m.uuid === id))
				.filter(Boolean) as Mod[];
			const notInstalled = selectedMods.filter((m) => !m.isInstalled);
			const installed = selectedMods.filter((m) => m.isInstalled);

			if (notInstalled.length > 0 && !locked) {
				items.push({
					label: `${m.mods_contextMenu_install()} (${notInstalled.length})`,
					icon: 'mdi:download',
					onclick: () => doBatchInstall()
				});
			}

			if (installed.length > 0) {
				items.push({
					label: `${m.mods_contextMenu_uninstall()} (${installed.length})`,
					icon: 'mdi:delete',
					danger: true,
					disabled: locked,
					onclick: async () => {
						for (const im of installed) {
							await api.profile.forceRemoveMods([im.uuid]);
						}
						await refresh();
					}
				});
			}
		} else {
			if (!mod.isInstalled && !locked) {
				items.push({
					label: m.mods_contextMenu_install(),
					icon: 'mdi:download',
					onclick: () => installLatest(mod)
				});
			}

			if (mod.isInstalled) {
				items.push({
					label: mod.enabled === false ? m.mods_contextMenu_enable() : m.mods_contextMenu_disable(),
					icon: mod.enabled === false ? 'mdi:eye' : 'mdi:eye-off',
					disabled: locked,
					onclick: () => toggleMod(mod)
				});
			}

			if (mod.websiteUrl) {
				items.push({
					label: m.mods_contextMenu_openThunderstore(),
					icon: 'mdi:open-in-new',
					onclick: () => open(mod.websiteUrl!)
				});
			}

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
		}

		ctxMenu = { items, x: e.clientX, y: e.clientY };
	}
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
						bind:viewMode={viewMode.current}
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
						{#if (curseForgeEnabled.current && cfPage.items.length > 0) || (zephyrServerState.current.enabled && serverPage.items.length > 0)}
							<label class="z-filter-toggle">
								<Checkbox bind:checked={showCurseForgeOnly} />
								<span>CurseForge</span>
							</label>
						{/if}

						{#if games.categories.length > 0}
							<div class="z-filter-categories">
								{#each games.categories.slice(0, 20) as cat}
									<button
										class="z-category-chip"
										class:active={modQuery.current.includeCategories.includes(cat.name)}
										onclick={() => {
											const idx = modQuery.current.includeCategories.indexOf(cat.name);
											if (idx >= 0) {
												modQuery.current.includeCategories =
													modQuery.current.includeCategories.filter((c) => c !== cat.name);
											} else {
												modQuery.current.includeCategories = [
													...modQuery.current.includeCategories,
													cat.name
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
						<span class="z-browse-count">{displayedMods.length} mods</span>
					</div>
				{/if}
			</div>

			{#if locked}
				<div class="z-locked-banner">
					<Icon icon="mdi:lock" />
					<span>{i18nState.locale && m.browse_lockedBanner()}</span>
				</div>
			{/if}

			<div class="z-browse-list" class:z-grid-layout={viewMode.current === 'grid'}>
				{#if mods.length === 0 && !hasRefreshed}
					<Loader />
				{:else if cfPage.loading && showCurseForgeOnly && cfPage.items.length === 0}
					<Loader />
				{:else if displayedMods.length === 0 && hasRefreshed}
					<div class="z-browse-empty">
						<div class="z-browse-empty-icon">
							<Icon icon="mdi:package-variant-remove" />
						</div>
						<p class="z-browse-empty-title">{i18nState.locale && m.browse_noMods()}</p>
						<p class="z-browse-empty-desc">
							{showCurseForgeOnly
								? 'No CurseForge mods available for this game.'
								: i18nState.locale && m.browse_noMods_desc()}
						</p>
					</div>
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

					{#if showCurseForgeOnly}
						{#if externalHasMore()}
							<button
								class="z-load-more"
								onclick={loadMoreExternalMods}
								disabled={externalLoading()}
							>
								{externalLoading() ? 'Loading...' : i18nState.locale && m.browse_loadMore()}
							</button>
						{/if}
					{:else if displayedMods.length >= maxCount || (shouldUseZephyrServer() ? serverPage.hasMore : curseForgeEnabled.current && cfPage.hasMore)}
						<button
							class="z-load-more"
							onclick={() => {
								maxCount += 30;
								if (shouldUseZephyrServer() && serverPage.hasMore) {
									loadMoreFromServer();
								} else if (curseForgeEnabled.current && cfPage.hasMore) {
									loadMoreCF();
								} else if (zephyrServerState.current.enabled && cfPage.hasMore) {
									loadMoreCF();
								}
							}}
						>
							{i18nState.locale && m.browse_loadMore()}
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
			<div class="z-multi-nav">
				<button
					class="z-multi-nav-btn"
					disabled={multiViewIndex <= 0}
					onclick={() => multiViewIndex--}
				>
					<Icon icon="mdi:chevron-left" />
				</button>
				<span class="z-multi-nav-label">{multiViewIndex + 1} / {selectedMods.length}</span>
				<button
					class="z-multi-nav-btn"
					disabled={multiViewIndex >= selectedMods.length - 1}
					onclick={() => multiViewIndex++}
				>
					<Icon icon="mdi:chevron-right" />
				</button>
			</div>
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

<!-- Remove mod confirmation dialog -->
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
		flex: 1;
	}

	.z-browse-list.z-grid-layout {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
		grid-auto-rows: min-content;
		gap: var(--space-md);
	}

	.z-browse-list.z-grid-layout .z-browse-empty,
	.z-browse-list.z-grid-layout > :global(.z-loader) {
		grid-column: 1 / -1;
	}

	.z-browse-list > :global(.z-loader) {
		flex: 1;
		min-height: 50vh;
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
