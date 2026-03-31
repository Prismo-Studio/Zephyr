<script lang="ts">
	import * as api from '$lib/api';
	import type { Mod, ModId, SortBy } from '$lib/types';

	import ModCard from '$lib/components/mod-list/ModCard.svelte';
	import ModDetails from '$lib/components/mod-list/ModDetails.svelte';
	import ModListFilters from '$lib/components/mod-list/ModListFilters.svelte';
	import InstallModButton from '$lib/components/mod-list/InstallModButton.svelte';
	import Header from '$lib/components/layout/Header.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import ContextMenu from '$lib/components/ui/ContextMenu.svelte';
	import type { ContextMenuItem } from '$lib/components/ui/ContextMenu.svelte';

	import { onMount } from 'svelte';
	import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { open } from '@tauri-apps/plugin-shell';
	import { profileQuery } from '$lib/state/misc.svelte';
	import profiles from '$lib/state/profile.svelte';
	import Icon from '@iconify/svelte';
	import type { AvailableUpdate, ProfileQuery } from '$lib/types';
	import { communityUrl, isOutdated } from '$lib/util';

	const sortOptions: SortBy[] = ['custom', 'name', 'author', 'installDate', 'diskSpace'];

	let mods: Mod[] = $state([]);
	let updates: AvailableUpdate[] = $state([]);
	let unknownMods: { fullName: string; uuid: string }[] = $state([]);
	let totalModCount = $state(0);
	let maxCount = $state(40);
	let selectedMod: Mod | null = $state(null);

	// Context menu state
	let ctxMenu: { items: ContextMenuItem[]; x: number; y: number } | null = $state(null);

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

			if (selectedMod) {
				selectedMod = mods.find((m) => m.uuid === selectedMod!.uuid) ?? null;
			}
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
		const response = await api.profile.toggleMod(mod.uuid);
		if (response.type === 'done') {
			await refresh();
		} else if (response.type === 'hasDependants') {
			await api.profile.forceToggleMods([mod.uuid, ...response.dependants.map(d => d.uuid)]);
			await refresh();
		}
	}

	async function removeMod(mod: Mod) {
		const response = await api.profile.removeMod(mod.uuid);
		if (response.type === 'done') {
			if (selectedMod?.uuid === mod.uuid) selectedMod = null;
			await refresh();
		} else if (response.type === 'hasDependants') {
			await api.profile.forceRemoveMods([mod.uuid, ...response.dependants.map(d => d.uuid)]);
			if (selectedMod?.uuid === mod.uuid) selectedMod = null;
			await refresh();
		}
	}

	async function updateAllMods() {
		const uuids = updates.filter((u) => !u.ignore).map((u) => u.packageUuid);
		if (uuids.length > 0) {
			await api.profile.update.mods(uuids, true);
			await refresh();
		}
	}

	function openModContextMenu(e: MouseEvent, mod: Mod) {
		const locked = profiles.activeLocked;
		const items: ContextMenuItem[] = [];

		// Toggle enable/disable
		if (mod.isInstalled) {
			items.push({
				label: mod.enabled === false ? 'Enable' : 'Disable',
				icon: mod.enabled === false ? 'mdi:eye' : 'mdi:eye-off',
				disabled: locked,
				onclick: () => toggleMod(mod)
			});
		}

		// Open website
		if (mod.websiteUrl) {
			items.push({
				label: 'Open on Thunderstore',
				icon: 'mdi:open-in-new',
				onclick: () => open(mod.websiteUrl!)
			});
		}

		// Donate
		if (mod.donateUrl) {
			items.push({
				label: 'Donate',
				icon: 'mdi:heart',
				onclick: () => open(mod.donateUrl!)
			});
		}

		// Open mod folder
		if (mod.isInstalled) {
			items.push({
				label: 'Open folder',
				icon: 'mdi:folder-open',
				onclick: () => api.profile.openModDir(mod.uuid)
			});
		}

		// Config file
		if (mod.configFile) {
			items.push({
				label: 'Edit config',
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
				label: 'Uninstall',
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
</script>

<div class="z-mods-page">
	<div class="z-mods-main">
		<Header title="Mods" subtitle="{totalModCount} installed">
			{#snippet actions()}
				{#if updates.length > 0}
					<Button variant="accent" size="sm" onclick={updateAllMods}>
						{#snippet icon()}<Icon icon="mdi:update" />{/snippet}
						Update {updates.length}
					</Button>
				{/if}
				{#if locked}
					<Badge variant="warning">
						<Icon icon="mdi:lock" class="text-xs" /> Locked
					</Badge>
				{/if}
			{/snippet}
		</Header>

		<div class="z-mods-content">
			<div class="z-mods-filters">
				<ModListFilters queryArgs={profileQuery.current} {sortOptions} />
			</div>

			{#if unknownMods.length > 0}
				<div class="z-unknown-banner">
					<Icon icon="mdi:help-circle" />
					<span>{unknownMods.length} unknown mod{unknownMods.length > 1 ? 's' : ''} in profile</span>
				</div>
			{/if}

			<div class="z-mods-list">
				{#if mods.length === 0}
					<div class="z-mods-empty">
						<div class="z-empty-icon">
							<Icon icon="mdi:package-variant" />
						</div>
						<p class="z-empty-title">No mods installed</p>
						<p class="z-empty-desc">Browse the store to find mods for your game</p>
						<a href="/browse" class="z-empty-action">
							<Icon icon="mdi:store-search" />
							Browse mods
						</a>
					</div>
				{:else}
					{#each mods as mod (mod.uuid)}
						<ModCard
							{mod}
							isSelected={selectedMod?.uuid === mod.uuid}
							locked={locked}
							showInstallBtn={false}
							onclick={() => (selectedMod = selectedMod?.uuid === mod.uuid ? null : mod)}
							oncontextmenu={openModContextMenu}
						/>
					{/each}

					{#if mods.length < totalModCount}
						<button class="z-load-more" onclick={() => (maxCount += 40)}>
							Load more ({totalModCount - mods.length} remaining)
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
			onclose={() => (selectedMod = null)}
			ontoggle={() => toggleMod(selectedMod!)}
			onremove={() => removeMod(selectedMod!)}
		>
			<InstallModButton mod={selectedMod} {install} {locked} />
		</ModDetails>
	{/if}
</div>

<!-- Context menu -->
{#if ctxMenu}
	<ContextMenu
		items={ctxMenu.items}
		x={ctxMenu.x}
		y={ctxMenu.y}
		onclose={() => (ctxMenu = null)}
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

	.z-mods-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
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
</style>
