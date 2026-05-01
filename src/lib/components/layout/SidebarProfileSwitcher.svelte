<script lang="ts">
	import Icon from '@iconify/svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import ProfileSwitchPopover from './ProfileSwitchPopover.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	let menuOpen = $state(false);

	function profileIconSrc(icon: string | null): string | null {
		if (!icon) return null;
		if (icon.startsWith('http')) return icon;
		return convertFileSrc(icon);
	}

	async function switchProfile(id: number) {
		if (id === profiles.activeId) return;
		const index = profiles.list.findIndex((p) => p.id === id);
		if (index === -1) return;
		await profiles.setActive(index);
		await profiles.refresh();
		menuOpen = false;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') menuOpen = false;
	}
</script>

{#if profiles.active}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="z-profile-wrapper" onkeydown={handleKeydown}>
		<Tooltip text={profiles.active.name} position="right" delay={300}>
			<button
				class="z-sidebar-profile"
				class:open={menuOpen}
				onclick={() => (menuOpen = !menuOpen)}
				aria-label={i18nState.locale && m.sidebar_switchProfile()}
				aria-expanded={menuOpen}
			>
				{#if profiles.active.icon}
					<img
						src={profileIconSrc(profiles.active.icon)}
						alt={profiles.active.name}
						class="z-profile-img"
					/>
				{:else}
					<Icon icon="mdi:account-circle" class="z-profile-avatar-icon" />
				{/if}
			</button>
		</Tooltip>

		{#if menuOpen}
			<ProfileSwitchPopover onswitch={switchProfile} onclose={() => (menuOpen = false)} />
		{/if}
	</div>
{/if}

<style>
	.z-profile-wrapper {
		position: relative;
		display: flex;
		justify-content: center;
		width: 100%;
	}

	.z-sidebar-profile {
		width: 40px;
		height: 40px;
		border-radius: var(--radius-lg);
		border: 2px solid var(--border-default);
		background: var(--bg-elevated);
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all var(--transition-fast);
		color: var(--text-muted);
		padding: 0;
		overflow: hidden;
	}

	.z-sidebar-profile:hover {
		border-color: var(--accent-400);
		box-shadow: var(--shadow-glow);
		color: var(--text-secondary);
	}

	.z-sidebar-profile.open {
		border-color: var(--accent-400);
		box-shadow: var(--shadow-glow);
		color: var(--accent-400);
	}

	:global(.z-profile-avatar-icon) {
		font-size: 28px;
	}

	.z-profile-img {
		width: 32px;
		height: 32px;
		object-fit: cover;
		border-radius: calc(var(--radius-lg) - 2px);
	}
</style>
