<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { Snippet } from 'svelte';

	type Props = {
		title: string;
		open: boolean;
		count?: number;
		disabled?: boolean;
		disabledLabel?: string;
		disabledIcon?: string;
		ontoggle: () => void;
		extra?: Snippet;
		children: Snippet;
	};

	let {
		title,
		open,
		count,
		disabled = false,
		disabledLabel,
		disabledIcon = 'mdi:laptop',
		ontoggle,
		extra,
		children
	}: Props = $props();
</script>

<div class="rdz-block" class:rdz-block-disabled={disabled}>
	<div class="rdz-block-row">
		<button class="rdz-block-header" onclick={ontoggle}>
			<span class="rdz-block-name">
				{title}
				{#if count !== undefined}<small>({count})</small>{/if}
			</span>
			{#if disabled && disabledLabel}
				<span class="rdz-block-badge">
					<Icon icon={disabledIcon} />
					{disabledLabel}
				</span>
			{/if}
			<Icon
				icon="mdi:chevron-down"
				class={open ? 'rdz-chev' : 'rdz-chev rdz-chev-closed'}
			/>
		</button>
		{#if extra}{@render extra()}{/if}
	</div>

	{#if open}
		{@render children()}
	{/if}
</div>

<style>
	.rdz-block {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
		padding: var(--space-md) 0;
		border-top: 1px solid var(--border-subtle);
		transition: opacity var(--transition-fast);
		position: relative;
	}

	.rdz-block:first-child {
		border-top: none;
		padding-top: 0;
	}

	.rdz-block-disabled {
		pointer-events: none;
		overflow: hidden;
	}

	.rdz-block-disabled > :not(.rdz-block-row) {
		opacity: 0.45;
	}

	.rdz-block-disabled .rdz-block-badge {
		color: var(--text-primary);
		position: relative;
		z-index: 6;
	}

	.rdz-block-disabled::before {
		content: '';
		position: absolute;
		inset: 0;
		background-image: repeating-linear-gradient(
			135deg,
			transparent,
			transparent 6px,
			rgba(255, 255, 255, 0.04) 6px,
			rgba(255, 255, 255, 0.04) 12px
		);
		pointer-events: none;
		z-index: 5;
	}

	.rdz-block-badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		margin-left: auto;
		padding: 2px 8px;
		font-size: 10px;
		font-weight: 700;
		text-transform: none;
		letter-spacing: 0;
		color: var(--text-muted);
	}

	.rdz-block-row {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		width: 100%;
	}

	.rdz-block-row > :global(.rdz-icon-btn) {
		flex-shrink: 0;
	}

	.rdz-block-header {
		flex: 1;
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		width: 100%;
		padding: 4px 0;
		border: none;
		background: transparent;
		color: var(--text-secondary);
		font-family: var(--font-body);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		cursor: pointer;
		border-radius: 0;
		text-align: left;
		transition: color var(--transition-fast);
	}

	.rdz-block-header:hover {
		color: var(--text-primary);
		background: transparent;
	}

	.rdz-block-name {
		flex: 1;
		color: var(--text-primary);
	}

	.rdz-block-name small {
		color: var(--text-muted);
		font-weight: 600;
		margin-left: 4px;
	}

	:global(.rdz-chev) {
		font-size: 16px;
		color: var(--text-muted);
		transition: transform 180ms ease;
		flex-shrink: 0;
	}

	:global(.rdz-chev-closed) {
		transform: rotate(-90deg);
	}
</style>
