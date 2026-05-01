<script lang="ts">
	import Header from '$lib/components/layout/Header.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import Icon from '@iconify/svelte';

	import ProfileAuthPill from '$lib/components/profiles/ProfileAuthPill.svelte';
	import ProfileCard from '$lib/components/profiles/ProfileCard.svelte';
	import CreateProfileModal from '$lib/components/profiles/CreateProfileModal.svelte';
	import RenameProfileModal from '$lib/components/profiles/RenameProfileModal.svelte';
	import DisconnectSyncModal from '$lib/components/profiles/DisconnectSyncModal.svelte';
	import RestoreCloudDialog from '$lib/components/dialogs/RestoreCloudDialog.svelte';
	import RestoreCrossGameDialog from '$lib/components/dialogs/RestoreCrossGameDialog.svelte';

	import profiles from '$lib/state/profile.svelte';
	import auth from '$lib/state/auth.svelte';
	import games from '$lib/state/game.svelte';
	import * as api from '$lib/api';
	import { m } from '$lib/paraglide/messages';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { pushToast } from '$lib/toast.svelte';
	import { maybeSyncAfterImport } from '$lib/state/autoSync.svelte';
	import { createProfileDrag } from '$lib/utils/useProfileDrag.svelte';
	import type { ListedSyncProfile, Game } from '$lib/types';

	const MAX_PROFILES = 20;

	let createOpen = $state(false);
	let renameId: number | null = $state(null);
	let renameInitialName = $state('');

	let restoreOpen = $state(false);
	let restoreItems: ListedSyncProfile[] = $state([]);
	let crossGameOpen = $state(false);
	let crossGameBuckets: { slug: string; name: string; game: Game | null; count: number }[] = $state(
		[]
	);

	let disconnectId: number | null = $state(null);
	let disconnectName = $state('');
	let disconnectAlsoDelete = $state(false);
	let disconnectBusy = $state(false);

	const drag = createProfileDrag({ refresh: () => profiles.refresh() });

	function getProfileIndex(id: number): number {
		return profiles.list.findIndex((p) => p.id === id);
	}

	function checkProfileLimit(): boolean {
		if (profiles.list.length >= MAX_PROFILES) {
			pushToast({
				type: 'error',
				name: m.sync_limitReached(),
				message: m.sync_limitReachedDesc({ max: MAX_PROFILES.toString() })
			});
			return false;
		}
		return true;
	}

	function askDisconnect(id: number, name: string) {
		disconnectId = id;
		disconnectName = name;
		const p = profiles.list.find((pr) => pr.id === id);
		disconnectAlsoDelete = !!(
			p?.sync &&
			auth.user &&
			p.sync.owner.discordId === auth.user.discordId
		);
	}

	async function confirmDisconnect() {
		if (disconnectId === null) return;
		disconnectBusy = true;
		try {
			const idx = getProfileIndex(disconnectId);
			if (idx !== -1 && disconnectId !== profiles.activeId) {
				await api.profile.setActive(idx);
			}
			await api.profile.sync.disconnect(disconnectAlsoDelete);
			await profiles.refresh();
			pushToast({
				type: 'info',
				name: m.sync_unsynced(),
				message: disconnectAlsoDelete
					? m.sync_removedFromCloud({ name: disconnectName })
					: m.sync_unlinkedFromCloud({ name: disconnectName })
			});
			disconnectId = null;
		} catch (e: any) {
			pushToast({ type: 'error', name: m.sync_unsyncFailed(), message: e?.message ?? String(e) });
		} finally {
			disconnectBusy = false;
		}
	}

	async function checkForRestorable(silent: boolean = false) {
		if (!auth.user) return;
		try {
			const owned = await api.profile.sync.getOwned();
			const activeSlug = games.active?.slug;
			const allSyncIds = await api.profile.getAllSyncIds();
			const localSyncIds = new Set(allSyncIds);

			const notLocal = owned.filter((p) => !localSyncIds.has(p.id));
			const forCurrentGame = notLocal.filter(
				(p) => !activeSlug || !p.community || p.community === activeSlug
			);
			const forOtherGames = notLocal.filter(
				(p) => activeSlug && p.community && p.community !== activeSlug
			);

			if (silent) return;

			if (forCurrentGame.length > 0) {
				restoreItems = forCurrentGame;
				restoreOpen = true;
				return;
			}

			if (forOtherGames.length > 0) {
				const countBySlug = new Map<string, number>();
				for (const p of forOtherGames) {
					const slug = p.community ?? 'unknown';
					countBySlug.set(slug, (countBySlug.get(slug) ?? 0) + 1);
				}
				crossGameBuckets = Array.from(countBySlug.entries()).map(([slug, count]) => {
					const game = games.list.find((g) => g.slug === slug) ?? null;
					return { slug, name: game?.name ?? slug, game, count };
				});
				crossGameOpen = true;
				return;
			}

			pushToast({
				type: 'info',
				name: m.sync_nothingToRestore(),
				message: m.sync_nothingToRestoreDesc()
			});
		} catch (e: any) {
			if (!silent) {
				pushToast({
					type: 'error',
					name: m.sync_restoreCheckFailed(),
					message: e?.message ?? String(e)
				});
			}
		}
	}

	let autoSync = $state(false);
	(async () => {
		try {
			const p = await api.prefs.get();
			autoSync = p.pullBeforeLaunch;
		} catch {}
	})();

	async function createProfile(name: string) {
		if (!checkProfileLimit()) return;
		await api.profile.create(name, null);
		await profiles.refresh();
		if (autoSync && auth.user) {
			try {
				await api.profile.sync.create();
				await profiles.refresh();
			} catch (e: any) {
				pushToast({
					type: 'error',
					name: m.sync_autoSyncFailed(),
					message: e?.message ?? String(e)
				});
			}
		}
		createOpen = false;
	}

	async function deleteProfile(id: number) {
		let syncId: string | null = null;
		try {
			const info = await api.profile.getInfo();
			const fresh = info.profiles.find((pr) => pr.id === id);
			if (fresh?.sync && auth.user && fresh.sync.owner.discordId === auth.user.discordId) {
				syncId = fresh.sync.id;
			}
		} catch {}

		if (syncId) {
			try {
				await api.profile.sync.deleteProfile(syncId);
			} catch (e: any) {
				pushToast({
					type: 'error',
					name: m.sync_cloudDeleteFailed(),
					message: e?.message ?? String(e)
				});
			}
		}
		await api.profile.deleteProfile(id);
		await profiles.refresh();
	}

	function getUniqueCopyName(baseName: string): string {
		const existingNames = new Set(profiles.list.map((p) => p.name));
		const copyName = `${baseName} (copy)`;
		if (!existingNames.has(copyName)) return copyName;
		let i = 2;
		while (existingNames.has(`${baseName} (copy ${i})`)) i++;
		return `${baseName} (copy ${i})`;
	}

	async function duplicateProfile(id: number, name: string) {
		if (!checkProfileLimit()) return;
		const index = getProfileIndex(id);
		if (index === -1) return;
		if (id !== profiles.activeId) {
			await api.profile.setActive(index);
		}
		await api.profile.duplicate(getUniqueCopyName(name));
		await profiles.refresh();
		await maybeSyncAfterImport({ forceFork: true });
	}

	function startRename(id: number, name: string) {
		renameId = id;
		renameInitialName = name;
	}

	async function confirmRename(newName: string) {
		if (renameId === null) return;
		const index = getProfileIndex(renameId);
		if (index === -1) return;
		const target = profiles.list.find((pr) => pr.id === renameId);
		const isOwnedSync = !!(
			target?.sync &&
			auth.user &&
			target.sync.owner.discordId === auth.user.discordId
		);
		const previousActiveIndex =
			profiles.activeId !== null ? getProfileIndex(profiles.activeId) : -1;
		const needsSwitch = renameId !== profiles.activeId;
		if (needsSwitch) {
			await api.profile.setActive(index);
		}
		await api.profile.rename(newName);
		if (isOwnedSync) {
			try {
				await api.profile.sync.push();
			} catch (e: any) {
				pushToast({ type: 'error', name: m.sync_syncFailed(), message: e?.message ?? String(e) });
			}
		}
		if (needsSwitch && previousActiveIndex !== -1) {
			await api.profile.setActive(previousActiveIndex);
		}
		await profiles.refresh();
		renameId = null;
	}

	async function selectProfile(id: number) {
		const index = getProfileIndex(id);
		if (index === -1) return;
		await profiles.setActive(index);
		await profiles.refresh();
	}

	async function pickIcon(profileId: number) {
		const file = await openDialog({
			filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif', 'bmp'] }],
			multiple: false
		});
		if (!file) return;

		pushToast({
			type: 'info',
			message: (i18nState.locale && m.profiles_uploadingIcon()) || 'Uploading...'
		});
		try {
			await api.profile.uploadProfileIcon(profileId, file);
			await profiles.refresh();
			pushToast({
				type: 'info',
				message: (i18nState.locale && m.profiles_iconUploaded()) || 'Done!'
			});
		} catch (e) {
			pushToast({ type: 'error', message: String(e) });
		}
	}

	async function syncToCloud(profile: { id: number; name: string }) {
		try {
			if (profile.id !== profiles.activeId) {
				const idx = getProfileIndex(profile.id);
				if (idx !== -1) await api.profile.setActive(idx);
			}
			const id = await api.profile.sync.create();
			pushToast({
				type: 'info',
				name: m.sync_synced(),
				message: m.sync_uploaded({ name: profile.name, id: id.slice(0, 8) + '…' })
			});
			await profiles.refresh();
		} catch (e: any) {
			pushToast({ type: 'error', name: m.sync_syncFailed(), message: e?.message ?? String(e) });
		}
	}

	async function signIn() {
		try {
			await auth.login();
			pushToast({
				type: 'info',
				name: m.sync_signedIn(),
				message: m.sync_welcome({ name: auth.user?.displayName ?? '' })
			});
		} catch (e: any) {
			pushToast({ type: 'error', name: m.sync_signInFailed(), message: e?.message ?? String(e) });
		}
	}
