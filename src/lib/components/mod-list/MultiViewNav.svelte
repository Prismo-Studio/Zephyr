<script lang="ts">
	import Icon from '@iconify/svelte';

	type Props = {
		index: number;
		total: number;
		onprev: () => void;
		onnext: () => void;
	};

	let { index, total, onprev, onnext }: Props = $props();
</script>

<div class="z-multi-nav">
	<button class="z-multi-nav-btn" disabled={index <= 0} onclick={onprev}>
		<Icon icon="mdi:chevron-left" />
	</button>
	<span class="z-multi-nav-label">{index + 1} / {total}</span>
	<button class="z-multi-nav-btn" disabled={index >= total - 1} onclick={onnext}>
		<Icon icon="mdi:chevron-right" />
	</button>
</div>

<style>
	.z-multi-nav {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-sm);
		padding: 8px 0;
	}

	.z-multi-nav-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		color: var(--text-secondary);
		cursor: pointer;
		transition: all var(--transition-fast);
		font-size: 18px;
	}

	.z-multi-nav-btn:hover:not(:disabled) {
		background: var(--bg-hover);
		color: var(--text-primary);
		border-color: var(--border-strong);
	}

	.z-multi-nav-btn:disabled {
		opacity: 0.35;
		cursor: not-allowed;
	}

	.z-multi-nav-label {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-secondary);
		min-width: 50px;
		text-align: center;
		font-family: var(--font-body);
	}
</style>
