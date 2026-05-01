<script lang="ts">
	import Icon from '@iconify/svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import profiles from '$lib/state/profile.svelte';
	import auth from '$lib/state/auth.svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Profile = (typeof profiles.list)[number];

	type Props = {
		profile: Profile;
		index: number;
		isActive: boolean;
		isDragging: boolean;
		isDragOver: boolean;
		onpointerdown: (e: PointerEvent, profile: Profile, idx: number) => void;
		onselect: (id: number) => void;
		onpickIcon: (id: number) => void;
		onsyncToCloud: (profile: Profile) => void;
		onaskDisconnect: (id: number, name: string) => void;
		onstartRename: (id: number, name: string) => void;
		onduplicate: (id: number, name: string) => void;
		ondelete: (id: number) => void;
	};

	let {
		profile,
		index,
		isActive,
		isDragging,
		isDragOver,
		onpointerdown,
		onselect,
		onpickIcon,
		onsyncToCloud,
		onaskDisconnect,
		onstartRename,
		onduplicate,
		ondelete
	}: Props = $props();

	function profileIconSrc(icon: string | null): string | null {
		if (!icon) return null;
		if (icon.startsWith('http')) return icon;
		return convertFileSrc(icon);
	}

	let canDelete = $derived(profiles.list.length > 1);
</script>

<div
	class="z-profile-card"
	class:active={isActive}
	class:dragging={isDragging}
	class:drag-over={isDragOver}
	data-profile-idx={index}
	onpointerdown={(e) => onpointerdown(e, profile, index)}
>
	<button class="z-profile-select" onclick={() => onselect(profile.id)}>
		<div class="z-profile-header">
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="z-profile-icon"
				role="button"
				tabindex="-1"
				onclick={(e) => {
					e.stopPropagation();
					onpickIcon(profile.id);
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
					{#if isActive}
						<span class="z-profile-active-dot"></span>
					{/if}
					{#if profile.sync}
						<Icon icon="mdi:cloud" class="z-profile-sync-icon" />
					{/if}
				</div>
				<span class="z-profile-mods">
					{i18nState.locale && m.profiles_mods({ count: profile.modCount.toString() })}
				</span>
			</div>
		</div>
	</button>

	<div class="z-profile-footer">
		<div class="z-profile-badges">
			{#if isActive}
				<span class="z-profile-badge accent">{i18nState.locale && m.profiles_active()}</span>
			{/if}
			{#if profile.sync}
				<span class="z-profile-badge info">{i18nState.locale && m.profiles_synced()}</span>
			{/if}
		</div>

		<div class="z-profile-actions">
			{#if auth.user && !profile.sync}
				<Tooltip text={(i18nState.locale && m.sync_toCloud()) || ''} position="bottom" delay={200}>
					<button class="z-profile-action" onclick={() => onsyncToCloud(profile)}>
						<Icon icon="mdi:cloud-upload" />
					</button>
				</Tooltip>
			{/if}
			{#if auth.user && profile.sync}
				<Tooltip
					text={(i18nState.locale && m.sync_stopSyncing()) || ''}
					position="bottom"
					delay={200}
				>
					<button
						class="z-profile-action"
						onclick={() => onaskDisconnect(profile.id, profile.name)}
					>
						<Icon icon="mdi:cloud-off-outline" />
					</button>
				</Tooltip>
			{/if}
			<Tooltip text={i18nState.locale && m.profiles_rename()} position="bottom" delay={200}>
				<button class="z-profile-action" onclick={() => onstartRename(profile.id, profile.name)}>
					<Icon icon="mdi:pencil" />
				</button>
			</Tooltip>
			<Tooltip text={i18nState.locale && m.profiles_duplicate()} position="bottom" delay={200}>
				<button class="z-profile-action" onclick={() => onduplicate(profile.id, profile.name)}>
					<Icon icon="mdi:content-copy" />
				</button>
			</Tooltip>
			{#if canDelete}
				<Tooltip text={i18nState.locale && m.profiles_delete()} position="bottom" delay={200}>
					<button class="z-profile-action danger" onclick={() => ondelete(profile.id)}>
						<Icon icon="mdi:delete" />
					</button>
				</Tooltip>
			{/if}
		</div>
	</div>
</div>

<style>
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
</style>
