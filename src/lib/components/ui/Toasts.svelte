<script lang="ts">
	import { toasts, clearToast, type Toast } from '$lib/toast';
	import Icon from '@iconify/svelte';

	function toastIcon(type: Toast['type']): string {
		return type === 'error' ? 'mdi:alert-circle' : 'mdi:check-circle';
	}
</script>

<div class="z-toast-container">
	{#each $toasts as toast, i (toast.id)}
		<div class="z-toast z-toast-{toast.type}" role="alert">
			<Icon icon={toastIcon(toast.type)} class="z-toast-icon" />
			<div class="z-toast-content">
				{#if toast.name}
					<div class="z-toast-title">{toast.name}</div>
				{/if}
				<div class="z-toast-message">{toast.message}</div>
			</div>
			<button class="z-toast-close" onclick={() => clearToast(i)}>
				<Icon icon="mdi:close" />
			</button>
		</div>
	{/each}
</div>

<style>
	.z-toast-container {
		position: fixed;
		bottom: var(--space-xl);
		right: var(--space-xl);
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		z-index: var(--z-toast);
		max-width: 400px;
	}

	.z-toast {
		display: flex;
		align-items: flex-start;
		gap: var(--space-md);
		padding: var(--space-md) var(--space-lg);
		border-radius: var(--radius-lg);
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		box-shadow: var(--shadow-lg);
		animation: slideUp var(--transition-normal) ease;
		min-width: 300px;
	}

	.z-toast-error {
		border-color: rgba(255, 92, 92, 0.3);
		background: linear-gradient(135deg, var(--bg-elevated), rgba(255, 92, 92, 0.05));
	}

	.z-toast-info {
		border-color: rgba(26, 255, 250, 0.2);
		background: linear-gradient(135deg, var(--bg-elevated), rgba(26, 255, 250, 0.03));
	}

	:global(.z-toast-icon) {
		flex-shrink: 0;
		font-size: 18px;
		margin-top: 1px;
	}

	.z-toast-error :global(.z-toast-icon) { color: var(--error); }
	.z-toast-info :global(.z-toast-icon) { color: var(--accent-400); }

	.z-toast-content {
		flex: 1;
		min-width: 0;
	}

	.z-toast-title {
		font-weight: 600;
		font-size: 13px;
		color: var(--text-primary);
		margin-bottom: 2px;
	}

	.z-toast-message {
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.4;
		word-break: break-word;
	}

	.z-toast-close {
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		width: 20px;
		height: 20px;
		background: transparent;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		border-radius: var(--radius-sm);
		transition: all var(--transition-fast);
	}

	.z-toast-close:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}
</style>
