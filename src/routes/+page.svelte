<script lang="ts">
	import * as api from '$lib/api';
	import type { Mod, ModId, SortBy, AvailableUpdate, ProfileQuery, Dependant } from '$lib/types';
	import Icon from '@iconify/svelte';
	import ScrollToTop from '$lib/components/ui/ScrollToTop.svelte';
	import profiles from '$lib/state/profile.svelte';

	import ModCard from '$lib/components/mod-list/ModCard.svelte';
	import ModDetails from '$lib/components/mod-list/ModDetails.svelte';
	import InstallModButton from '$lib/components/mod-list/InstallModButton.svelte';
	import RemoveModDialog from '$lib/components/mod-list/RemoveModDialog.svelte';
	import ToggleModDialog from '$lib/components/mod-list/ToggleModDialog.svelte';
	import ModListFilterBar from '$lib/components/mod-list/ModListFilterBar.svelte';
	import ModListEmptyState from '$lib/components/mod-list/ModListEmptyState.svelte';
	import ModListDropPlaceholder from '$lib/components/mod-list/ModListDropPlaceholder.svelte';
	import MultiViewNav from '$lib/components/mod-list/MultiViewNav.svelte';
	import UnknownModsBanner from '$lib/components/mod-list/UnknownModsBanner.svelte';
	import UnknownModsModal from '$lib/components/mod-list/UnknownModsModal.svelte';
	import BatchActionBar from '$lib/components/ui/BatchActionBar.svelte';
	import { gamepadState } from '$lib/gamepad.svelte';
	import BatchConfirmDialog from '$lib/components/dialogs/BatchConfirmDialog.svelte';
	import Header from '$lib/components/layout/Header.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import ContextMenu from '$lib/components/ui/ContextMenu.svelte';
	import Loader from '$lib/components/ui/Loader.svelte';
	import ShareProfileDialog from '$lib/components/dialogs/ShareProfileDialog.svelte';

	import { installState, viewMode } from '$lib/state/misc.svelte';
	import { profileQuery, isModPinned, pinnedMods } from '$lib/state/misc.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { pushToast } from '$lib/toast.svelte';
	import { handleMultiSelect } from '$lib/utils/multiSelect';
	import { createModDrag } from '$lib/utils/useModDrag.svelte';
	import { buildModContextMenu } from '$lib/utils/modContextMenu';
	import { modNameFromIdent } from '$lib/utils/modIdent';
	import { PAGE_SIZE_CHOICES, loadPageSize, persistPageSize } from '$lib/utils/pageSize';

	const sortOptions: SortBy[] = ['custom', 'name', 'author', 'installDate', 'diskSpace'];

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

	let mods: Mod[] = $state([]);
	let updates: AvailableUpdate[] = $state([]);
	let unknownMods: { fullName: string; uuid: string }[] = $state([]);
	let unknownModalOpen = $state(false);
	let totalModCount = $state(0);
	let filteredModCount = $state(0);

	let pageSize = $state(loadPageSize());
	let maxCount = $state(pageSize);

	function changePageSize(v: number) {
		pageSize = persistPageSize(v);
		maxCount = pageSize;
	}

	let selectedModIds: string[] = $state([]);
	let lastClickedIndex = -1;
	// Cache selected mods so they survive being filtered out of the visible list
	let cachedSelectedMods: Map<string, Mod> = new Map();

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

	function handleModClick(evt: MouseEvent, mod: Mod, index: number) {
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

	let ctxMenu: { items: ReturnType<typeof buildModContextMenu>; x: number; y: number } | null =
		$state(null);

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

	let batchRemoveDialog: { open: boolean; extraDependants: string[] } | null = $state(null);
	let batchToggleDialog: { open: boolean; extraDependants: string[] } | null = $state(null);

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
		ctxMenu = {
			items: buildModContextMenu({
				mod,
				locked,
				selectedModIds,
				handlers: { toggleMod, removeMod, doBatchToggle, doBatchRemove }
			}),
			x: e.clientX,
			y: e.clientY
		};
	}

	$effect(() => {
		if (maxCount > 0) {
			JSON.stringify(profileQuery.current);
			profiles.active;
			refresh();
		}
	});

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
			profileQuery.current.includeCategories = cats.includes(category)
				? cats.filter((c) => c !== category)
				: [...cats, category];
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

	// A mod that changes version is briefly removed and reinstalled — re-select it
	// so the details panel does not flicker back to empty.
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
				<ModListFilterBar
					bind:expanded={filtersExpanded}
					bind:viewMode={viewMode.current}
					{sortOptions}
					{pageSize}
					pageSizeChoices={PAGE_SIZE_CHOICES}
					{isAllSelected}
					visibleCount={sortedMods.length}
					ontoggleSelectAll={toggleSelectAll}
					onchangePageSize={changePageSize}
				/>

				{#if unknownMods.length > 0}
					<UnknownModsBanner
						count={unknownMods.length}
						onclick={() => (unknownModalOpen = true)}
					/>
				{/if}

				<div class="z-mods-list" class:z-grid-layout={viewMode.current === 'grid'}>
					{#if sortedMods.length === 0}
						<ModListEmptyState />
					{:else}
						{#each sortedMods as mod, i (mod.uuid)}
							{#if drag.draggedMod && drag.placeholderIndex === i}
								<ModListDropPlaceholder viewMode={viewMode.current} />
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
							<ModListDropPlaceholder viewMode={viewMode.current} />
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

<UnknownModsModal
	bind:open={unknownModalOpen}
	mods={unknownMods}
	onclose={() => (unknownModalOpen = false)}
	onchange={(next) => (unknownMods = next)}
	onafterRemove={refresh}
/>

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

	.z-mods-list.z-grid-layout > :global(.z-loader) {
		grid-column: 1 / -1;
	}

	.z-mods-list > :global(.z-loader) {
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
</style>
