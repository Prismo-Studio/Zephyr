<script lang="ts">
	import Header from '$lib/components/layout/Header.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Icon from '@iconify/svelte';

	import profiles from '$lib/state/profile.svelte';
	import auth from '$lib/state/auth.svelte';
	import * as api from '$lib/api';

	let createOpen = $state(false);
	let newName = $state('');
	let renameId: number | null = $state(null);
	let renameName = $state('');

	async function createProfile() {
		if (!newName.trim()) return;
		await api.profile.create(newName.trim(), null);
		await profiles.refresh();
		newName = '';
		createOpen = false;
	}

	async function deleteProfile(id: number) {
		await api.profile.deleteProfile(id);
		await profiles.refresh();
	}

	async function duplicateProfile(name: string) {
		await api.profile.duplicate(name + ' (copy)');
		await profiles.refresh();
	}

	async function startRename(id: number, name: string) {
		renameId = id;
		renameName = name;
	}

	async function confirmRename() {
		if (!renameName.trim() || renameId === null) return;
		await api.profile.setActive(renameId);
		await api.profile.rename(renameName.trim());
		await profiles.refresh();
		renameId = null;
	}

	async function selectProfile(id: number) {
		await profiles.setActive(id);
	}

	async function exportCode() {
		const code = await api.profile.export.code();
		// Copy to clipboard would be done via Tauri plugin
	}
</script>

<div class="z-profiles-page">
	<Header title="Profiles" subtitle="{profiles.list.length} total">
		{#snippet actions()}
			<Button variant="primary" size="sm" onclick={() => (createOpen = true)}>
				{#snippet icon()}<Icon icon="mdi:plus" />{/snippet}
				New Profile
			</Button>
		{/snippet}
	</Header>

	<div class="z-profiles-content">
		<div class="z-profiles-grid">
			{#each profiles.list as profile (profile.id)}
				<div
					class="z-profile-card"
					class:active={profile.id === profiles.activeId}
				>
					<button class="z-profile-select" onclick={() => selectProfile(profile.id)}>
						<div class="z-profile-header">
							<div class="z-profile-icon">
								<Icon icon="mdi:account-circle" />
							</div>
							<div class="z-profile-info">
								<span class="z-profile-name">{profile.name}</span>
								<span class="z-profile-mods">{profile.modCount} mods</span>
							</div>
						</div>

						{#if profile.id === profiles.activeId}
							<Badge variant="accent">Active</Badge>
						{/if}

						{#if profile.sync}
							<div class="z-profile-sync">
								<Icon icon="mdi:cloud" class="text-xs" />
								<span>Synced</span>
							</div>
						{/if}
					</button>

					<div class="z-profile-actions">
						<button class="z-profile-action" onclick={() => startRename(profile.id, profile.name)} title="Rename">
							<Icon icon="mdi:pencil" />
						</button>
						<button class="z-profile-action" onclick={() => duplicateProfile(profile.name)} title="Duplicate">
							<Icon icon="mdi:content-copy" />
						</button>
						{#if profiles.list.length > 1}
							<button class="z-profile-action danger" onclick={() => deleteProfile(profile.id)} title="Delete">
								<Icon icon="mdi:delete" />
							</button>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	</div>
</div>

<!-- Create modal -->
<Modal bind:open={createOpen} title="New Profile">
	<div class="z-modal-form">
		<Input
			bind:value={newName}
			placeholder="Profile name"
			onkeydown={(e) => { if (e.key === 'Enter') createProfile(); }}
		/>
	</div>

	{#snippet actions()}
		<Button variant="ghost" onclick={() => (createOpen = false)}>Cancel</Button>
		<Button variant="primary" onclick={createProfile} disabled={!newName.trim()}>Create</Button>
	{/snippet}
</Modal>

<!-- Rename modal -->
<Modal open={renameId !== null} onclose={() => (renameId = null)} title="Rename Profile">
	<div class="z-modal-form">
		<Input
			bind:value={renameName}
			placeholder="New name"
			onkeydown={(e) => { if (e.key === 'Enter') confirmRename(); }}
		/>
	</div>

	{#snippet actions()}
		<Button variant="ghost" onclick={() => (renameId = null)}>Cancel</Button>
		<Button variant="primary" onclick={confirmRename} disabled={!renameName.trim()}>Rename</Button>
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
		padding: 0 var(--space-xl) var(--space-xl);
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
		overflow: hidden;
		transition: all var(--transition-fast);
	}

	.z-profile-card:hover {
		border-color: var(--border-default);
	}

	.z-profile-card.active {
		border-color: var(--border-accent);
		box-shadow: 0 0 16px rgba(26, 255, 250, 0.06);
	}

	.z-profile-select {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		width: 100%;
		padding: var(--space-lg);
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
		width: 40px;
		height: 40px;
		border-radius: var(--radius-md);
		background: var(--bg-overlay);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 20px;
		color: var(--text-muted);
		flex-shrink: 0;
	}

	.z-profile-card.active .z-profile-icon {
		background: var(--bg-active);
		color: var(--text-accent);
	}

	.z-profile-info {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}

	.z-profile-name {
		font-weight: 600;
		font-size: 14px;
		color: var(--text-primary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.z-profile-mods {
		font-size: 12px;
		color: var(--text-muted);
	}

	.z-profile-sync {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		color: var(--info);
	}

	.z-profile-actions {
		display: flex;
		gap: 2px;
		padding: 0 var(--space-md) var(--space-md);
	}

	.z-profile-action {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
		font-size: 16px;
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
