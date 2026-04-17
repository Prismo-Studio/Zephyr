<script lang="ts">
	import Header from '$lib/components/layout/Header.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Icon from '@iconify/svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';

	import profiles from '$lib/state/profile.svelte';
	import auth from '$lib/state/auth.svelte';
	import * as api from '$lib/api';
	import { m } from '$lib/paraglide/messages';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { pushToast } from '$lib/toast';

	const MAX_PROFILES = 20;

	let createOpen = $state(false);
	let newName = $state('');
	let renameId: number | null = $state(null);
	let renameName = $state('');

	function getProfileIndex(id: number): number {
		return profiles.list.findIndex((p) => p.id === id);
	}

	function checkProfileLimit(): boolean {
		if (profiles.list.length >= MAX_PROFILES) {
			pushToast({
				type: 'error',
				name: 'Limite atteinte',
				message: `Vous ne pouvez pas avoir plus de ${MAX_PROFILES} profils.`
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
		newName = '';
		createOpen = false;
	}

	async function deleteProfile(id: number) {
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
	}

	async function startRename(id: number, name: string) {
		renameId = id;
		renameName = name;
	}

	async function confirmRename() {
		if (!renameName.trim() || renameId === null) return;
		const index = getProfileIndex(renameId);
		if (index === -1) return;
		const previousActiveIndex =
			profiles.activeId !== null ? getProfileIndex(profiles.activeId) : -1;
		const needsSwitch = renameId !== profiles.activeId;
		if (needsSwitch) {
			await api.profile.setActive(index);
		}
		await api.profile.rename(renameName.trim());
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
</style>
