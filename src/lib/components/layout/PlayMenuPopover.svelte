<script lang="ts">
	import games from '$lib/state/game.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { gameIconSrc } from '$lib/util';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		bottom: number;
		left: number;
		onlaunchModded: () => void;
		onlaunchVanilla: () => void;
	};

	let { bottom, left, onlaunchModded, onlaunchVanilla }: Props = $props();
</script>

<div
	id="z-play-menu"
	class="z-play-menu"
	role="menu"
	style="bottom: {bottom}px; left: {left}px;"
>
	{#if games.active || profiles.active}
		<div class="z-play-menu-header">
			{#if games.active}
				<img
					src={gameIconSrc(games.active)}
					alt=""
					aria-hidden="true"
					class="z-play-menu-game-icon"
				/>
			{/if}
			<div class="z-play-menu-header-text">
				{#if games.active}
					<span class="z-play-menu-game">{games.active.name}</span>
				{/if}
				{#if profiles.active}
					<span class="z-play-menu-profile">{profiles.active.name}</span>
				{/if}
			</div>
		</div>
	{/if}

	<button class="z-play-menu-item primary" role="menuitem" onclick={onlaunchModded}>
		<span class="z-play-menu-body">
			<span class="z-play-menu-title">
				{i18nState.locale && m.sidebar_playModded_title()}
			</span>
			<span class="z-play-menu-sub">
				{#if profiles.active}
					{i18nState.locale &&
						m.sidebar_playModded_subCount({ count: String(profiles.active.modCount) })}
				{:else}
					{i18nState.locale && m.sidebar_playModded_subFallback()}
				{/if}
			</span>
		</span>
		<svg
			class="z-play-menu-arrow"
			width="14"
			height="14"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2.5"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<path d="M9 6l6 6-6 6" />
		</svg>
	</button>

	<button class="z-play-menu-item" role="menuitem" onclick={onlaunchVanilla}>
		<span class="z-play-menu-body">
			<span class="z-play-menu-title">
				{i18nState.locale && m.sidebar_playVanilla_title()}
			</span>
			<span class="z-play-menu-sub">
				{i18nState.locale && m.sidebar_playVanilla_sub()}
			</span>
		</span>
		<svg
			class="z-play-menu-arrow"
			width="14"
			height="14"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2.5"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<path d="M9 6l6 6-6 6" />
		</svg>
	</button>
</div>

<style>
	.z-play-menu {
		position: fixed;
		min-width: 240px;
		max-width: 280px;
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 6px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		box-shadow:
			0 12px 32px rgba(0, 0, 0, 0.5),
			0 2px 8px rgba(0, 0, 0, 0.3);
		z-index: var(--z-dropdown, 9999);
		animation: z-play-menu-in 160ms cubic-bezier(0.2, 0, 0, 1);
	}

	@keyframes z-play-menu-in {
		from {
			opacity: 0;
			transform: translateX(-6px) scale(0.97);
		}
		to {
			opacity: 1;
			transform: translateX(0) scale(1);
		}
	}

	.z-play-menu-header {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 10px;
		margin-bottom: 4px;
		border-bottom: 1px solid var(--border-subtle);
	}

	.z-play-menu-game-icon {
		width: 28px;
		height: 28px;
		border-radius: var(--radius-sm);
		object-fit: cover;
		flex-shrink: 0;
	}

	.z-play-menu-header-text {
		display: flex;
		flex-direction: column;
		min-width: 0;
		flex: 1 1 auto;
		gap: 1px;
	}

	.z-play-menu-game {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.z-play-menu-profile {
		font-size: 11px;
		color: var(--text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.z-play-menu-item {
		position: relative;
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 12px 10px 14px;
		border: none;
		border-radius: var(--radius-md);
		background: transparent;
		color: var(--text-secondary);
		font-family: var(--font-body);
		cursor: pointer;
		text-align: left;
		transition:
			background var(--transition-fast),
			color var(--transition-fast);
	}

	.z-play-menu-item::before {
		content: '';
		position: absolute;
		left: 4px;
		top: 50%;
		width: 2px;
		height: 0;
		border-radius: var(--radius-full);
		background: var(--text-muted);
		transform: translateY(-50%);
		transition:
			height var(--transition-fast),
			background var(--transition-fast);
	}

	.z-play-menu-item.primary::before {
		background: var(--accent-400);
		height: 18px;
	}

	.z-play-menu-item:hover::before {
		height: 24px;
		background: var(--accent-300);
	}

	.z-play-menu-item:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.z-play-menu-arrow {
		flex-shrink: 0;
		color: var(--text-muted);
		opacity: 0;
		transform: translateX(-4px);
		transition:
			opacity var(--transition-fast),
			transform var(--transition-fast);
	}

	.z-play-menu-item:hover .z-play-menu-arrow {
		opacity: 1;
		transform: translateX(0);
	}

	.z-play-menu-item.primary .z-play-menu-arrow {
		color: var(--accent-400);
		opacity: 0.6;
		transform: translateX(0);
	}

	.z-play-menu-item.primary:hover .z-play-menu-arrow {
		opacity: 1;
	}

	.z-play-menu-body {
		display: flex;
		flex-direction: column;
		min-width: 0;
		flex: 1 1 auto;
		gap: 1px;
	}

	.z-play-menu-title,
	.z-play-menu-sub {
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.z-play-menu-title {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.z-play-menu-sub {
		font-size: 11px;
		color: var(--text-muted);
	}

	.z-play-menu-item.primary .z-play-menu-title {
		color: var(--accent-400);
	}
</style>
