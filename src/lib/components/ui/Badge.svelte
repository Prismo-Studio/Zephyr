<script lang="ts">
	import type { Snippet } from 'svelte';

	type Variant = 'default' | 'accent' | 'success' | 'warning' | 'error';

	type Props = {
		variant?: Variant;
		class?: string;
		children?: Snippet;
	};

	let { variant = 'default', class: className = '', children }: Props = $props();
</script>

<span class="z-badge z-badge-{variant} {className}">
	{#if children}{@render children()}{/if}
</span>

<style>
	.z-badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		border-radius: var(--radius-full);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.02em;
	}

	.z-badge-default {
		background: var(--bg-overlay);
		color: var(--text-secondary);
		border: 1px solid var(--border-subtle);
	}
	.z-badge-accent {
		background: var(--bg-active);
		color: var(--text-accent);
		border: 1px solid var(--border-accent);
	}
	.z-badge-success {
		background: rgba(0, 212, 170, 0.1);
		color: var(--success);
		border: 1px solid rgba(0, 212, 170, 0.2);
	}
	.z-badge-warning {
		background: rgba(255, 179, 71, 0.1);
		color: var(--warning);
		border: 1px solid rgba(255, 179, 71, 0.2);
	}
	.z-badge-error {
		background: rgba(255, 92, 92, 0.1);
		color: var(--error);
		border: 1px solid rgba(255, 92, 92, 0.2);
	}
</style>
