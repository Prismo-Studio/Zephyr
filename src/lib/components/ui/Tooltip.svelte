<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		text: string;
		position?: 'top' | 'bottom' | 'left' | 'right';
		children?: Snippet;
	};

	let { text, position = 'top', children }: Props = $props();

	let visible = $state(false);
</script>

<span
	class="z-tooltip-trigger"
	onmouseenter={() => (visible = true)}
	onmouseleave={() => (visible = false)}
>
	{#if children}{@render children()}{/if}
	{#if visible && text}
		<span class="z-tooltip z-tooltip-{position}" role="tooltip">
			{text}
		</span>
	{/if}
</span>

<style>
	.z-tooltip-trigger {
		position: relative;
		display: inline-flex;
	}

	.z-tooltip {
		position: absolute;
		padding: 4px 10px;
		font-size: 11px;
		font-weight: 500;
		color: var(--text-primary);
		background: var(--bg-overlay);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-sm);
		white-space: nowrap;
		pointer-events: none;
		z-index: var(--z-tooltip);
		animation: fadeIn 100ms ease;
		box-shadow: var(--shadow-md);
	}

	.z-tooltip-top { bottom: calc(100% + 6px); left: 50%; transform: translateX(-50%); }
	.z-tooltip-bottom { top: calc(100% + 6px); left: 50%; transform: translateX(-50%); }
	.z-tooltip-left { right: calc(100% + 6px); top: 50%; transform: translateY(-50%); }
	.z-tooltip-right { left: calc(100% + 6px); top: 50%; transform: translateY(-50%); }
</style>
