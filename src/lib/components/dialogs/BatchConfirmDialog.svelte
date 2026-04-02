<script lang="ts">
	import Icon from '@iconify/svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		open: boolean;
		title: string;
		description: string;
		hint: string;
		extraDependants: string[];
		confirmLabel: string;
		cancelLabel: string;
		confirmVariant?: 'accent' | 'danger';
		confirmIcon?: string;
		onconfirm: () => void;
		oncancel: () => void;
	};

	let {
		open = $bindable(),
		title,
		description,
		hint,
		extraDependants,
		confirmLabel,
		cancelLabel,
		confirmVariant = 'accent',
		confirmIcon = 'mdi:check',
		onconfirm,
		oncancel
	}: Props = $props();
</script>

<Modal bind:open {title} onclose={oncancel}>
	{#snippet children()}
		<div class="z-batch-body">
			<div class="z-batch-warning">
				<Icon icon="mdi:alert-circle" class="z-batch-warning-icon" />
				<p>{description}</p>
			</div>

			<ul class="z-batch-list">
				{#each extraDependants as dep}
					<li class="z-batch-item">
						<Icon icon="mdi:puzzle" />
						<span>{dep}</span>
					</li>
				{/each}
			</ul>

			<p class="z-batch-hint">{hint}</p>
		</div>
	{/snippet}

	{#snippet actions()}
		<Button variant="ghost" onclick={oncancel}>
			{cancelLabel}
		</Button>
		<Button variant={confirmVariant} onclick={onconfirm}>
			{#snippet icon()}<Icon icon={confirmIcon} />{/snippet}
			{confirmLabel}
		</Button>
	{/snippet}
</Modal>

<style>
	.z-batch-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.z-batch-warning {
		display: flex;
		align-items: flex-start;
		gap: var(--space-sm);
		padding: var(--space-md);
		border-radius: var(--radius-md);
		background: rgba(255, 179, 71, 0.08);
		border: 1px solid rgba(255, 179, 71, 0.25);
		color: var(--warning);
		font-size: 13px;
		line-height: 1.5;
	}

	:global(.z-batch-warning-icon) {
		flex-shrink: 0;
		font-size: 18px;
		margin-top: 1px;
	}

	.z-batch-list {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
		max-height: 220px;
		overflow-y: auto;
	}

	.z-batch-item {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-sm);
		background: var(--bg-elevated);
		border: 1px solid var(--border-subtle);
		font-size: 12px;
		color: var(--text-secondary);
	}

	.z-batch-item :global(svg) {
		color: var(--text-muted);
		font-size: 14px;
		flex-shrink: 0;
	}

	.z-batch-hint {
		font-size: 12px;
		color: var(--text-muted);
		line-height: 1.5;
	}
</style>
