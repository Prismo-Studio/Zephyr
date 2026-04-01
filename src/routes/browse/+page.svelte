<script lang="ts">
	import * as api from '$lib/api';
	import type { SortBy, Mod, ModId, Dependant } from '$lib/types';

	import ModCard from '$lib/components/mod-list/ModCard.svelte';
	import ModDetails from '$lib/components/mod-list/ModDetails.svelte';
	import ModListFilters from '$lib/components/mod-list/ModListFilters.svelte';
	import InstallModButton from '$lib/components/mod-list/InstallModButton.svelte';
	import RemoveModDialog from '$lib/components/mod-list/RemoveModDialog.svelte';
	import ToggleModDialog from '$lib/components/mod-list/ToggleModDialog.svelte';
	import Header from '$lib/components/layout/Header.svelte';

	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { modQuery, togglePin, isModPinned, pinnedMods } from '$lib/state/misc.svelte';
	import ContextMenu from '$lib/components/ui/ContextMenu.svelte';
	import type { ContextMenuItem } from '$lib/components/ui/ContextMenu.svelte';
	import { open } from '@tauri-apps/plugin-shell';
	import profiles from '$lib/state/profile.svelte';
	import Icon from '@iconify/svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { pushToast } from '$lib/toast';

	const sortOptions: SortBy[] = ['lastUpdated', 'newest', 'rating', 'downloads'];

	let mods: Mod[] = $state([]);
	let maxCount: number = $state(30);
	let selectedMod: Mod | null = $state(null);
	let hasRefreshed = $state(false);

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

	async function refresh() {
		if (refreshing) return;
		refreshing = true;

		try {
			mods = await api.thunderstore.query({ ...modQuery.current, maxCount });
			if (selectedMod) {
				selectedMod = mods.find((mod) => mod.uuid === selectedMod!.uuid) ?? null;
			}
		} catch {}

		refreshing = false;
		hasRefreshed = true;
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
		// Force update selectedMod with fresh data
		if (selectedMod) {
			selectedMod = mods.find((m) => m.uuid === selectedMod!.uuid) ?? null;
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
			if (selectedMod?.uuid === mod.uuid) selectedMod = null;
			await refresh();
		} else if (response.type === 'confirm') {
			// Show cascade-delete dialog
			removeDialog = { open: true, mod, dependants: response.dependants };
		}
	}

	async function doForceRemoveOne(mod: Mod) {
		await api.profile.forceRemoveMods([mod.uuid]);
		if (selectedMod?.uuid === mod.uuid) selectedMod = null;
		removeDialog = null;
		await refresh();
	}

	async function doForceRemoveAll(mod: Mod, dependants: Dependant[]) {
		const uuids = [mod.uuid, ...dependants.map((d) => d.uuid)];
		await api.profile.forceRemoveMods(uuids);
		if (selectedMod && uuids.includes(selectedMod.uuid)) selectedMod = null;
		removeDialog = null;
		await refresh();
	}

	function onModClicked(evt: MouseEvent, mod: Mod) {
		if (evt.ctrlKey) {
			installLatest(mod);
		} else {
			selectedMod = selectedMod?.uuid === mod.uuid ? null : mod;
		}
	}

	$effect(() => {
		if (maxCount > 0) {
			modQuery.current;
			profiles.active;
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
				label: 'Installer',
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
		<Header title={i18nState.locale && m.navBar_label_browse()} subtitle="Thunderstore">
			{#snippet actions()}
				<button
					class="z-refresh-btn"
					onclick={() => {
						api.thunderstore.triggerModFetch();
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
				<ModListFilters queryArgs={modQuery.current} {sortOptions} showCategories />
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
					<div class="z-browse-loading">
						<span class="z-browse-spinner"></span>
						<span>{i18nState.locale && m.browse_loading()}</span>
					</div>
				{:else}
					{#each mods as mod (mod.uuid)}
						<ModCard
							{mod}
							isSelected={selectedMod?.uuid === mod.uuid}
							{locked}
							onclick={(evt: MouseEvent) => onModClicked(evt, mod)}
							oninstall={() => installLatest(mod)}
							oncontextmenu={openModContextMenu}
						/>
					{/each}

					<button class="z-load-more" onclick={() => (maxCount += 30)}>
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
			onclose={() => (selectedMod = null)}
			ontoggle={() => toggleMod(selectedMod!)}
			onremove={() => removeMod(selectedMod!)}
		>
			<InstallModButton mod={selectedMod} {install} {locked} />
		</ModDetails>
	{/if}
</div>

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
		ontoggleOne={() => doForceToggleOne(toggleDialog!.mod)}
		ontoggleAll={() => doForceToggleAll(toggleDialog!.mod, toggleDialog!.dependants)}
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
		background: var(--bg-base);
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

	.z-load-more:hover {
		border-color: var(--border-accent);
		color: var(--text-accent);
		background: var(--bg-hover);
	}
</style>
