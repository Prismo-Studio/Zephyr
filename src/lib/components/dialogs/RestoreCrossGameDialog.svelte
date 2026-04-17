<script lang="ts">
	import Modal from '$lib/components/ui/Modal.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Icon from '@iconify/svelte';
	import games from '$lib/state/game.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { gameIconSrc } from '$lib/util';
	import type { Game } from '$lib/types';

	type GameBucket = {
		slug: string;
		name: string;
		game: Game | null;
		count: number;
	};

	type Props = {
		open: boolean;
		buckets: GameBucket[];
		onclose: () => void;
		onPicked: (slug: string) => void;
	};

	let { open = $bindable(), buckets, onclose, onPicked }: Props = $props();

	let busy = $state<string | null>(null);

	function close() {
		if (busy) return;
		open = false;
		onclose();
	}

	async function pick(slug: string) {
		if (busy) return;
		busy = slug;
		try {
			await games.setActive(slug);
			await profiles.refresh();
			open = false;
			onclose();
			onPicked(slug);
		} finally {
			busy = null;
		}
	}
</script>

<Modal bind:open onclose={close} title={(i18nState.locale && m.restore_crossGameTitle()) || ''}>
	<div class="z-cross">
		<p class="z-cross-hint">
			{i18nState.locale && m.restore_crossGameDescription()}
		</p>

		<div class="z-cross-list">
			{#each buckets as bucket (bucket.slug)}
				<button class="z-cross-item" disabled={busy !== null} onclick={() => pick(bucket.slug)}>
					{#if bucket.game}
						<img src={gameIconSrc(bucket.game)} alt={bucket.name} class="z-cross-item-img" />
					{:else}
						<div class="z-cross-item-img placeholder">
							<Icon icon="mdi:gamepad-variant" />
						</div>
					{/if}
					<div class="z-cross-item-body">
						<div class="z-cross-item-name">{bucket.name}</div>
						<div class="z-cross-item-count">
							{i18nState.locale && m.restore_crossGameCount({ count: bucket.count.toString() })}
						</div>
					</div>
					{#if busy === bucket.slug}
						<Icon icon="mdi:loading" class="z-cross-item-arrow spin" />
					{:else}
						<Icon icon="mdi:chevron-right" class="z-cross-item-arrow" />
					{/if}
				</button>
			{/each}
		</div>
	</div>

	{#snippet actions()}
		<Button variant="ghost" onclick={close} disabled={busy !== null}>
			{i18nState.locale && m.restore_skip()}
		</Button>
	{/snippet}
</Modal>

<style>
	.z-cross {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		min-width: 420px;
		max-width: 560px;
	}

	.z-cross-hint {
		font-size: 13px;
		color: var(--text-secondary);
		margin: 0;
	}

	.z-cross-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
		max-height: 320px;
		overflow-y: auto;
		padding-right: 4px;
	}

	.z-cross-item {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding: 10px 12px;
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		border: 1px solid var(--border-subtle);
		cursor: pointer;
		font-family: var(--font-body);
		text-align: left;
		transition: all var(--transition-fast);
	}

	.z-cross-item:hover:not(:disabled) {
		border-color: var(--border-accent);
		background: var(--bg-elevated);
	}

	.z-cross-item:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.z-cross-item-img {
		width: 40px;
		height: 40px;
		border-radius: var(--radius-md);
		object-fit: cover;
		flex-shrink: 0;
		background: var(--bg-overlay);
	}

	.z-cross-item-img.placeholder {
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-muted);
		font-size: 20px;
	}

	.z-cross-item-body {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
		flex: 1;
	}

	.z-cross-item-name {
		font-weight: 600;
		font-size: 13px;
		color: var(--text-primary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.z-cross-item-count {
		font-size: 11px;
		color: var(--text-muted);
	}

	:global(.z-cross-item-arrow) {
		color: var(--text-muted);
		font-size: 18px;
		flex-shrink: 0;
	}

	:global(.z-cross-item-arrow.spin) {
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
