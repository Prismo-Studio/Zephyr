<script lang="ts">
	import Icon from '@iconify/svelte';
	import profiles from '$lib/state/profile.svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		onswitch: (id: number) => Promise<void>;
		onclose: () => void;
	};

	let { onswitch, onclose }: Props = $props();

	function profileIconSrc(icon: string | null): string | null {
		if (!icon) return null;
		if (icon.startsWith('http')) return icon;
		return convertFileSrc(icon);
	}
</script>

<div class="z-profile-dropdown" role="menu">
	<div class="z-profile-dropdown-header">
		<Icon icon="mdi:account-switch" />
		<span>{m.sidebar_switchProfile()}</span>
	</div>
	<div class="z-profile-list">
		{#each profiles.list as profile}
			<button
				class="z-profile-item"
				class:active={profile.id === profiles.activeId}
				onclick={() => onswitch(profile.id)}
				role="menuitem"
			>
				<div class="z-profile-item-icon">
					{#if profile.icon}
						<img src={profileIconSrc(profile.icon)} alt={profile.name} class="z-profile-item-img" />
					{:else}
						<Icon icon="mdi:account-circle" />
					{/if}
				</div>
				<span class="z-profile-item-name">{profile.name}</span>
				{#if profile.id === profiles.activeId}
					<Icon icon="mdi:check-circle" class="z-profile-check" />
					<span class="z-profile-active-badge">{m.sidebar_activeProfile()}</span>
				{/if}
			</button>
		{/each}
	</div>
</div>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="z-profile-backdrop" onclick={onclose}></div>

<style>
	.z-profile-backdrop {
		position: fixed;
		inset: 0;
		z-index: calc(var(--z-dropdown) - 1);
	}

	.z-profile-dropdown {
		position: absolute;
		bottom: 0;
		left: 72px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		padding: var(--space-xs);
		min-width: 200px;
		max-height: 320px;
		overflow-y: auto;
		z-index: var(--z-dropdown);
		box-shadow: var(--shadow-lg), var(--shadow-glow);
		animation: slideUp 150ms ease;
	}

	@keyframes slideUp {
		from {
			opacity: 0;
			transform: translateY(6px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.z-profile-dropdown-header {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		padding: var(--space-xs) var(--space-sm);
		color: var(--text-muted);
		font-size: 10px;
		font-weight: 700;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		border-bottom: 1px solid var(--border-subtle);
		padding-bottom: var(--space-sm);
		margin-bottom: var(--space-xs);
	}

	.z-profile-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
		max-height: 240px;
		overflow-y: auto;
	}

	.z-profile-item {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		width: 100%;
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-md);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		transition: all var(--transition-fast);
		font-family: var(--font-body);
		font-size: 13px;
		text-align: left;
	}

	.z-profile-item:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.z-profile-item.active {
		background: var(--bg-active);
		color: var(--accent-400);
	}

	.z-profile-item-icon {
		width: 24px;
		height: 24px;
		font-size: 24px;
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-muted);
		overflow: hidden;
		border-radius: var(--radius-full);
	}

	.z-profile-item.active .z-profile-item-icon {
		color: var(--accent-400);
	}

	.z-profile-item-img {
		width: 24px;
		height: 24px;
		border-radius: var(--radius-full);
		object-fit: cover;
	}

	:global(.z-profile-check) {
		color: var(--accent-400);
	}

	.z-profile-item-name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.z-profile-active-badge {
		font-size: 9px;
		font-weight: 700;
		letter-spacing: 0.05em;
		text-transform: uppercase;
		color: var(--accent-400);
		background: var(--bg-active);
		padding: 2px 6px;
		border-radius: var(--radius-full);
		flex-shrink: 0;
	}
</style>
