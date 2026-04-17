<script lang="ts">
	import Header from '$lib/components/layout/Header.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import Icon from '@iconify/svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';

	import profiles from '$lib/state/profile.svelte';
	import auth from '$lib/state/auth.svelte';
	import games from '$lib/state/game.svelte';
	import * as api from '$lib/api';
	import { m } from '$lib/paraglide/messages';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { pushToast } from '$lib/toast';
	import RestoreCloudDialog from '$lib/components/dialogs/RestoreCloudDialog.svelte';
	import RestoreCrossGameDialog from '$lib/components/dialogs/RestoreCrossGameDialog.svelte';
	import type { ListedSyncProfile, Game } from '$lib/types';
	import { maybeSyncAfterImport } from '$lib/state/autoSync.svelte';

	const MAX_PROFILES = 20;

	let createOpen = $state(false);
	let newName = $state('');
	let renameId: number | null = $state(null);
	let renameName = $state('');
	let autoSync = $state(false);

	let restoreOpen = $state(false);
	let restoreItems: ListedSyncProfile[] = $state([]);
	let crossGameOpen = $state(false);
	let crossGameBuckets: { slug: string; name: string; game: Game | null; count: number }[] =
		$state([]);

	let disconnectId: number | null = $state(null);
	let disconnectName = $state('');
	let disconnectAlsoDelete = $state(false);
	let disconnectBusy = $state(false);

	function askDisconnect(id: number, name: string) {
		disconnectId = id;
		disconnectName = name;
		const p = profiles.list.find((pr) => pr.id === id);
		disconnectAlsoDelete = !!(
			p?.sync && auth.user && p.sync.owner.discordId === auth.user.discordId
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
			pushToast({
				type: 'error',
				name: m.sync_unsyncFailed(),
				message: e?.message ?? String(e)
			});
		} finally {
			disconnectBusy = false;
		}
	}

	async function checkForRestorable(silent: boolean = false) {
		if (!auth.user) return;
		try {
			const owned = await api.profile.sync.getOwned();
			const activeSlug = games.active?.slug;
			const info = await api.profile.getInfo();
			const localSyncIds = new Set(
				info.profiles.map((p) => p.sync?.id).filter((id): id is string => !!id)
			);

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
					return {
						slug,
						name: game?.name ?? slug,
						game,
						count
					};
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

	(async () => {
		try {
			const p = await api.prefs.get();
			autoSync = p.pullBeforeLaunch;
		} catch {}
	})();

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

	async function createProfile() {
		if (!newName.trim()) return;
		if (!checkProfileLimit()) return;
		await api.profile.create(newName.trim(), null);
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
		newName = '';
		createOpen = false;
	}

	async function deleteProfile(id: number) {
		let syncId: string | null = null;
		try {
			const info = await api.profile.getInfo();
			const fresh = info.profiles.find((pr) => pr.id === id);
			if (
				fresh?.sync &&
				auth.user &&
				fresh.sync.owner.discordId === auth.user.discordId
			) {
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
		while (existingNames.has(`${baseName} (copy ${i})`)) {
			i++;
		}
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

	async function startRename(id: number, name: string) {
		renameId = id;
		renameName = name;
	}

	async function confirmRename() {
		if (!renameName.trim() || renameId === null) return;
		const index = getProfileIndex(renameId);
		if (index === -1) return;
		const target = profiles.list.find((pr) => pr.id === renameId);
		const isOwnedSync = !!(
			target?.sync && auth.user && target.sync.owner.discordId === auth.user.discordId
		);
		const previousActiveIndex =
			profiles.activeId !== null ? getProfileIndex(profiles.activeId) : -1;
		const needsSwitch = renameId !== profiles.activeId;
		if (needsSwitch) {
			await api.profile.setActive(index);
		}
		await api.profile.rename(renameName.trim());
		if (isOwnedSync) {
			try {
				await api.profile.sync.push();
			} catch (e: any) {
				pushToast({
					type: 'error',
					name: m.sync_syncFailed(),
					message: e?.message ?? String(e)
				});
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

	async function exportCode() {
		const code = await api.profile.export.code();
	}

	async function pickIcon(profileId: number) {
		const file = await openDialog({
			filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif', 'bmp'] }],
			multiple: false
		});

		if (file) {
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
	}

	function profileIconSrc(icon: string | null): string | null {
		if (!icon) return null;
		if (icon.startsWith('http')) return icon;
		return convertFileSrc(icon);
	}

	let draggingId: number | null = $state(null);
	let dragOverIdx: number = $state(-1);
	let ghostEl: HTMLDivElement | null = null;
	let dragOffsetX = 0;
	let dragOffsetY = 0;
	let dragFromIdx = -1;
	let dragStarted = false;
	let pointerStartX = 0;
	let pointerStartY = 0;
	let pendingCard: HTMLElement | null = null;
	let pendingProfileId: number | null = null;
	const DRAG_THRESHOLD = 5;

	function onPointerDown(e: PointerEvent, profile: { id: number }, idx: number) {
		const target = e.target as HTMLElement;
		if (target.closest('.z-profile-action, .z-profile-icon, input')) return;

		const card = target.closest('.z-profile-card') as HTMLElement;
		if (!card) return;

		dragFromIdx = idx;
		pendingProfileId = profile.id;
		pendingCard = card;
		pointerStartX = e.clientX;
		pointerStartY = e.clientY;
		dragStarted = false;

		const rect = card.getBoundingClientRect();
		dragOffsetX = e.clientX - rect.left;
		dragOffsetY = e.clientY - rect.top;

		window.addEventListener('pointermove', onPointerMove);
		window.addEventListener('pointerup', onPointerUp);
	}

	function startDrag(e: PointerEvent) {
		if (!pendingCard) return;
		dragStarted = true;
		draggingId = pendingProfileId;
		const rect = pendingCard.getBoundingClientRect();
		ghostEl = pendingCard.cloneNode(true) as HTMLDivElement;
		ghostEl.style.cssText = `
			position: fixed;
			left: ${rect.left}px;
			top: ${rect.top}px;
			width: ${rect.width}px;
			height: ${rect.height}px;
			pointer-events: none;
			z-index: 9999;
			opacity: 0.95;
			border: 2px solid var(--accent-400);
			box-shadow: var(--shadow-glow-strong);
			transform: scale(1.03);
			cursor: grabbing;
		`;
		document.body.appendChild(ghostEl);
		document.body.classList.add('dragging-active');
		onPointerMove(e);
	}

	function onPointerMove(e: PointerEvent) {
		if (!dragStarted) {
			const dx = e.clientX - pointerStartX;
			const dy = e.clientY - pointerStartY;
			if (dx * dx + dy * dy < DRAG_THRESHOLD * DRAG_THRESHOLD) return;
			startDrag(e);
			return;
		}
		if (!ghostEl) return;
		ghostEl.style.left = e.clientX - dragOffsetX + 'px';
		ghostEl.style.top = e.clientY - dragOffsetY + 'px';

		const cards = document.querySelectorAll<HTMLElement>('.z-profile-card[data-profile-idx]');
		let bestIdx = -1;
		let bestDist = Infinity;
		cards.forEach((el) => {
			const idx = parseInt(el.dataset.profileIdx!);
			if (idx === dragFromIdx) return;
			const r = el.getBoundingClientRect();
			const cx = r.left + r.width / 2;
			const cy = r.top + r.height / 2;
			const d = (e.clientX - cx) ** 2 + (e.clientY - cy) ** 2;
			if (d < bestDist) {
				bestDist = d;
				bestIdx = idx;
			}
		});
		dragOverIdx = bestIdx;
	}

	async function onPointerUp() {
		window.removeEventListener('pointermove', onPointerMove);
		window.removeEventListener('pointerup', onPointerUp);

		if (!dragStarted) {
			pendingCard = null;
			pendingProfileId = null;
			dragFromIdx = -1;
			return;
		}

		document.body.classList.remove('dragging-active');

		if (ghostEl) {
			ghostEl.remove();
			ghostEl = null;
		}

		const from = dragFromIdx;
		const to = dragOverIdx;
		draggingId = null;
		dragOverIdx = -1;
		dragFromIdx = -1;
		dragStarted = false;
		pendingCard = null;
		pendingProfileId = null;

		if (from === -1 || to === -1 || from === to) return;
		try {
			await api.profile.reorderProfile(from, to);
			await profiles.refresh();
		} catch (err) {
			console.error('Failed to reorder profile:', err);
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
				<div class="z-auth-pill">
					{#if auth.user.avatar}
						<img src={auth.user.avatar} alt={auth.user.displayName} />
					{:else}
						<Icon icon="mdi:account-circle" />
					{/if}
					<span>{auth.user.displayName}</span>
					<button
						class="z-auth-logout"
						onclick={async () => { await auth.logout(); }}
						title={(i18nState.locale && m.sync_logOut()) || ''}
					>
						<Icon icon="mdi:logout" />
					</button>
				</div>
			{:else}
				<Button
					variant="secondary"
					size="sm"
					onclick={async () => {
						try {
							await auth.login();
							pushToast({
								type: 'info',
								name: m.sync_signedIn(),
								message: m.sync_welcome({ name: auth.user?.displayName ?? '' })
							});
						} catch (e: any) {
							pushToast({
								type: 'error',
								name: m.sync_signInFailed(),
								message: e?.message ?? String(e)
							});
						}
					}}
				>
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
				<div
					class="z-profile-card"
					class:active={profile.id === profiles.activeId}
					class:dragging={draggingId === profile.id}
					class:drag-over={dragOverIdx === i && draggingId !== null && draggingId !== profile.id}
					data-profile-idx={i}
					onpointerdown={(e) => onPointerDown(e, profile, i)}
				>
					<button class="z-profile-select" onclick={() => selectProfile(profile.id)}>
						<div class="z-profile-header">
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div
								class="z-profile-icon"
								role="button"
								tabindex="-1"
								onclick={(e) => {
									e.stopPropagation();
									pickIcon(profile.id);
								}}
								onkeydown={() => {}}
							>
								{#if profile.icon}
									<img
										src={profileIconSrc(profile.icon)}
										alt={profile.name}
										class="z-profile-avatar"
									/>
								{:else}
									<Icon icon="mdi:account-circle" />
								{/if}
								<span class="z-profile-icon-overlay">
									<Icon icon="mdi:camera" />
								</span>
							</div>
							<div class="z-profile-info">
								<div class="z-profile-name-row">
									<span class="z-profile-name">{profile.name}</span>
									{#if profile.id === profiles.activeId}
										<span class="z-profile-active-dot"></span>
									{/if}
									{#if profile.sync}
										<Icon icon="mdi:cloud" class="z-profile-sync-icon" />
									{/if}
								</div>
								<span class="z-profile-mods"
									>{i18nState.locale &&
										m.profiles_mods({ count: profile.modCount.toString() })}</span
								>
							</div>
						</div>
					</button>

					<div class="z-profile-footer">
						<div class="z-profile-badges">
							{#if profile.id === profiles.activeId}
								<span class="z-profile-badge accent">{i18nState.locale && m.profiles_active()}</span
								>
							{/if}
							{#if profile.sync}
								<span class="z-profile-badge info">{i18nState.locale && m.profiles_synced()}</span>
							{/if}
						</div>

						<div class="z-profile-actions">
							{#if auth.user && !profile.sync}
								<Tooltip text={(i18nState.locale && m.sync_toCloud()) || ''} position="bottom" delay={200}>
									<button
										class="z-profile-action"
										onclick={async () => {
											try {
												if (profile.id !== profiles.activeId) {
													const idx = getProfileIndex(profile.id);
													if (idx !== -1) await api.profile.setActive(idx);
												}
												const id = await api.profile.sync.create();
												pushToast({
													type: 'info',
													name: m.sync_synced(),
													message: m.sync_uploaded({
														name: profile.name,
														id: id.slice(0, 8) + '…'
													})
												});
												await profiles.refresh();
											} catch (e: any) {
												pushToast({
													type: 'error',
													name: m.sync_syncFailed(),
													message: e?.message ?? String(e)
												});
											}
										}}
									>
										<Icon icon="mdi:cloud-upload" />
									</button>
								</Tooltip>
							{/if}
							{#if auth.user && profile.sync}
								<Tooltip text={(i18nState.locale && m.sync_stopSyncing()) || ''} position="bottom" delay={200}>
									<button
										class="z-profile-action"
										onclick={() => askDisconnect(profile.id, profile.name)}
									>
										<Icon icon="mdi:cloud-off-outline" />
									</button>
								</Tooltip>
							{/if}
							<Tooltip text={i18nState.locale && m.profiles_rename()} position="bottom" delay={200}>
								<button
									class="z-profile-action"
									onclick={() => startRename(profile.id, profile.name)}
								>
									<Icon icon="mdi:pencil" />
								</button>
							</Tooltip>
							<Tooltip
								text={i18nState.locale && m.profiles_duplicate()}
								position="bottom"
								delay={200}
							>
								<button
									class="z-profile-action"
									onclick={() => duplicateProfile(profile.id, profile.name)}
								>
									<Icon icon="mdi:content-copy" />
								</button>
							</Tooltip>
							{#if profiles.list.length > 1}
								<Tooltip
									text={i18nState.locale && m.profiles_delete()}
									position="bottom"
									delay={200}
								>
									<button class="z-profile-action danger" onclick={() => deleteProfile(profile.id)}>
										<Icon icon="mdi:delete" />
									</button>
								</Tooltip>
							{/if}
						</div>
					</div>
				</div>
			{/each}
		</div>
	</div>
</div>

<!-- Cloud restore dialog -->
<RestoreCloudDialog
	bind:open={restoreOpen}
	items={restoreItems}
	onclose={() => {
		restoreItems = [];
	}}
/>

<RestoreCrossGameDialog
	bind:open={crossGameOpen}
	buckets={crossGameBuckets}
	onclose={() => {
		crossGameBuckets = [];
	}}
	onPicked={() => {
		checkForRestorable(false);
	}}
/>

<!-- Create modal -->
<Modal bind:open={createOpen} title={i18nState.locale && m.profiles_newProfile()}>
	<div class="z-modal-form">
		<Input
			bind:value={newName}
			placeholder={i18nState.locale && m.profiles_profileName()}
			onkeydown={(e) => {
				if (e.key === 'Enter') createProfile();
			}}
		/>
	</div>

	{#snippet actions()}
		<Button variant="ghost" onclick={() => (createOpen = false)}
			>{i18nState.locale && m.dialog_cancel()}</Button
		>
		<Button variant="primary" onclick={createProfile} disabled={!newName.trim()}
			>{i18nState.locale && m.profiles_create()}</Button
		>
	{/snippet}
</Modal>

<!-- Disconnect / unsync modal -->
<Modal
	open={disconnectId !== null}
	onclose={() => {
		if (!disconnectBusy) disconnectId = null;
	}}
	title={(i18nState.locale && m.sync_unsyncTitle()) || ''}
>
	<div class="z-modal-form">
		<p class="z-unsync-text">
			{i18nState.locale && m.sync_unsyncText({ name: disconnectName })}
		</p>
		<div
			class="z-unsync-option"
			role="button"
			tabindex="0"
			onclick={() => {
				if (!disconnectBusy) disconnectAlsoDelete = !disconnectAlsoDelete;
			}}
			onkeydown={(e) => {
				if (e.key === ' ' || e.key === 'Enter') {
					e.preventDefault();
					if (!disconnectBusy) disconnectAlsoDelete = !disconnectAlsoDelete;
				}
			}}
		>
			<Checkbox bind:checked={disconnectAlsoDelete} disabled={disconnectBusy} />
			<span>
				{i18nState.locale && m.sync_unsyncAlsoDelete()}
				<span class="z-unsync-hint">
					{i18nState.locale && m.sync_unsyncAlsoDeleteHint()}
				</span>
			</span>
		</div>
	</div>

	{#snippet actions()}
		<Button variant="ghost" onclick={() => (disconnectId = null)} disabled={disconnectBusy}>
			{i18nState.locale && m.dialog_cancel()}
		</Button>
		<Button
			variant={disconnectAlsoDelete ? 'danger' : 'primary'}
			onclick={confirmDisconnect}
			loading={disconnectBusy}
			disabled={disconnectBusy}
		>
			{i18nState.locale &&
				(disconnectAlsoDelete ? m.sync_unsyncAndDeleteAction() : m.sync_unsyncAction())}
		</Button>
	{/snippet}
</Modal>

<!-- Rename modal -->
<Modal
	open={renameId !== null}
	onclose={() => (renameId = null)}
	title={i18nState.locale && m.profiles_renameProfile()}
>
	<div class="z-modal-form">
		<Input
			bind:value={renameName}
			placeholder={i18nState.locale && m.profiles_newName()}
			onkeydown={(e) => {
				if (e.key === 'Enter') confirmRename();
			}}
		/>
	</div>

	{#snippet actions()}
		<Button variant="ghost" onclick={() => (renameId = null)}
			>{i18nState.locale && m.dialog_cancel()}</Button
		>
		<Button variant="primary" onclick={confirmRename} disabled={!renameName.trim()}
			>{i18nState.locale && m.profiles_rename()}</Button
		>
	{/snippet}
</Modal>

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

	.z-auth-pill {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 4px 4px 4px 12px;
		border-radius: var(--radius-full);
		background: var(--bg-active);
		border: 1px solid var(--border-accent);
		font-size: 12px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.z-auth-pill img,
	.z-auth-pill :global(svg:first-child) {
		width: 20px;
		height: 20px;
		border-radius: 50%;
		object-fit: cover;
	}

	.z-auth-logout {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: none;
		border-radius: 50%;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.z-auth-logout:hover {
		background: var(--bg-hover);
		color: var(--error);
	}

	.z-profiles-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
		gap: var(--space-md);
	}

	.z-profile-card {
		border-radius: var(--radius-lg);
		background: var(--bg-surface);
		border: 1px solid var(--border-subtle);
		overflow: visible;
		transition:
			border-color var(--transition-fast),
			box-shadow var(--transition-fast);
		cursor: grab;
		touch-action: none;
		user-select: none;
	}

	.z-profile-card:active {
		cursor: grabbing;
	}

	.z-profile-card.dragging {
		opacity: 0.3;
	}

	.z-profile-card.drag-over {
		border-color: var(--accent-400);
		box-shadow: 0 0 0 2px var(--accent-400);
	}

	.z-profile-card:hover {
		border-color: var(--border-default);
	}

	.z-profile-card.active {
		border-color: var(--border-accent);
		box-shadow: var(--shadow-glow);
	}

	.z-profile-select {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		width: 100%;
		padding: var(--space-lg);
		padding-bottom: var(--space-sm);
		background: transparent;
		border: none;
		cursor: pointer;
		text-align: left;
		font-family: var(--font-body);
	}

	.z-profile-header {
		display: flex;
		align-items: center;
		gap: var(--space-md);
	}

	.z-profile-icon {
		width: 44px;
		height: 44px;
		border-radius: var(--radius-lg);
		background: var(--bg-overlay);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 22px;
		color: var(--text-muted);
		flex-shrink: 0;
		transition: all var(--transition-fast);
		position: relative;
		overflow: hidden;
		cursor: pointer;
		border: none;
	}

	.z-profile-icon:hover .z-profile-icon-overlay {
		opacity: 1;
	}

	.z-profile-icon-overlay {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		background: rgba(0, 0, 0, 0.55);
		color: white;
		font-size: 16px;
		opacity: 0;
		transition: opacity var(--transition-fast);
		border-radius: var(--radius-lg);
	}

	.z-profile-avatar {
		width: 100%;
		height: 100%;
		object-fit: contain;
		border-radius: var(--radius-lg);
		padding: 4px;
	}

	.z-profile-card.active .z-profile-icon {
		background: var(--bg-active);
		color: var(--text-accent);
	}

	.z-profile-info {
		display: flex;
		flex-direction: column;
		min-width: 0;
		flex: 1;
	}

	.z-profile-name-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.z-profile-name {
		font-weight: 600;
		font-size: 14px;
		color: var(--text-primary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.z-profile-active-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--accent-400);
		box-shadow: 0 0 6px var(--accent-400);
		flex-shrink: 0;
	}

	:global(.z-profile-sync-icon) {
		font-size: 12px;
		color: var(--info);
		flex-shrink: 0;
	}

	.z-profile-mods {
		font-size: 12px;
		color: var(--text-muted);
		margin-top: 1px;
	}

	.z-profile-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-xs) var(--space-md) var(--space-md);
	}

	.z-profile-badges {
		display: flex;
		gap: 4px;
	}

	.z-profile-badge {
		font-size: 10px;
		font-weight: 600;
		padding: 2px 8px;
		border-radius: var(--radius-full);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.z-profile-badge.accent {
		background: var(--bg-active);
		color: var(--accent-400);
	}

	.z-profile-badge.info {
		background: rgba(59, 130, 246, 0.1);
		color: var(--info);
	}

	.z-profile-actions {
		display: flex;
		gap: 2px;
	}

	.z-profile-action {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
		font-size: 14px;
	}

	.z-profile-action:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.z-profile-action.danger:hover {
		background: rgba(255, 92, 92, 0.1);
		color: var(--error);
	}

	.z-modal-form {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.z-unsync-text {
		font-size: 13px;
		color: var(--text-secondary);
		margin: 0;
	}

	.z-unsync-text strong {
		color: var(--text-primary);
	}

	.z-unsync-option {
		display: flex;
		align-items: flex-start;
		gap: var(--space-sm);
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		border: 1px solid var(--border-subtle);
		cursor: pointer;
		font-size: 13px;
	}

	.z-unsync-option:hover {
		border-color: var(--border-default);
	}

	.z-unsync-hint {
		display: block;
		font-size: 11px;
		color: var(--text-muted);
		margin-top: 2px;
		line-height: 1.4;
	}
</style>
