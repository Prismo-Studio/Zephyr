<script lang="ts">
	import type { Snippet } from 'svelte';
	import Icon from '@iconify/svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		count: number;
		total?: number;
		onclear: () => void;
		onselectAll?: () => void;
		actions: Snippet;
		legendActive?: boolean;
	};

	let { count, total, onclear, onselectAll, actions, legendActive = false }: Props = $props();
</script>

{#if count >= 2}
	<div class="z-batch-bar-container{legendActive ? ' z-batch-bar--with-legend' : ''}">
		<div class="z-batch-bar shadow-xl">
			<div class="z-batch-info">
				<button class="z-batch-clear" onclick={onclear} aria-label="Clear selection">
					<Icon icon="mdi:close" />
				</button>
				<span class="z-batch-count">
					{i18nState.locale &&
						(total
							? m.batch_totalSelected({ count: count.toString(), total: total.toString() })
							: m.batch_selected({ count: count.toString() }))}
				</span>
				{#if onselectAll && total && count < total}
					<button class="z-batch-select-all" onclick={onselectAll}>
						{i18nState.locale && m.batch_selectAll()}
					</button>
				{/if}
			</div>

			<div class="z-batch-actions">
				{@render actions()}
			</div>
		</div>
	</div>
{/if}

<style>
	.z-batch-bar-container {
		position: fixed;
		left: 0;
		right: 0;
		display: flex;
		justify-content: center;
		z-index: calc(var(--z-sticky, 100) + 1);
		pointer-events: none;
		animation: slideUp 0.3s cubic-bezier(0.16, 1, 0.3, 1);
		margin-bottom: 0;
		bottom: var(--space-xl);
	}
	.z-batch-bar--with-legend {
		bottom: calc(36px + var(--space-md));
	}

	.z-batch-bar {
		background: var(--bg-elevated);
		border: 1px solid var(--border-accent);
		border-radius: var(--radius-full);
		padding: var(--space-xs) var(--space-xs) var(--space-xs) var(--space-lg);
		display: flex;
		align-items: center;
		gap: var(--space-xl);
		pointer-events: auto;
		margin-bottom: var(--space-md);
		box-shadow: 0 0 0 1px var(--border-accent);
	}

	.z-batch-info {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
	}

	.z-batch-count {
		font-weight: 600;
		color: var(--text-primary);
		font-size: 14px;
	}

	.z-batch-clear {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: 50%;
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.z-batch-clear:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.z-batch-select-all {
		background: transparent;
		border: none;
		color: var(--text-accent);
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		padding: var(--space-xs) var(--space-sm);
		border-radius: var(--radius-sm);
		transition: all var(--transition-fast);
	}

	.z-batch-select-all:hover {
		background: var(--bg-active);
		text-decoration: underline;
	}

	.z-batch-actions {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
	}

	.z-batch-actions :global(.z-btn) {
		border-radius: var(--radius-full);
	}

	@keyframes slideUp {
		from {
			opacity: 0;
			transform: translateY(20px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
</style>
