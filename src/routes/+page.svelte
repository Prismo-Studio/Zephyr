<script lang="ts">
	import * as api from '$lib/api';
	import type { Mod, ModId, SortBy, AvailableUpdate, ProfileQuery, Dependant } from '$lib/types';
	import Icon from '@iconify/svelte';
	import ScrollToTop from '$lib/components/ui/ScrollToTop.svelte';
	import profiles from '$lib/state/profile.svelte';
	import games from '$lib/state/game.svelte';

	import ModCard from '$lib/components/mod-list/ModCard.svelte';
	import ModDetails from '$lib/components/mod-list/ModDetails.svelte';
	import ModListFilters from '$lib/components/mod-list/ModListFilters.svelte';
	import InstallModButton from '$lib/components/mod-list/InstallModButton.svelte';
	import RemoveModDialog from '$lib/components/mod-list/RemoveModDialog.svelte';
	import ToggleModDialog from '$lib/components/mod-list/ToggleModDialog.svelte';
	import BatchActionBar from '$lib/components/ui/BatchActionBar.svelte';
	import { gamepadState } from '$lib/gamepad.svelte';
	import BatchConfirmDialog from '$lib/components/dialogs/BatchConfirmDialog.svelte';
	import Header from '$lib/components/layout/Header.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import ContextMenu from '$lib/components/ui/ContextMenu.svelte';
	import type { ContextMenuItem } from '$lib/components/ui/ContextMenu.svelte';
	import Loader from '$lib/components/ui/Loader.svelte';
	import ShareProfileDialog from '$lib/components/dialogs/ShareProfileDialog.svelte';

	import { open } from '@tauri-apps/plugin-shell';
	import { installState, viewMode } from '$lib/state/misc.svelte';
	import { profileQuery, togglePin, isModPinned, pinnedMods } from '$lib/state/misc.svelte';
	import { isOutdated } from '$lib/util';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { pushToast } from '$lib/toast.svelte';
	import { handleMultiSelect } from '$lib/utils/multiSelect';
	import { createModDrag } from '$lib/utils/useModDrag.svelte';

	const sortOptions: SortBy[] = ['custom', 'name', 'author', 'installDate', 'diskSpace'];

	// --- Container ref (still needed by ScrollToTop) ---
	let modsContentEl: HTMLDivElement | undefined = $state();

	let isCustomSort = $derived(profileQuery.current.sortBy === 'custom');
	let canDrag = $derived(isCustomSort && !profiles.activeLocked);

	const drag = createModDrag({
		getSortedMods: () => sortedMods,
		getSortOrder: () => profileQuery.current.sortOrder,
		getViewMode: () => viewMode.current,
		canDrag: () => canDrag,
		isPinned: isModPinned,
		refresh: () => refresh()
	});

	// --- Mod list state ---
	let mods: Mod[] = $state([]);
	let updates: AvailableUpdate[] = $state([]);
	let unknownMods: { fullName: string; uuid: string }[] = $state([]);
	let unknownModalOpen = $state(false);
	let unknownBusy = $state(false);

	async function removeOneUnknown(uuid: string) {
		unknownBusy = true;
		try {
			await api.profile.forceRemoveMods([uuid]);
			unknownMods = unknownMods.filter((m) => m.uuid !== uuid);
			if (unknownMods.length === 0) {
				unknownModalOpen = false;
			}
			await refresh();
		} catch (err) {
			pushToast({
				type: 'error',
				name: m.mods_unknownMods_removeFailed(),
				message: err instanceof Error ? err.message : String(err)
			});
		} finally {
			unknownBusy = false;
		}
	}

	async function removeAllUnknown() {
		if (unknownMods.length === 0) return;
		unknownBusy = true;
		try {
			await api.profile.forceRemoveMods(unknownMods.map((m) => m.uuid));
			unknownMods = [];
			unknownModalOpen = false;
			await refresh();
		} catch (err) {
			pushToast({
				type: 'error',
				name: m.mods_unknownMods_removeFailed(),
				message: err instanceof Error ? err.message : String(err)
			});
		} finally {
			unknownBusy = false;
		}
	}
	let totalModCount = $state(0);
	let filteredModCount = $state(0);

	const PAGE_SIZE_KEY = 'zephyr-mods-page-size';
	const PAGE_SIZE_CHOICES = [30, 50, 100, 500];
	function loadPageSize(): number {
		if (typeof localStorage === 'undefined') return 30;
		const raw = localStorage.getItem(PAGE_SIZE_KEY);
		const v = raw === null ? NaN : parseInt(raw, 10);
		return PAGE_SIZE_CHOICES.includes(v) ? v : 30;
	}
	let pageSize = $state(loadPageSize());
	let maxCount = $state(pageSize);

	function changePageSize(v: number) {
		pageSize = PAGE_SIZE_CHOICES.includes(v) ? v : 30;
		try {
			localStorage.setItem(PAGE_SIZE_KEY, String(pageSize));
		} catch {
			/* ignore */
		}
		maxCount = pageSize;
	}

	// --- Selection ---
	let selectedModIds: string[] = $state([]);
	let lastClickedIndex = -1;
	// Cache selected mods so they persist even when filtered out of the list
	let cachedSelectedMods: Map<string, Mod> = new Map();

	function cacheSelectedMods() {
		for (const mod of mods) {
			if (selectedModIds.includes(mod.uuid)) {
				cachedSelectedMods.set(mod.uuid, mod);
			}
		}
	}

	function getSelectedMod(uuid: string): Mod | null {
		const fresh = mods.find((m) => m.uuid === uuid);
		if (fresh) {
			cachedSelectedMods.set(uuid, fresh);
			return fresh;
		}
		return cachedSelectedMods.get(uuid) ?? null;
	}

	let selectedMod = $derived.by(() => {
		mods; // track mods changes to recalculate
		return selectedModIds.length === 1 ? getSelectedMod(selectedModIds[0]) : null;
	});

	// Multi-select detail navigation
	let multiViewIndex = $state(0);
	let selectedMods = $derived(
		selectedModIds.length > 1
			? (selectedModIds.map((id) => getSelectedMod(id)).filter(Boolean) as Mod[])
			: []
	);
	// Clamp index when selection changes
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

	function handleModClick(evt: MouseEvent, mod: Mod, index: number) {
		// Cache the mod being clicked so it persists through filters
		cachedSelectedMods.set(mod.uuid, mod);
		const result = handleMultiSelect(
			evt,
			mod.uuid,
			index,
			selectedModIds,
			sortedMods.map((m) => m.uuid),
			lastClickedIndex
		);
		selectedModIds = result.selection;
		lastClickedIndex = result.lastIndex;
	}

	// --- Context menu ---
	let ctxMenu: { items: ContextMenuItem[]; x: number; y: number } | null = $state(null);

	// --- Dialog state ---
	let removeDialog: { open: boolean; mod: Mod; dependants: Dependant[] } | null = $state(null);
	let toggleDialog: {
		open: boolean;
		mod: Mod;
		dependants: Dependant[];
		isDisabling: boolean;
	} | null = $state(null);

	let refreshing = false;
	let pendingRefresh = false;

	async function refresh() {
		if (refreshing) {
			pendingRefresh = true;
			return;
		}
		refreshing = true;
		try {
			const result: ProfileQuery = await api.profile.query({
				...profileQuery.current,
				maxCount
			});
			mods = result.mods;
			totalModCount = result.totalModCount;
			filteredModCount = result.filteredModCount;
			updates = result.updates;
			unknownMods = result.unknownMods;
		} catch {}
		refreshing = false;

		if (pendingRefresh) {
			pendingRefresh = false;
			refresh();
		}
	}

	async function install(id: ModId) {
		await api.profile.install.mod(id);
		await refresh();
	}

	async function toggleMod(mod: Mod) {
		try {
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
		} catch (err: any) {
			pushToast({
				type: 'error',
				name:
					mod.enabled === false
						? m.toast_cannotEnable_title({ name: mod.name })
						: m.toast_cannotDisable_title({ name: mod.name }),
				message: String(err?.message ?? err)
			});
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

	async function updateAllMods() {
		const uuids = updates.filter((u) => !u.ignore).map((u) => u.packageUuid);
		if (uuids.length > 0) {
			await api.profile.update.mods(uuids, true);
			await refresh();
		}
	}

	// --- Batch actions ---
	let batchRemoveDialog: { open: boolean; extraDependants: string[] } | null = $state(null);
	let batchToggleDialog: { open: boolean; extraDependants: string[] } | null = $state(null);

	// Extracts the mod name from a VersionIdent string ("Owner-Name-1.0.0" → "Name")
	function modNameFromIdent(ident: string): string {
		const lastDash = ident.lastIndexOf('-');
		if (lastDash === -1) return ident;
		const secondLastDash = ident.lastIndexOf('-', lastDash - 1);
		if (secondLastDash === -1) return ident;
		return ident.substring(secondLastDash + 1, lastDash);
	}

	async function collectExtraDeps(uuidFilter?: (uuid: string) => boolean) {
		const ids = uuidFilter ? selectedModIds.filter(uuidFilter) : selectedModIds;
		const selectedSet = new Set(selectedModIds);
		const extraDeps = new Set<string>();

		await Promise.all(
			ids.map(async (uuid) => {
				const deps = await api.profile.getDependants(uuid);
				for (const dep of deps) {
					const depName = modNameFromIdent(dep);
					const depMod = mods.find((m) => m.name === depName);
					if (depMod && !selectedSet.has(depMod.uuid)) {
						extraDeps.add(dep);
					} else if (!depMod) {
						extraDeps.add(dep);
					}
				}
			})
		);

		return [...extraDeps];
	}

	async function doBatchRemove() {
		if (locked) return;
		const extraDeps = await collectExtraDeps();
		if (extraDeps.length > 0) {
			batchRemoveDialog = { open: true, extraDependants: extraDeps };
		} else {
			await api.profile.forceRemoveMods(selectedModIds);
			selectedModIds = [];
			await refresh();
		}
	}

	async function doBatchToggle() {
		if (locked) return;
		const extraDeps = await collectExtraDeps((uuid) => {
			const mod = mods.find((m) => m.uuid === uuid);
			return mod ? mod.enabled !== false : false;
		});
		if (extraDeps.length > 0) {
			batchToggleDialog = { open: true, extraDependants: extraDeps };
		} else {
			await api.profile.forceToggleMods(selectedModIds);
			await refresh();
		}
	}

	function openModContextMenu(e: MouseEvent, mod: Mod) {
		const locked = profiles.activeLocked;
		const items: ContextMenuItem[] = [];
		const isMulti = selectedModIds.length > 1 && selectedModIds.includes(mod.uuid);

		if (isMulti) {
			const selectedMods = selectedModIds
				.map((id) => mods.find((m) => m.uuid === id))
				.filter(Boolean) as Mod[];

			items.push({
				label: `${m.mods_contextMenu_enable()} / ${m.mods_contextMenu_disable()} (${selectedMods.length})`,
				icon: 'mdi:toggle-switch',
				disabled: locked,
				onclick: () => doBatchToggle()
			});

			items.push({ label: '', separator: true });
			items.push({
				label: `${m.mods_contextMenu_uninstall()} (${selectedMods.length})`,
				icon: 'mdi:delete',
				danger: true,
				disabled: locked,
				onclick: () => doBatchRemove()
			});
		} else {
			if (mod.isInstalled) {
				items.push({
					label: mod.enabled === false ? m.mods_contextMenu_enable() : m.mods_contextMenu_disable(),
					icon: mod.enabled === false ? 'mdi:eye' : 'mdi:eye-off',
					disabled: locked,
					onclick: () => toggleMod(mod)
				});
			}

			if (mod.isInstalled) {
				const pinned = isModPinned(mod.uuid);
				items.push({
					label: pinned ? m.mods_contextMenu_unpin() : m.mods_contextMenu_pin(),
					icon: pinned ? 'mdi:pin-off' : 'mdi:pin',
					onclick: () => togglePin(mod.uuid)
				});
			}

			if (mod.websiteUrl) {
				items.push({
					label: m.mods_contextMenu_openThunderstore(),
					icon: 'mdi:open-in-new',
					onclick: () => open(mod.websiteUrl!)
				});
			}

			if (mod.donateUrl) {
				items.push({
					label: m.mods_contextMenu_donate(),
					icon: 'mdi:heart',
					onclick: () => open(mod.donateUrl!)
				});
			}

			if (mod.isInstalled) {
				items.push({
					label: m.mods_contextMenu_openFolder(),
					icon: 'mdi:folder-open',
					onclick: () => api.profile.openModDir(mod.uuid)
				});
			}

			if (mod.isInstalled) {
				items.push({
					label: m.mods_contextMenu_editConfig(),
					icon: 'mdi:file-cog',
					onclick: () => {
						window.location.href = `/config?mod=${encodeURIComponent(mod.name)}`;
					}
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

	$effect(() => {
		if (maxCount > 0) {
			JSON.stringify(profileQuery.current);
			profiles.active;
			refresh();
		}
	});

	// Clear selection when game/profile changes
	let prevProfileId: number | null = null;
	$effect(() => {
		const id = profiles.activeId;
		if (prevProfileId !== null && id !== prevProfileId) {
			selectedModIds = [];
			cachedSelectedMods.clear();
			multiViewIndex = 0;
		}
		prevProfileId = id;
	});

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

	let locked = $derived(profiles.activeLocked);
	let filtersExpanded = $state(false);

	function toggleCategoryFilter(category: string, multi = false) {
		const cats = profileQuery.current.includeCategories;
		if (multi) {
			if (cats.includes(category)) {
				profileQuery.current.includeCategories = cats.filter((c) => c !== category);
			} else {
				profileQuery.current.includeCategories = [...cats, category];
			}
		} else {
			if (cats.includes(category) && cats.length === 1) {
				profileQuery.current.includeCategories = [];
			} else {
				profileQuery.current.includeCategories = [category];
			}
		}
		filtersExpanded = true;
	}

	let sortedMods = $derived.by(() => {
		const pinned = pinnedMods.current;
		if (pinned.length === 0) return mods;
		return [...mods].sort((a, b) => {
			const aPinned = pinned.includes(a.uuid);
			const bPinned = pinned.includes(b.uuid);
			if (aPinned && !bPinned) return -1;
			if (!aPinned && bPinned) return 1;
			return 0;
		});
	});

	function selectAll() {
		selectedModIds = sortedMods.map((m) => m.uuid);
	}

	let isAllSelected = $derived(
		sortedMods.length > 0 && selectedModIds.length === sortedMods.length
	);

	function toggleSelectAll() {
		selectedModIds = isAllSelected ? [] : sortedMods.map((m) => m.uuid);
	}

	// Re-select mod after version change (mod gets removed then reinstalled)
	$effect(() => {
		const handler = (e: Event) => {
			const uuid = (e as CustomEvent).detail;
			if (uuid) {
				setTimeout(() => {
					selectedModIds = [uuid];
				}, 500);
			}
		};
		window.addEventListener('zephyr-reselect-mod', handler);
		return () => window.removeEventListener('zephyr-reselect-mod', handler);
	});

	let shareDialog: { open: boolean; mode: 'export' | 'import' } = $state({
		open: false,
		mode: 'export'
	});

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

	function handleKeydown(e: KeyboardEvent) {
		if (locked) return;
		// Ignore if user is typing in an input
		const tag = (e.target as HTMLElement).tagName;
		if (tag === 'INPUT' || tag === 'TEXTAREA') return;

		if (e.key === 'Delete') {
			if (selectedModIds.length === 1) {
				const mod = mods.find((m) => m.uuid === selectedModIds[0]);
				if (mod) removeMod(mod);
			} else if (selectedModIds.length > 1) {
				doBatchRemove();
			}
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if !profiles.ready}
	<Loader />
{:else}
	<div class="z-mods-page">
		<div class="z-mods-main">
			<Header
				title={i18nState.locale && m.navBar_label_mods()}
				subtitle={i18nState.locale &&
					m.mods_installed({
						count: (filteredModCount || totalModCount).toString()
					})}
			>
				{#snippet actions()}
					<Button
						variant="ghost"
						size="sm"
						onclick={() => (shareDialog = { open: true, mode: 'import' })}
					>
						{#snippet icon()}<Icon icon="mdi:import" />{/snippet}
						{i18nState.locale && m.share_importButton()}
					</Button>
					<Button
						variant="accent"
						size="sm"
						onclick={() => (shareDialog = { open: true, mode: 'export' })}
					>
						{#snippet icon()}<Icon icon="mdi:share-variant" />{/snippet}
						{i18nState.locale && m.share_exportButton()}
					</Button>
					{#if updates.length > 0}
						<Button variant="accent" size="sm" onclick={updateAllMods}>
							{#snippet icon()}<Icon icon="mdi:update" />{/snippet}
							{m.mods_update({ count: updates.length.toString() })}
						</Button>
					{/if}
					{#if locked}
						<Badge variant="warning">
							<Icon icon="mdi:lock" class="text-xs" />
							{m.mods_locked()}
						</Badge>
					{/if}
				{/snippet}
			</Header>

			<div class="z-mods-content" bind:this={modsContentEl}>
				<div class="z-mods-filters">
					<div class="z-mods-filters-row">
						{#if !filtersExpanded}
							<label class="z-master-checkbox-wrapper">
								<Checkbox checked={isAllSelected} onchange={toggleSelectAll} />
								<span class="z-master-checkbox-label"
									>{i18nState.locale && m.batch_selectAll()}</span
								>
							</label>
						{/if}
						<div class="flex-1"></div>
						<ModListFilters
							queryArgs={profileQuery.current}
							{sortOptions}
							externalPanel
							bind:expanded={filtersExpanded}
							bind:viewMode={viewMode.current}
							{pageSize}
							pageSizeChoices={PAGE_SIZE_CHOICES}
							onChangePageSize={changePageSize}
						/>
					</div>
					{#if filtersExpanded}
						<div class="z-mods-filter-panel">
							<label class="z-filter-toggle">
								<Checkbox bind:checked={profileQuery.current.includeNsfw} />
								<span>{i18nState.locale && m.modListFilters_options_NSFW()}</span>
							</label>
							<label class="z-filter-toggle">
								<Checkbox bind:checked={profileQuery.current.includeDeprecated} />
								<span>{i18nState.locale && m.modListFilters_options_deprecated()}</span>
							</label>

							{#if games.categories.length > 0}
								<div class="z-filter-categories">
									{#each games.categories.slice(0, 20) as cat}
										<button
											class="z-category-chip"
											class:active={profileQuery.current.includeCategories.includes(cat.name)}
											onclick={() => {
												const idx = profileQuery.current.includeCategories.indexOf(cat.name);
												if (idx >= 0) {
													profileQuery.current.includeCategories =
														profileQuery.current.includeCategories.filter((c) => c !== cat.name);
												} else {
													profileQuery.current.includeCategories = [
														...profileQuery.current.includeCategories,
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

						<div class="z-mods-select-row">
							<label class="z-master-checkbox-wrapper">
								<Checkbox checked={isAllSelected} onchange={toggleSelectAll} />
								<span class="z-master-checkbox-label"
									>{i18nState.locale && m.batch_selectAll()}</span
								>
							</label>
							<span class="z-mods-count">{sortedMods.length} mods</span>
						</div>
					{/if}
				</div>

				{#if unknownMods.length > 0}
					<button class="z-unknown-banner" onclick={() => (unknownModalOpen = true)}>
						<Icon icon="mdi:help-circle" />
						<span>{m.mods_unknownMods({ count: unknownMods.length.toString() })}</span>
						<Icon icon="mdi:chevron-right" class="z-unknown-chev" />
					</button>
				{/if}

				<div class="z-mods-list" class:z-grid-layout={viewMode.current === 'grid'}>
					{#if sortedMods.length === 0}
						<div class="z-mods-empty">
							<div class="z-empty-icon">
								<Icon icon="mdi:package-variant" />
							</div>
							<p class="z-empty-title">{i18nState.locale && m.mods_noMods()}</p>
							<p class="z-empty-desc">{i18nState.locale && m.mods_noMods_desc()}</p>
							<a href="/browse" class="z-empty-action">
								<Icon icon="mdi:store-search" />
								{i18nState.locale && m.mods_browseMods()}
							</a>
						</div>
					{:else}
						{#each sortedMods as mod, i (mod.uuid)}
							{#if drag.draggedMod && drag.placeholderIndex === i}
								{#if viewMode.current === 'grid'}
									<div class="z-drop-placeholder-grid"></div>
								{:else}
									<div class="z-drop-placeholder">
										<div class="z-drop-line"></div>
										<Icon icon="mdi:plus-circle" class="z-drop-icon" />
										<div class="z-drop-line"></div>
									</div>
								{/if}
							{/if}

							<div data-mod-index={i} class:z-dragging-card={drag.draggedMod?.uuid === mod.uuid}>
								<ModCard
									{mod}
									isSelected={selectedModIds.includes(mod.uuid)}
									{locked}
									showInstallBtn={false}
									showDragHandle={canDrag}
									isDragging={drag.draggedMod?.uuid === mod.uuid}
									viewMode={viewMode.current}
									onclick={(evt: MouseEvent) => {
										if (!drag.draggedMod) handleModClick(evt, mod, i);
									}}
									ontoggle={toggleMod}
									oncontextmenu={openModContextMenu}
									onpointerdownHandle={drag.handleDragHandleDown}
									oncategoryclick={toggleCategoryFilter}
									activeCategories={profileQuery.current.includeCategories}
								/>
							</div>
						{/each}

						{#if drag.draggedMod && drag.placeholderIndex === sortedMods.length}
							{#if viewMode.current === 'grid'}
								<div class="z-drop-placeholder-grid"></div>
							{:else}
								<div class="z-drop-placeholder">
									<div class="z-drop-line"></div>
									<Icon icon="mdi:plus-circle" class="z-drop-icon" />
									<div class="z-drop-line"></div>
								</div>
							{/if}
						{/if}

						{#if mods.length < (filteredModCount || totalModCount) && pageSize !== -1}
							<button
								class="z-load-more"
								onclick={() => (maxCount += pageSize === -1 ? 10_000 : pageSize)}
							>
								{m.mods_loadMore({
									count: ((filteredModCount || totalModCount) - mods.length).toString()
								})}
							</button>
						{/if}
					{/if}
				</div>
			</div>
			<ScrollToTop target={modsContentEl} lifted={selectedModIds.length >= 2} />
		</div>

		{#if selectedMod}
			<ModDetails
				mod={selectedMod}
				{locked}
				onclose={() => (selectedModIds = [])}
				ontoggle={() => toggleMod(selectedMod!)}
				onremove={() => removeMod(selectedMod!)}
				oncategoryclick={toggleCategoryFilter}
				ondepclick={handleDepClick}
				activeCategories={profileQuery.current.includeCategories}
			>
				<InstallModButton mod={selectedMod} {install} {locked} />
			</ModDetails>
		{:else if multiViewMod}
			<ModDetails
				mod={multiViewMod}
				{locked}
				onclose={() => (selectedModIds = [])}
				ontoggle={() => toggleMod(multiViewMod!)}
				onremove={() => removeMod(multiViewMod!)}
				oncategoryclick={toggleCategoryFilter}
				ondepclick={handleDepClick}
				activeCategories={profileQuery.current.includeCategories}
			>
				<InstallModButton mod={multiViewMod} {install} {locked} />
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
			total={sortedMods.length}
			onclear={() => (selectedModIds = [])}
			onselectAll={selectAll}
			legendActive={!!(gamepadState.enabled && gamepadState.connected)}
		>
			{#snippet actions()}
				<Button variant="accent" size="sm" onclick={doBatchToggle} disabled={locked}>
					{#snippet icon()}<Icon icon="mdi:toggle-switch" />{/snippet}
					{i18nState.locale && m.batch_toggleSelected()}
				</Button>
				<Button variant="danger" size="sm" onclick={doBatchRemove} disabled={locked}>
					{#snippet icon()}<Icon icon="mdi:delete" />{/snippet}
					{i18nState.locale && m.batch_uninstallSelected()}
				</Button>
			{/snippet}
		</BatchActionBar>
	</div>

	{#if ctxMenu}
		<ContextMenu
			items={ctxMenu.items}
			x={ctxMenu.x}
			y={ctxMenu.y}
			onclose={() => (ctxMenu = null)}
		/>
	{/if}

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

	{#if batchToggleDialog}
		<BatchConfirmDialog
			bind:open={batchToggleDialog.open}
			title={i18nState.locale
				? m.batchToggleDialog_title({ count: selectedModIds.length.toString() })
				: ''}
			description={i18nState.locale ? m.batchToggleDialog_description() : ''}
			hint={i18nState.locale ? m.batchToggleDialog_hint() : ''}
			extraDependants={batchToggleDialog.extraDependants}
			confirmLabel={i18nState.locale ? m.batchToggleDialog_confirm() : ''}
			cancelLabel={i18nState.locale ? m.batchToggleDialog_cancel() : ''}
			confirmVariant="accent"
			confirmIcon="mdi:toggle-switch"
			onconfirm={async () => {
				await api.profile.forceToggleMods(selectedModIds);
				batchToggleDialog = null;
				await refresh();
			}}
			oncancel={() => (batchToggleDialog = null)}
		/>
	{/if}

	{#if batchRemoveDialog}
		<BatchConfirmDialog
			bind:open={batchRemoveDialog.open}
			title={i18nState.locale
				? m.batchRemoveDialog_title({ count: selectedModIds.length.toString() })
				: ''}
			description={i18nState.locale ? m.batchRemoveDialog_description() : ''}
			hint={i18nState.locale ? m.batchRemoveDialog_hint() : ''}
			extraDependants={batchRemoveDialog.extraDependants}
			confirmLabel={i18nState.locale
				? m.batchRemoveDialog_confirm({ count: selectedModIds.length.toString() })
				: ''}
			cancelLabel={i18nState.locale ? m.batchRemoveDialog_cancel() : ''}
			confirmVariant="danger"
			confirmIcon="mdi:delete-sweep"
			onconfirm={async () => {
				await api.profile.forceRemoveMods(selectedModIds);
				selectedModIds = [];
				batchRemoveDialog = null;
				await refresh();
			}}
			oncancel={() => (batchRemoveDialog = null)}
		/>
	{/if}

	<ShareProfileDialog
		bind:mode={shareDialog.mode}
		bind:open={shareDialog.open}
		onclose={() => (shareDialog.open = false)}
	/>
{/if}

<Modal
	bind:open={unknownModalOpen}
	title={m.mods_unknownMods({ count: unknownMods.length.toString() })}
	onclose={() => (unknownModalOpen = false)}
>
	<div class="z-unknown-modal">
		<p class="z-unknown-desc">
			{i18nState.locale && m.mods_unknownMods_desc()}
		</p>
		<ul class="z-unknown-list">
			{#each unknownMods as unknown (unknown.uuid)}
				<li class="z-unknown-item">
					<Icon icon="mdi:help-circle" />
					<code>{unknown.fullName}</code>
					<button
						class="z-unknown-remove"
						disabled={unknownBusy}
						onclick={() => removeOneUnknown(unknown.uuid)}
						title={i18nState.locale && m.mods_unknownMods_removeTooltip()}
					>
						<Icon icon="mdi:delete" />
					</button>
				</li>
			{/each}
		</ul>
	</div>
	{#snippet actions()}
		<Button variant="ghost" onclick={() => (unknownModalOpen = false)}>
			{i18nState.locale && m.mods_unknownMods_close()}
		</Button>
		<Button variant="danger" disabled={unknownBusy} onclick={removeAllUnknown}>
			{#snippet icon()}<Icon icon="mdi:delete-sweep" />{/snippet}
			{i18nState.locale && m.mods_unknownMods_removeAll()}
		</Button>
	{/snippet}
</Modal>

<style>
	.z-mods-page {
		display: flex;
		height: 100%;
		overflow: hidden;
	}

	.z-mods-main {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		min-width: 0;
		position: relative;
	}

	.z-mods-content {
		flex: 1;
		overflow-y: auto;
		padding: 0 var(--space-xl);
		padding-bottom: var(--space-xl);
		display: flex;
		flex-direction: column;
		min-height: 0;
	}

	.z-mods-filters {
		position: sticky;
		top: 0;
		z-index: 10;
		padding-top: var(--space-sm);
		padding-bottom: var(--space-xs);
		background: var(--bg-base);
		border-bottom: 1px solid var(--border-subtle);
		margin-bottom: var(--space-sm);
	}

	.z-mods-filters-row {
		display: flex;
		align-items: center;
		gap: var(--space-md);
	}

	.z-mods-filter-panel {
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

	.z-mods-select-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding-top: var(--space-xs);
	}

	.z-mods-count {
		font-size: 11px;
		color: var(--text-muted);
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

	.z-mods-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
		user-select: none;
		flex: 1;
	}

	.z-mods-list.z-grid-layout {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
		grid-auto-rows: min-content;
		gap: var(--space-md);
	}

	.z-mods-list.z-grid-layout .z-mods-empty,
	.z-mods-list.z-grid-layout .z-drop-placeholder,
	.z-mods-list.z-grid-layout > :global(.z-loader) {
		grid-column: 1 / -1;
	}

	.z-mods-list > :global(.z-loader) {
		flex: 1;
		min-height: 50vh;
	}

	.z-mods-list.z-grid-layout .z-drop-placeholder-grid {
		/* Takes up one grid cell, not spanning */
	}

	.z-unknown-banner {
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
		width: 100%;
		text-align: left;
		cursor: pointer;
		font-family: inherit;
		transition: all var(--transition-fast);
	}

	.z-unknown-banner:hover {
		background: rgba(255, 179, 71, 0.15);
		border-color: rgba(255, 179, 71, 0.4);
	}

	.z-unknown-banner span {
		flex: 1;
	}

	.z-unknown-banner :global(.z-unknown-chev) {
		font-size: 16px;
		opacity: 0.6;
	}

	.z-unknown-modal {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.z-unknown-desc {
		margin: 0;
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.5;
	}

	.z-unknown-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
		max-height: 360px;
		overflow-y: auto;
	}

	.z-unknown-item {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: 8px 12px;
		border-radius: var(--radius-sm);
		background: var(--bg-base);
		border: 1px solid var(--border-subtle);
	}

	.z-unknown-item > :global(svg:first-child) {
		color: var(--warning);
		flex-shrink: 0;
	}

	.z-unknown-item code {
		flex: 1;
		min-width: 0;
		font-size: 12px;
		color: var(--text-primary);
		background: transparent;
		padding: 0;
		overflow-wrap: anywhere;
		word-break: break-all;
	}

	.z-unknown-remove {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
		flex-shrink: 0;
	}

	.z-unknown-remove:hover {
		color: var(--error);
		border-color: var(--error);
	}

	.z-unknown-remove:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.z-mods-empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: var(--space-3xl) var(--space-xl);
		gap: var(--space-sm);
	}

	.z-empty-icon {
		width: 64px;
		height: 64px;
		border-radius: var(--radius-xl);
		background: var(--bg-elevated);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 28px;
		color: var(--text-muted);
		margin-bottom: var(--space-sm);
	}

	.z-empty-title {
		font-family: var(--font-display);
		font-size: 16px;
		font-weight: 700;
		color: var(--text-primary);
	}

	.z-empty-desc {
		font-size: 13px;
		color: var(--text-muted);
	}

	.z-empty-action {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-top: var(--space-md);
		padding: var(--space-sm) var(--space-lg);
		border-radius: var(--radius-md);
		background: var(--bg-active);
		color: var(--text-accent);
		font-size: 13px;
		font-weight: 600;
		text-decoration: none;
		transition: all var(--transition-fast);
	}

	.z-empty-action:hover {
		background: var(--bg-active);
		box-shadow: var(--shadow-glow);
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

	.z-load-more:hover {
		border-color: var(--border-accent);
		color: var(--text-accent);
		background: var(--bg-hover);
	}

	.z-dragging-card {
		opacity: 0.25;
		pointer-events: none;
	}

	.z-grid-layout .z-dragging-card {
		display: none;
	}

	.z-drop-placeholder-grid {
		width: 100%;
		border: 2px dashed var(--accent-400);
		border-radius: var(--radius-lg);
		background: var(--bg-active);
		overflow: hidden;
		position: relative;
		display: flex;
		align-items: center;
		justify-content: center;
		aspect-ratio: 0.7;
		min-height: 220px;
	}

	.z-drop-placeholder-grid::after {
		content: '';
		width: 32px;
		height: 32px;
		border-radius: 50%;
		border: 2px solid var(--accent-400);
		opacity: 0.3;
	}

	.z-drop-placeholder {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm) 0;
		animation: placeholderIn 150ms ease;
	}

	.z-drop-line {
		flex: 1;
		height: 2px;
		border-top: 2px dashed var(--accent-400);
		opacity: 0.6;
	}

	:global(.z-drop-icon) {
		font-size: 18px;
		color: var(--accent-400);
		flex-shrink: 0;
	}

	@keyframes placeholderIn {
		from {
			opacity: 0;
			padding: 0;
		}
		to {
			opacity: 1;
			padding: var(--space-sm) 0;
		}
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
