<script lang="ts">
	import * as api from '$lib/api';
	import type { Mod, ModId, SortBy, AvailableUpdate, ProfileQuery, Dependant } from '$lib/types';
	import Icon from '@iconify/svelte';
	import profiles from '$lib/state/profile.svelte';

	import ModCard from '$lib/components/mod-list/ModCard.svelte';
	import ModDetails from '$lib/components/mod-list/ModDetails.svelte';
	import ModListFilters from '$lib/components/mod-list/ModListFilters.svelte';
	import InstallModButton from '$lib/components/mod-list/InstallModButton.svelte';
	import RemoveModDialog from '$lib/components/mod-list/RemoveModDialog.svelte';
	import ToggleModDialog from '$lib/components/mod-list/ToggleModDialog.svelte';
	import BatchActionBar from '$lib/components/ui/BatchActionBar.svelte';
	import Header from '$lib/components/layout/Header.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import ContextMenu from '$lib/components/ui/ContextMenu.svelte';
	import type { ContextMenuItem } from '$lib/components/ui/ContextMenu.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Loader from '$lib/components/ui/Loader.svelte';
	import ShareProfileDialog from '$lib/components/dialogs/ShareProfileDialog.svelte';

	import { onMount } from 'svelte';
	import { open } from '@tauri-apps/plugin-shell';
	import { profileQuery, togglePin, isModPinned, pinnedMods } from '$lib/state/misc.svelte';
	import { communityUrl, isOutdated } from '$lib/util';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { pushToast } from '$lib/toast';

	const sortOptions: SortBy[] = ['custom', 'name', 'author', 'installDate', 'diskSpace'];

	// Drag & drop state
	let draggedMod: Mod | null = $state(null);
	let dragFromIndex = -1;
	let insertPos = -1; // target position in post-removal array
	let placeholderIndex: number = $state(-1); // index in mods[] for template display
	let ghostEl: HTMLDivElement | null = null;
	let dragOffsetX = 0;
	let dragOffsetY = 0;

	let isCustomSort = $derived(profileQuery.current.sortBy === 'custom');
	let canDrag = $derived(isCustomSort && !profiles.activeLocked);

	function handleDragHandleDown(e: PointerEvent, mod: Mod) {
		if (!canDrag || isModPinned(mod.uuid)) return;
		e.preventDefault();
		e.stopPropagation();

		dragFromIndex = sortedMods.findIndex((m) => m.uuid === mod.uuid);
		draggedMod = mod;
		insertPos = -1;
		placeholderIndex = -1;

		const card = (e.target as HTMLElement).closest('.z-mod-card') as HTMLElement;
		if (!card) return;

		const rect = card.getBoundingClientRect();
		dragOffsetX = e.clientX - rect.left;
		dragOffsetY = e.clientY - rect.top;

		ghostEl = card.cloneNode(true) as HTMLDivElement;
		ghostEl.style.cssText = `
			position: fixed;
			left: ${rect.left}px;
			top: ${rect.top}px;
			width: ${rect.width}px;
			pointer-events: none;
			z-index: 9999;
			opacity: 0.9;
			border: 1px solid var(--accent-400);
			box-shadow: 0 8px 32px rgba(26, 255, 250, 0.2), 0 0 0 1px rgba(26, 255, 250, 0.3);
			background: var(--bg-elevated);
			border-radius: var(--radius-lg);
			transform: scale(1.02);
			cursor: grabbing;
		`;
		document.body.appendChild(ghostEl);

		window.addEventListener('pointermove', handlePointerMove);
		window.addEventListener('pointerup', handlePointerUp);
	}

	function handlePointerMove(e: PointerEvent) {
		if (!draggedMod || !ghostEl) return;

		ghostEl.style.left = e.clientX - dragOffsetX + 'px';
		ghostEl.style.top = e.clientY - dragOffsetY + 'px';

		// Count how many non-dragged cards have their center ABOVE the cursor
		const wrappers = document.querySelectorAll('[data-mod-index]');
		let aboveCount = 0;

		wrappers.forEach((el) => {
			const idx = parseInt(el.getAttribute('data-mod-index')!);
			if (idx === dragFromIndex) return;

			const rect = el.getBoundingClientRect();
			const midY = rect.top + rect.height / 2;
			if (midY < e.clientY) aboveCount++;
		});

		insertPos = aboveCount;

		// Convert insertPos (post-removal position) to template index for placeholder
		if (insertPos <= dragFromIndex) {
			placeholderIndex = insertPos;
		} else {
			placeholderIndex = insertPos + 1;
		}

		// Don't show placeholder at current position (no-op)
		if (insertPos === dragFromIndex) {
			placeholderIndex = -1;
		}
	}

	async function handlePointerUp(_e: PointerEvent) {
		window.removeEventListener('pointermove', handlePointerMove);
		window.removeEventListener('pointerup', handlePointerUp);

		if (ghostEl) {
			ghostEl.remove();
			ghostEl = null;
		}

		if (draggedMod && insertPos >= 0 && insertPos !== dragFromIndex) {
			// Sort order is descending by default, so visual order is reversed from array order
			const isDescending = profileQuery.current.sortOrder === 'descending';
			const delta = isDescending ? -(insertPos - dragFromIndex) : insertPos - dragFromIndex;
			if (delta !== 0) {
				try {
					await api.profile.reorderMod(draggedMod.uuid, delta);
					await refresh();
				} catch (err) {
					console.error('Failed to reorder mod:', err);
				}
			}
		}

		draggedMod = null;
		insertPos = -1;
		placeholderIndex = -1;
		dragFromIndex = -1;
	}

	let mods: Mod[] = $state([]);
	let updates: AvailableUpdate[] = $state([]);
	let unknownMods: { fullName: string; uuid: string }[] = $state([]);
	let totalModCount = $state(0);
	let maxCount = $state(40);

	let selectedModIds: string[] = $state([]);
	let lastClickedIndex = -1;
	let selectedMod = $derived(
		selectedModIds.length === 1 ? (mods.find((m) => m.uuid === selectedModIds[0]) ?? null) : null
	);

	function handleModClick(evt: MouseEvent, mod: Mod, index: number) {
		if (evt.ctrlKey || evt.metaKey) {
			if (selectedModIds.includes(mod.uuid)) {
				selectedModIds = selectedModIds.filter((id) => id !== mod.uuid);
			} else {
				selectedModIds = [...selectedModIds, mod.uuid];
			}
			lastClickedIndex = index;
		} else if (evt.shiftKey && lastClickedIndex !== -1) {
			// Find indices in sortedMods array
			const start = Math.min(lastClickedIndex, index);
			const end = Math.max(lastClickedIndex, index);
			const newSelection = new Set(selectedModIds);
			for (let j = start; j <= end; j++) {
				if (sortedMods[j]) newSelection.add(sortedMods[j].uuid);
			}
			selectedModIds = Array.from(newSelection);
		} else {
			if (selectedModIds.length === 1 && selectedModIds[0] === mod.uuid) {
				selectedModIds = [];
			} else {
				selectedModIds = [mod.uuid];
			}
			lastClickedIndex = index;
		}
	}

	// Context menu state
	let ctxMenu: { items: ContextMenuItem[]; x: number; y: number } | null = $state(null);

	// Remove-mod dialog state
	let removeDialog: { open: boolean; mod: Mod; dependants: Dependant[] } | null = $state(null);

	// Toggle-mod dialog state
	let toggleDialog: {
		open: boolean;
		mod: Mod;
		dependants: Dependant[];
		isDisabling: boolean;
	} | null = $state(null);

	let refreshing = false;

	async function refresh() {
		if (refreshing) return;
		refreshing = true;

		try {
			const result: ProfileQuery = await api.profile.query({
				...profileQuery.current,
				maxCount
			});

			mods = result.mods;
			totalModCount = result.totalModCount;
			updates = result.updates;
			unknownMods = result.unknownMods;
		} catch {}

		refreshing = false;
	}

	async function installLatest(mod: Mod) {
		if (mod.versions.length === 0) return;
		await api.profile.install.mod({
			packageUuid: mod.uuid,
			versionUuid: mod.versions[0].uuid
		});
		await refresh();
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

	async function doForceToggleOne(mod: Mod) {
		await api.profile.forceToggleMods([mod.uuid]);
		toggleDialog = null;
		await refresh();
	}

	async function doForceToggleAll(mod: Mod, dependants: Dependant[]) {
		const uuids = [mod.uuid, ...dependants.map((d) => d.uuid)];
		await api.profile.forceToggleMods(uuids);
		toggleDialog = null;
		await refresh();
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

	async function doForceRemoveOne(mod: Mod) {
		await api.profile.forceRemoveMods([mod.uuid]);
		selectedModIds = selectedModIds.filter((id) => id !== mod.uuid);
		removeDialog = null;
		await refresh();
	}

	async function doForceRemoveAll(mod: Mod, dependants: Dependant[]) {
		const uuids = [mod.uuid, ...dependants.map((d) => d.uuid)];
		await api.profile.forceRemoveMods(uuids);
		selectedModIds = selectedModIds.filter((id) => !uuids.includes(id));
		removeDialog = null;
		await refresh();
	}

	async function updateAllMods() {
		const uuids = updates.filter((u) => !u.ignore).map((u) => u.packageUuid);
		if (uuids.length > 0) {
			await api.profile.update.mods(uuids, true);
			await refresh();
		}
	}

	let batchRemoveDialog: {
		open: boolean;
		extraDependants: string[];
	} | null = $state(null);

	async function doBatchRemove() {
		if (locked) return;

		// Read-only check: collect dependants not already in selection
		const selectedSet = new Set(selectedModIds);
		const extraDeps = new Set<string>();

		await Promise.all(
			selectedModIds.map(async (uuid) => {
				const deps = await api.profile.getDependants(uuid);
				for (const dep of deps) {
					// getDependants returns VersionIdent strings;
					// match them to mods by checking if their uuid is in selection
					const depMod = mods.find((m) => dep.includes(m.name));
					if (depMod && !selectedSet.has(depMod.uuid)) {
						extraDeps.add(dep);
					} else if (!depMod) {
						extraDeps.add(dep);
					}
				}
			})
		);

		if (extraDeps.size > 0) {
			batchRemoveDialog = {
				open: true,
				extraDependants: [...extraDeps]
			};
		} else {
			await api.profile.forceRemoveMods(selectedModIds);
			selectedModIds = [];
			await refresh();
		}
	}

	async function doBatchForceRemoveAll() {
		if (!batchRemoveDialog) return;
		// Force remove all selected — backend handles cascade
		await api.profile.forceRemoveMods(selectedModIds);
		selectedModIds = [];
		batchRemoveDialog = null;
		await refresh();
	}

	async function doBatchForceRemoveCancel() {
		batchRemoveDialog = null;
	}

	let batchToggleDialog: {
		open: boolean;
		extraDependants: string[];
	} | null = $state(null);

	async function doBatchToggle() {
		if (locked) return;

		const selectedSet = new Set(selectedModIds);
		const extraDeps = new Set<string>();

		// Only check dependants for mods being disabled (enabled mods)
		const disablingIds = selectedModIds.filter((id) => {
			const mod = mods.find((m) => m.uuid === id);
			return mod && mod.enabled !== false;
		});

		await Promise.all(
			disablingIds.map(async (uuid) => {
				const deps = await api.profile.getDependants(uuid);
				for (const dep of deps) {
					const depMod = mods.find((m) => dep.includes(m.name));
					if (depMod && !selectedSet.has(depMod.uuid)) {
						extraDeps.add(dep);
					} else if (!depMod) {
						extraDeps.add(dep);
					}
				}
			})
		);

		if (extraDeps.size > 0) {
			batchToggleDialog = {
				open: true,
				extraDependants: [...extraDeps]
			};
		} else {
			await api.profile.forceToggleMods(selectedModIds);
			await refresh();
		}
	}

	async function doBatchForceToggleAll() {
		await api.profile.forceToggleMods(selectedModIds);
		batchToggleDialog = null;
		await refresh();
	}

	async function doBatchForceToggleCancel() {
		batchToggleDialog = null;
	}

	function openModContextMenu(e: MouseEvent, mod: Mod) {
		const locked = profiles.activeLocked;
		const items: ContextMenuItem[] = [];

		// Toggle enable/disable
		if (mod.isInstalled) {
			items.push({
				label: mod.enabled === false ? m.mods_contextMenu_enable() : m.mods_contextMenu_disable(),
				icon: mod.enabled === false ? 'mdi:eye' : 'mdi:eye-off',
				disabled: locked,
				onclick: () => toggleMod(mod)
			});
		}

		// Pin/unpin
		if (mod.isInstalled) {
			const pinned = isModPinned(mod.uuid);
			items.push({
				label: pinned ? 'Désépingler' : 'Épingler en haut',
				icon: pinned ? 'mdi:pin-off' : 'mdi:pin',
				onclick: () => togglePin(mod.uuid)
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

		// Donate
		if (mod.donateUrl) {
			items.push({
				label: m.mods_contextMenu_donate(),
				icon: 'mdi:heart',
				onclick: () => open(mod.donateUrl!)
			});
		}

		// Open mod folder
		if (mod.isInstalled) {
			items.push({
				label: m.mods_contextMenu_openFolder(),
				icon: 'mdi:folder-open',
				onclick: () => api.profile.openModDir(mod.uuid)
			});
		}

		// Config file
		if (mod.configFile) {
			items.push({
				label: m.mods_contextMenu_editConfig(),
				icon: 'mdi:file-cog',
				onclick: () => {
					// Navigate to config page
					window.location.href = '/config';
				}
			});
		}

		// Separator + destructive
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

	$effect(() => {
		if (maxCount > 0) {
			profileQuery.current;
			profiles.active;
			refresh();
		}
	});

	let locked = $derived(profiles.activeLocked);

	// Sort pinned mods to top
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
		if (isAllSelected) {
			selectedModIds = [];
		} else {
			selectAll();
		}
	}

	let shareDialog: { open: boolean; mode: 'export' | 'import' } = $state({ open: false, mode: 'export' });
</script>

{#if !profiles.ready}
	<Loader />
{:else}
<div class="z-mods-page">
	<div class="z-mods-main">
		<Header
			title={i18nState.locale && m.navBar_label_mods()}
			subtitle={i18nState.locale && m.mods_installed({ count: totalModCount.toString() })}
		>
			{#snippet actions()}
				<Button variant="ghost" size="sm" onclick={() => { shareDialog = { open: true, mode: 'import' }; }}>
					{#snippet icon()}<Icon icon="mdi:import" />{/snippet}
					{i18nState.locale && m.share_importButton()}
				</Button>
				<Button variant="ghost" size="sm" onclick={() => { shareDialog = { open: true, mode: 'export' }; }}>
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

		<div class="z-mods-content">
			<div class="z-mods-filters">
				<div class="z-mods-filters-row">
					<label class="z-master-checkbox-wrapper">
						<Checkbox checked={isAllSelected} onchange={toggleSelectAll} />
						<span class="z-master-checkbox-label">{i18nState.locale && m.batch_selectAll()}</span>
					</label>
					<div class="flex-1"></div>
					<ModListFilters queryArgs={profileQuery.current} {sortOptions} />
				</div>
			</div>

			{#if unknownMods.length > 0}
				<div class="z-unknown-banner">
					<Icon icon="mdi:help-circle" />
					<span>{m.mods_unknownMods({ count: unknownMods.length.toString() })}</span>
				</div>
			{/if}

			<div class="z-mods-list">
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
						{#if draggedMod && placeholderIndex === i}
							<div class="z-drop-placeholder">
								<div class="z-drop-line"></div>
								<Icon icon="mdi:plus-circle" class="z-drop-icon" />
								<div class="z-drop-line"></div>
							</div>
						{/if}

						<div data-mod-index={i} class:z-dragging-card={draggedMod?.uuid === mod.uuid}>
							<ModCard
								{mod}
								isSelected={selectedModIds.includes(mod.uuid)}
								{locked}
								showInstallBtn={false}
								showDragHandle={canDrag}
								onclick={(evt: MouseEvent) => {
									if (!draggedMod) handleModClick(evt, mod, i);
								}}
								oncontextmenu={openModContextMenu}
								onpointerdownHandle={handleDragHandleDown}
							/>
						</div>
					{/each}

					{#if draggedMod && placeholderIndex === sortedMods.length}
						<div class="z-drop-placeholder">
							<div class="z-drop-line"></div>
							<Icon icon="mdi:plus-circle" class="z-drop-icon" />
							<div class="z-drop-line"></div>
						</div>
					{/if}

					{#if mods.length < totalModCount}
						<button class="z-load-more" onclick={() => (maxCount += 40)}>
							{m.mods_loadMore({ count: (totalModCount - mods.length).toString() })}
						</button>
					{/if}
				{/if}
			</div>
		</div>
	</div>

	{#if selectedMod}
		<ModDetails
			mod={selectedMod}
			{locked}
			onclose={() => (selectedModIds = [])}
			ontoggle={() => toggleMod(selectedMod!)}
			onremove={() => removeMod(selectedMod!)}
		>
			<InstallModButton mod={selectedMod} {install} {locked} />
		</ModDetails>
	{/if}

	<BatchActionBar
		count={selectedModIds.length}
		total={sortedMods.length}
		onclear={() => (selectedModIds = [])}
		onselectAll={selectAll}
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

<!-- Context menu -->
{#if ctxMenu}
	<ContextMenu items={ctxMenu.items} x={ctxMenu.x} y={ctxMenu.y} onclose={() => (ctxMenu = null)} />
{/if}

<!-- Remove mod confirmation dialog -->
{#if removeDialog}
	<RemoveModDialog
		bind:open={removeDialog.open}
		modName={removeDialog.mod.name}
		dependants={removeDialog.dependants}
		oncancel={() => (removeDialog = null)}
		onremoveOne={() => doForceRemoveOne(removeDialog!.mod)}
		onremoveAll={() => doForceRemoveAll(removeDialog!.mod, removeDialog!.dependants)}
	/>
{/if}

<!-- Toggle mod confirmation dialog -->
{#if toggleDialog}
	<ToggleModDialog
		bind:open={toggleDialog.open}
		modName={toggleDialog.mod.name}
		isDisabling={toggleDialog.isDisabling}
		dependants={toggleDialog.dependants}
		oncancel={() => (toggleDialog = null)}
		ontoggleOne={() => doForceToggleOne(toggleDialog!.mod)}
		ontoggleAll={() => doForceToggleAll(toggleDialog!.mod, toggleDialog!.dependants)}
	/>
{/if}

<!-- Batch toggle confirmation dialog -->
{#if batchToggleDialog}
	<Modal
		bind:open={batchToggleDialog.open}
		title={i18nState.locale && m.batchToggleDialog_title({ count: selectedModIds.length.toString() })}
		onclose={doBatchForceToggleCancel}
	>
		{#snippet children()}
			<div class="z-rmd-body">
				<div class="z-rmd-warning">
					<Icon icon="mdi:alert-circle" class="z-rmd-warning-icon" />
					<p>{i18nState.locale && m.batchToggleDialog_description()}</p>
				</div>

				<ul class="z-rmd-list">
					{#each batchToggleDialog!.extraDependants as dep}
						<li class="z-rmd-item">
							<Icon icon="mdi:puzzle" />
							<span>{dep}</span>
						</li>
					{/each}
				</ul>

				<p class="z-rmd-hint">{i18nState.locale && m.batchToggleDialog_hint()}</p>
			</div>
		{/snippet}

		{#snippet actions()}
			<Button variant="ghost" onclick={doBatchForceToggleCancel}>
				{i18nState.locale && m.batchToggleDialog_cancel()}
			</Button>
			<Button variant="accent" onclick={doBatchForceToggleAll}>
				{#snippet icon()}<Icon icon="mdi:toggle-switch" />{/snippet}
				{i18nState.locale && m.batchToggleDialog_confirm()}
			</Button>
		{/snippet}
	</Modal>
{/if}

<!-- Batch remove confirmation dialog -->
{#if batchRemoveDialog}
	<Modal
		bind:open={batchRemoveDialog.open}
		title={i18nState.locale && m.batchRemoveDialog_title({ count: selectedModIds.length.toString() })}
		onclose={doBatchForceRemoveCancel}
	>
		{#snippet children()}
			<div class="z-rmd-body">
				<div class="z-rmd-warning">
					<Icon icon="mdi:alert-circle" class="z-rmd-warning-icon" />
					<p>{i18nState.locale && m.batchRemoveDialog_description()}</p>
				</div>

				<ul class="z-rmd-list">
					{#each batchRemoveDialog!.extraDependants as dep}
						<li class="z-rmd-item">
							<Icon icon="mdi:puzzle" />
							<span>{dep}</span>
						</li>
					{/each}
				</ul>

				<p class="z-rmd-hint">{i18nState.locale && m.batchRemoveDialog_hint()}</p>
			</div>
		{/snippet}

		{#snippet actions()}
			<Button variant="ghost" onclick={doBatchForceRemoveCancel}>
				{i18nState.locale && m.batchRemoveDialog_cancel()}
			</Button>
			<Button variant="danger" onclick={doBatchForceRemoveAll}>
				{#snippet icon()}<Icon icon="mdi:delete-sweep" />{/snippet}
				{i18nState.locale && m.batchRemoveDialog_confirm({ count: selectedModIds.length.toString() })}
			</Button>
		{/snippet}
	</Modal>
{/if}

<ShareProfileDialog
	bind:mode={shareDialog.mode}
	bind:open={shareDialog.open}
	onclose={() => { shareDialog.open = false; }}
/>
{/if}

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
	}

	.z-mods-content {
		flex: 1;
		overflow-y: auto;
		padding: 0 var(--space-xl);
		padding-bottom: var(--space-xl);
	}

	.z-mods-filters {
		position: sticky;
		top: 0;
		z-index: 10;
		padding-top: var(--space-sm);
		background: var(--bg-base);
	}

	.z-mods-filters-row {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding-bottom: var(--space-xs);
		border-bottom: 1px solid var(--border-subtle);
		margin-bottom: var(--space-sm);
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
	}

	/* Empty state */
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
		background: rgba(26, 255, 250, 0.12);
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

	/* Drop placeholder */
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

	/* Batch remove dialog */
	.z-rmd-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.z-rmd-warning {
		display: flex;
		align-items: flex-start;
		gap: var(--space-sm);
		padding: var(--space-md);
		border-radius: var(--radius-md);
		background: rgba(255, 179, 71, 0.08);
		border: 1px solid rgba(255, 179, 71, 0.25);
		color: var(--warning);
		font-size: 13px;
		line-height: 1.5;
	}

	:global(.z-rmd-warning-icon) {
		flex-shrink: 0;
		font-size: 18px;
		margin-top: 1px;
	}

	.z-rmd-list {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
		max-height: 220px;
		overflow-y: auto;
	}

	.z-rmd-item {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-sm);
		background: var(--bg-elevated);
		border: 1px solid var(--border-subtle);
		font-size: 12px;
		color: var(--text-secondary);
	}

	.z-rmd-item :global(svg) {
		color: var(--text-muted);
		font-size: 14px;
		flex-shrink: 0;
	}

	.z-rmd-hint {
		font-size: 12px;
		color: var(--text-muted);
		line-height: 1.5;
	}
</style>
