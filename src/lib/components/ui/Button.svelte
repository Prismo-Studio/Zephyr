<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { MouseEventHandler } from 'svelte/elements';

	type Variant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'accent';
	type Size = 'sm' | 'md' | 'lg';

	type Props = {
		variant?: Variant;
		size?: Size;
		disabled?: boolean;
		loading?: boolean;
		class?: string;
		onclick?: MouseEventHandler<HTMLButtonElement>;
		children?: Snippet;
		icon?: Snippet;
	};

	let {
		variant = 'secondary',
		size = 'md',
		disabled = false,
		loading = false,
		class: className = '',
		onclick,
		children,
		icon
	}: Props = $props();
</script>

<button
	class="z-btn z-btn-{variant} z-btn-{size} {className}"
	disabled={disabled || loading}
	{onclick}
>
	{#if loading}
		<span class="z-btn-spinner"></span>
	{:else if icon}
		<span class="z-btn-icon">{@render icon()}</span>
	{/if}
	{#if children}
		<span class="z-btn-label">{@render children()}</span>
	{/if}
</button>

<style>
	.z-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		border: 1px solid transparent;
		border-radius: var(--radius-md);
		font-family: var(--font-body);
		font-weight: 600;
		cursor: pointer;
		transition: all var(--transition-fast);
		white-space: nowrap;
		position: relative;
		overflow: hidden;
	}

	.z-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	/* Sizes */
	.z-btn-sm { padding: 6px 12px; font-size: 12px; border-radius: var(--radius-sm); }
	.z-btn-md { padding: 8px 16px; font-size: 13px; }
	.z-btn-lg { padding: 12px 24px; font-size: 15px; border-radius: var(--radius-lg); }

	/* Primary */
	.z-btn-primary {
		background: linear-gradient(135deg, var(--accent-400), var(--accent-600));
		color: var(--text-inverse);
		border-color: var(--accent-400);
		box-shadow: 0 0 12px rgba(26, 255, 250, 0.15);
	}
	.z-btn-primary:hover:not(:disabled) {
		box-shadow: 0 0 24px rgba(26, 255, 250, 0.3);
		transform: translateY(-1px);
	}
	.z-btn-primary:active:not(:disabled) {
		transform: translateY(0);
	}

	/* Accent */
	.z-btn-accent {
		background: var(--accent-400);
		color: var(--text-inverse);
		font-weight: 700;
	}
	.z-btn-accent:hover:not(:disabled) {
		background: var(--accent-300);
		box-shadow: var(--shadow-glow);
	}

	/* Secondary */
	.z-btn-secondary {
		background: var(--bg-elevated);
		color: var(--text-primary);
		border-color: var(--border-default);
	}
	.z-btn-secondary:hover:not(:disabled) {
		background: var(--bg-overlay);
		border-color: var(--border-strong);
	}

	/* Ghost */
	.z-btn-ghost {
		background: transparent;
		color: var(--text-secondary);
	}
	.z-btn-ghost:hover:not(:disabled) {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	/* Danger */
	.z-btn-danger {
		background: rgba(255, 92, 92, 0.1);
		color: var(--error);
		border-color: rgba(255, 92, 92, 0.2);
	}
	.z-btn-danger:hover:not(:disabled) {
		background: rgba(255, 92, 92, 0.2);
		border-color: rgba(255, 92, 92, 0.4);
	}

	/* Spinner */
	.z-btn-spinner {
		width: 14px;
		height: 14px;
		border: 2px solid currentColor;
		border-top-color: transparent;
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.z-btn-icon {
		display: flex;
		align-items: center;
		font-size: 1.1em;
	}
</style>
