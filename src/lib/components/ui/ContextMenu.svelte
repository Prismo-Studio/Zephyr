<script lang="ts">
	import Icon from '@iconify/svelte';

	export type ContextMenuItem = {
		label: string;
		icon?: string;
		danger?: boolean;
		disabled?: boolean;
		separator?: boolean;
		onclick?: () => void;
	};

	type Props = {
		items: ContextMenuItem[];
		x: number;
		y: number;
		onclose: () => void;
	};

	let { items, x, y, onclose }: Props = $props();

	// Adjust position to stay within viewport
	let menuEl: HTMLDivElement | undefined = $state();

	$effect(() => {
		if (menuEl) {
			const rect = menuEl.getBoundingClientRect();
			if (rect.right > window.innerWidth) {
				x = window.innerWidth - rect.width - 8;
			}
			if (rect.bottom > window.innerHeight) {
				y = window.innerHeight - rect.height - 8;
			}
		}
	});

	function handleClick(item: ContextMenuItem) {
		if (item.disabled) return;
		item.onclick?.();
		onclose();
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="z-ctx-backdrop"
	onclick={onclose}
	oncontextmenu={(e) => {
		e.preventDefault();
		onclose();
	}}
>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="z-ctx-menu"
		bind:this={menuEl}
		style="left: {x}px; top: {y}px;"
		onclick={(e) => e.stopPropagation()}
	>
		{#each items as item}
			{#if item.separator}
				<div class="z-ctx-separator"></div>
			{:else}
				<button
					class="z-ctx-item"
					class:danger={item.danger}
					class:disabled={item.disabled}
					onclick={() => handleClick(item)}
					disabled={item.disabled}
				>
					{#if item.icon}
						<Icon icon={item.icon} class="z-ctx-item-icon" />
					{/if}
					<span>{item.label}</span>
				</button>
			{/if}
		{/each}
	</div>
</div>

<style>
	.z-ctx-backdrop {
		position: fixed;
		inset: 0;
		z-index: var(--z-dropdown);
	}

	.z-ctx-menu {
		position: fixed;
		min-width: 180px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		padding: 4px;
		box-shadow:
			var(--shadow-lg),
			0 0 0 1px rgba(0, 0, 0, 0.1);
		animation: scaleIn 80ms ease;
		z-index: var(--z-dropdown);
	}

	.z-ctx-item {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 8px 12px;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		font-family: var(--font-body);
		font-size: 13px;
		cursor: pointer;
		transition: all 80ms ease;
		text-align: left;
	}

	.z-ctx-item:hover:not(:disabled) {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.z-ctx-item.danger {
		color: var(--text-secondary);
	}

	.z-ctx-item.danger:hover:not(:disabled) {
		background: rgba(255, 92, 92, 0.1);
		color: var(--error);
	}

	.z-ctx-item.disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	:global(.z-ctx-item-icon) {
		font-size: 16px;
		flex-shrink: 0;
	}

	.z-ctx-separator {
		height: 1px;
		background: var(--border-subtle);
		margin: 4px 8px;
	}
</style>