</script>

<div class="z-profiles-page">
	<Header
		title={i18nState.locale && m.menuBar_profile_title()}
		subtitle={i18nState.locale && m.profiles_total({ count: profiles.list.length.toString() })}
	>
		{#snippet actions()}
			{#if auth.user}
				<Tooltip text={(i18nState.locale && m.sync_restoreTooltip()) || ''} position="bottom" delay={200}>
					<Button variant="secondary" size="sm" onclick={() => checkForRestorable(false)}>
						{#snippet icon()}<Icon icon="mdi:cloud-download" />{/snippet}
						{i18nState.locale && m.sync_restore()}
					</Button>
				</Tooltip>
				<ProfileAuthPill />
			{:else}
				<Button variant="secondary" size="sm" onclick={signIn}>
					{#snippet icon()}<Icon icon="mdi:discord" />{/snippet}
					{i18nState.locale && m.sync_signIn()}
				</Button>
			{/if}
			<Button variant="primary" size="sm" onclick={() => (createOpen = true)}>
				{#snippet icon()}<Icon icon="mdi:plus" />{/snippet}
				{i18nState.locale && m.profiles_newProfile()}
			</Button>
		{/snippet}
	</Header>

	<div class="z-profiles-content">
		<div class="z-profiles-grid">
			{#each profiles.list as profile, i (profile.id)}
				<ProfileCard
					{profile}
					index={i}
					isActive={profile.id === profiles.activeId}
					isDragging={drag.draggingId === profile.id}
					isDragOver={drag.dragOverIdx === i &&
						drag.draggingId !== null &&
						drag.draggingId !== profile.id}
					onpointerdown={(e, p, idx) => drag.onPointerDown(e, p.id, idx)}
					onselect={selectProfile}
					onpickIcon={pickIcon}
					onsyncToCloud={syncToCloud}
					onaskDisconnect={askDisconnect}
					onstartRename={startRename}
					onduplicate={duplicateProfile}
					ondelete={deleteProfile}
				/>
			{/each}
		</div>
	</div>
</div>

<RestoreCloudDialog
	bind:open={restoreOpen}
	items={restoreItems}
	onclose={() => (restoreItems = [])}
/>

<RestoreCrossGameDialog
	bind:open={crossGameOpen}
	buckets={crossGameBuckets}
	onclose={() => (crossGameBuckets = [])}
	onPicked={() => checkForRestorable(false)}
/>

<CreateProfileModal bind:open={createOpen} oncreate={createProfile} />

<DisconnectSyncModal
	open={disconnectId !== null}
	profileName={disconnectName}
	bind:alsoDelete={disconnectAlsoDelete}
	busy={disconnectBusy}
	onconfirm={confirmDisconnect}
	onclose={() => {
		if (!disconnectBusy) disconnectId = null;
	}}
/>

<RenameProfileModal
	open={renameId !== null}
	initialName={renameInitialName}
	onrename={confirmRename}
	onclose={() => (renameId = null)}
/>

<style>
	.z-profiles-page {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.z-profiles-content {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-xl);
	}

	.z-profiles-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
		gap: var(--space-md);
	}
</style>
