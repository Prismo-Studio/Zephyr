<script lang="ts">
	import Icon from '@iconify/svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import type { PlayerFile } from './types';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	let {
		players,
		initialLoading,
		onRenamePlayer,
		onDeletePlayer
	}: {
		players: PlayerFile[];
		initialLoading: boolean;
		onRenamePlayer: (p: PlayerFile) => void;
		onDeletePlayer: (p: PlayerFile) => void;
	} = $props();
</script>

<div class="rdz-block-body">
	{#if initialLoading}
		<p class="rdz-muted rdz-loading-row">
			<Icon icon="mdi:loading" class="rdz-spin" />
			<span>{i18nState.locale && m.randomizer_loadingData()}</span>
		</p>
	{:else if players.length === 0}
		<p class="rdz-muted">
			{i18nState.locale && m.randomizer_noPlayerYamls()}
		</p>
	{:else}
		<ul class="rdz-player-list">
			{#each players as p (p.path)}
				<li>
					<span class="rdz-player-name">{p.name}</span>
					<span class="rdz-player-size">{Math.ceil(p.size / 102.4) / 10} KB</span>
					<div class="rdz-seed-actions">
						<Tooltip text={i18nState.locale && m.randomizer_rename()} position="top" delay={200}>
							<button class="rdz-icon-btn" onclick={() => onRenamePlayer(p)}>
								<Icon icon="mdi:pencil" />
							</button>
						</Tooltip>
						<Tooltip text={i18nState.locale && m.randomizer_delete()} position="top" delay={200}>
							<button class="rdz-icon-btn rdz-icon-danger" onclick={() => onDeletePlayer(p)}>
								<Icon icon="mdi:delete" />
							</button>
						</Tooltip>
					</div>
				</li>
			{/each}
		</ul>
	{/if}
</div>

<style>
	.rdz-block-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		padding: 0;
		animation: rdz-block-in 180ms ease;
	}

	@keyframes rdz-block-in {
		from {
			opacity: 0;
			transform: translateY(-2px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.rdz-muted {
		margin: 0;
		color: var(--text-muted);
		font-size: 12px;
	}

	.rdz-loading-row {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}

	.rdz-player-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.rdz-player-list li {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: 6px 10px;
		border-radius: var(--radius-sm);
		border-bottom: 1px solid var(--border-subtle);
		transition: background var(--transition-fast);
	}

	.rdz-player-list li:hover {
		background: var(--bg-hover);
	}

	.rdz-player-list li:last-child {
		border-bottom: none;
	}

	.rdz-player-list :global(svg) {
		font-size: 16px;
		color: var(--text-muted);
	}

	.rdz-player-name {
		flex: 1;
		font-size: 13px;
		color: var(--text-primary);
		font-weight: 600;
	}

	.rdz-player-size {
		font-size: 11px;
		color: var(--text-muted);
		font-family: var(--font-mono, monospace);
	}

	.rdz-seed-actions {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		margin-left: auto;
		flex-shrink: 0;
	}

	.rdz-icon-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition:
			color var(--transition-fast),
			background var(--transition-fast);
	}

	.rdz-icon-btn:hover {
		color: var(--text-primary);
		background: var(--bg-hover);
	}

	.rdz-icon-btn.rdz-icon-danger:hover {
		color: var(--error);
		background: transparent;
	}

	.rdz-icon-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.rdz-icon-btn:disabled:hover {
		color: var(--text-muted);
		background: transparent;
	}
</style>
