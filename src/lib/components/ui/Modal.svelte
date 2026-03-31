<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		open?: boolean;
		onclose?: () => void;
		title?: string;
		class?: string;
		children?: Snippet;
		actions?: Snippet;
	};

	let { open = $bindable(false), onclose, title = '', class: className = '', children, actions }: Props = $props();

	function handleBackdrop() {
		open = false;
		onclose?.();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') handleBackdrop();
	}
</script>

<svelte:window onkeydown={open ? handleKeydown : undefined} />

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="z-modal-backdrop" onclick={handleBackdrop} onkeydown={() => {}}>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="z-modal {className}" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
			{#if title}
				<div class="z-modal-header">
					<h3>{title}</h3>
					<button class="z-modal-close" onclick={handleBackdrop}>
						<svg width="14" height="14" viewBox="0 0 14 14" fill="none">
							<path d="M1 1L13 13M13 1L1 13" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
						</svg>
					</button>
				</div>
			{/if}

			<div class="z-modal-body">
				{#if children}{@render children()}{/if}
			</div>

			{#if actions}
				<div class="z-modal-actions">
					{@render actions()}
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	.z-modal-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		backdrop-filter: blur(4px);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: var(--z-modal);
		animation: fadeIn var(--transition-fast) ease;
	}

	.z-modal {
		background: var(--bg-surface);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
		box-shadow: var(--shadow-lg);
		min-width: 400px;
		max-width: 560px;
		max-height: 80vh;
		display: flex;
		flex-direction: column;
		animation: scaleIn var(--transition-normal) ease;
	}

	.z-modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-lg) var(--space-xl);
		border-bottom: 1px solid var(--border-subtle);
	}

	.z-modal-header h3 {
		font-family: var(--font-display);
		font-size: 16px;
		font-weight: 700;
		color: var(--text-primary);
	}

	.z-modal-close {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border-radius: var(--radius-sm);
		background: transparent;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.z-modal-close:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.z-modal-body {
		padding: var(--space-xl);
		overflow-y: auto;
		flex: 1;
	}

	.z-modal-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-sm);
		padding: var(--space-lg) var(--space-xl);
		border-top: 1px solid var(--border-subtle);
	}
</style>
