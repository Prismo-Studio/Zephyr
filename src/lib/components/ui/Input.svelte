<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		value?: string;
		placeholder?: string;
		type?: string;
		disabled?: boolean;
		class?: string;
		oninput?: (e: Event & { currentTarget: HTMLInputElement }) => void;
		onkeydown?: (e: KeyboardEvent) => void;
		iconLeft?: Snippet;
		iconRight?: Snippet;
	};

	let {
		value = $bindable(''),
		placeholder = '',
		type = 'text',
		disabled = false,
		class: className = '',
		oninput,
		onkeydown,
		iconLeft,
		iconRight
	}: Props = $props();
</script>

<div class="z-input-wrapper {className}">
	{#if iconLeft}
		<span class="z-input-icon left">{@render iconLeft()}</span>
	{/if}
	<input
		class="z-input"
		class:has-icon-left={!!iconLeft}
		class:has-icon-right={!!iconRight}
		{type}
		{placeholder}
		{disabled}
		bind:value
		{oninput}
		{onkeydown}
	/>
	{#if iconRight}
		<span class="z-input-icon right">{@render iconRight()}</span>
	{/if}
</div>

<style>
	.z-input-wrapper {
		position: relative;
		display: flex;
		align-items: center;
	}

	.z-input {
		width: 100%;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		padding: 8px 12px;
		color: var(--text-primary);
		font-family: var(--font-body);
		font-size: 13px;
		transition: all var(--transition-fast);
		outline: none;
	}

	.z-input::placeholder {
		color: var(--text-muted);
	}

	.z-input:focus {
		border-color: var(--accent-400);
		box-shadow: 0 0 0 3px rgba(26, 255, 250, 0.1);
	}

	.z-input:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.z-input.has-icon-left {
		padding-left: 36px;
	}
	.z-input.has-icon-right {
		padding-right: 36px;
	}

	.z-input-icon {
		position: absolute;
		display: flex;
		align-items: center;
		color: var(--text-muted);
		font-size: 16px;
		pointer-events: none;
	}
	.z-input-icon.left {
		left: 10px;
	}
	.z-input-icon.right {
		right: 10px;
	}
</style>
