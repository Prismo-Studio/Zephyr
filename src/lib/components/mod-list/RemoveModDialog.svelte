<script lang="ts">
	import Modal from '$lib/components/ui/Modal.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import Icon from '@iconify/svelte';
	import type { Dependant } from '$lib/types';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		open?: boolean;
		modName: string;
		dependants: Dependant[];
		oncancel?: () => void;
		/** Remove only the target mod, ignoring dependants */
		onremoveOne?: () => void;
		/** Remove the target mod AND all dependants */
		onremoveAll?: () => void;
	};

	let {
		open = $bindable(false),
		modName,
		dependants,
		oncancel,
		onremoveOne,
		onremoveAll
	}: Props = $props();

	function handleCancel() {
		open = false;
		oncancel?.();
	}

	function handleRemoveOne() {
		open = false;
		onremoveOne?.();
	}

	function handleRemoveAll() {
		open = false;
		onremoveAll?.();
	}
</script>

<Modal bind:open title={m.removeModDialog_title({ name: modName })} onclose={handleCancel}>
	{#snippet children()}
		<div class="z-rmd-body">
			<div class="z-rmd-warning">
				<Icon icon="mdi:alert-circle" class="z-rmd-warning-icon" />
				<p>{m.removeModDialog_description({ name: modName })}</p>
			</div>

			<ul class="z-rmd-list">
				{#each dependants as dep (dep.uuid)}
					<li class="z-rmd-item">
						<Icon icon="mdi:puzzle" />
						<span>{dep.fullName}</span>
					</li>
				{/each}
			</ul>

			<p class="z-rmd-hint">{m.removeModDialog_hint()}</p>
		</div>
	{/snippet}

	{#snippet actions()}
		<Button variant="ghost" onclick={handleCancel}>
			{m.removeModDialog_cancel()}
		</Button>
		<Tooltip text={m.removeModDialog_removeOne({ name: modName })} position="top">
			<button class="z-rmd-action-btn secondary" onclick={handleRemoveOne}>
				<span class="z-rmd-action-label">{m.removeModDialog_removeOne({ name: modName })}</span>
			</button>
		</Tooltip>
		<Tooltip
			text={m.removeModDialog_removeAll({ count: (dependants.length + 1).toString() })}
			position="top"
		>
			<button class="z-rmd-action-btn danger" onclick={handleRemoveAll}>
				<Icon icon="mdi:delete-sweep" />
				<span class="z-rmd-action-label"
					>{m.removeModDialog_removeAll({ count: (dependants.length + 1).toString() })}</span
				>
			</button>
		</Tooltip>
	{/snippet}
</Modal>

<style>
	.z-rmd-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.z-rmd-warning {
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

	:global(.z-rmd-warning-icon) {
		flex-shrink: 0;
		font-size: 18px;
		margin-top: 1px;
	}

	.z-rmd-list {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
		max-height: 220px;
		overflow-y: auto;
	}

	.z-rmd-item {
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

	.z-rmd-item :global(svg) {
		color: var(--text-muted);
		font-size: 14px;
		flex-shrink: 0;
	}

	.z-rmd-hint {
		font-size: 12px;
		color: var(--text-muted);
		line-height: 1.5;
	}

	/* Action buttons with truncation */
	.z-rmd-action-btn {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 8px 16px;
		border-radius: var(--radius-md);
		border: 1px solid transparent;
		font-family: var(--font-body);
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		transition: all var(--transition-fast);
		max-width: 200px;
		overflow: hidden;
	}

	.z-rmd-action-btn :global(svg) {
		flex-shrink: 0;
		font-size: 1.1em;
	}

	.z-rmd-action-label {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.z-rmd-action-btn.secondary {
		background: var(--bg-elevated);
		color: var(--text-primary);
		border-color: var(--border-default);
	}

	.z-rmd-action-btn.secondary:hover {
		background: var(--bg-overlay);
		border-color: var(--border-strong);
	}

	.z-rmd-action-btn.danger {
		background: rgba(255, 92, 92, 0.1);
		color: var(--error);
		border-color: rgba(255, 92, 92, 0.2);
	}

	.z-rmd-action-btn.danger:hover {
		background: rgba(255, 92, 92, 0.2);
		border-color: rgba(255, 92, 92, 0.4);
	}
</style>
