<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { MouseEventHandler } from 'svelte/elements';

	type Props = {
		variant?: 'default' | 'glass' | 'interactive';
		class?: string;
		onclick?: MouseEventHandler<HTMLDivElement>;
		children?: Snippet;
	};

	let { variant = 'default', class: className = '', onclick, children }: Props = $props();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="z-card z-card-{variant} {className}"
	{onclick}
	onkeydown={onclick ? (e) => { if (e.key === 'Enter') onclick?.(e as any); } : undefined}
	role={onclick ? 'button' : undefined}
	tabindex={onclick ? 0 : undefined}
>
	{#if children}{@render children()}{/if}
</div>

<style>
	.z-card {
		border-radius: var(--radius-lg);
		border: 1px solid var(--border-subtle);
		padding: var(--space-lg);
		transition: all var(--transition-normal);
	}

	.z-card-default {
		background: var(--bg-surface);
	}

	.z-card-glass {
		background: var(--glass-bg);
		backdrop-filter: blur(var(--glass-blur));
		-webkit-backdrop-filter: blur(var(--glass-blur));
		border-color: var(--glass-border);
	}

	.z-card-interactive {
		background: var(--bg-surface);
		cursor: pointer;
	}
	.z-card-interactive:hover {
		background: var(--bg-elevated);
		border-color: var(--border-default);
		transform: translateY(-1px);
		box-shadow: var(--shadow-md);
	}
	.z-card-interactive:active {
		transform: translateY(0);
	}
</style>
