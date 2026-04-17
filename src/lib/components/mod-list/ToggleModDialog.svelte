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
		/** true = we're trying to disable, false = trying to enable */
		isDisabling: boolean;
		dependants: Dependant[];
		oncancel?: () => void;
		/** Toggle only the target mod */
		ontoggleOne?: () => void;
		/** Toggle the target mod AND all dependants */
		ontoggleAll?: () => void;
	};

	let {
		open = $bindable(false),
		modName,
		isDisabling,
		dependants,
		oncancel,
		ontoggleOne,
		ontoggleAll
	}: Props = $props();

	function handleCancel() {
		open = false;
		oncancel?.();
	}

	function handleToggleOne() {
		open = false;
		ontoggleOne?.();
	}

	function handleToggleAll() {
		open = false;
		ontoggleAll?.();
	}

	let title = $derived(
		isDisabling
			? m.toggleModDialog_disable_title({ name: modName })
			: m.toggleModDialog_enable_title({ name: modName })
	);

	let description = $derived(
		isDisabling
			? m.toggleModDialog_disable_description({ name: modName })
			: m.toggleModDialog_enable_description({ name: modName })
	);

	let hint = $derived(
		isDisabling ? m.toggleModDialog_disable_hint() : m.toggleModDialog_enable_hint()
	);

	let toggleOneLabel = $derived(
		isDisabling
			? m.toggleModDialog_disable_one({ name: modName })
			: m.toggleModDialog_enable_one({ name: modName })
	);

	let toggleAllLabel = $derived(
		isDisabling
			? m.toggleModDialog_disable_all({ count: (dependants.length + 1).toString() })
			: m.toggleModDialog_enable_all({ count: (dependants.length + 1).toString() })
	);
</script>

<Modal bind:open {title} onclose={handleCancel}>
	{#snippet children()}
		<div class="z-tmd-body">
			<div class="z-tmd-warning" class:enabling={!isDisabling}>
				<Icon
					icon={isDisabling ? 'mdi:alert-circle' : 'mdi:information'}
					class="z-tmd-warning-icon"
				/>
				<p>{description}</p>
			</div>

			<ul class="z-tmd-list">
				{#each dependants as dep (dep.uuid)}
					<li class="z-tmd-item">
						<Icon icon="mdi:puzzle" />
						<span>{dep.fullName}</span>
					</li>
				{/each}
			</ul>

			<p class="z-tmd-hint">{hint}</p>
		</div>
	{/snippet}

	{#snippet actions()}
		<Button variant="ghost" onclick={handleCancel}>
			{m.toggleModDialog_cancel()}
		</Button>
		<Tooltip text={toggleOneLabel} position="top">
			<button class="z-tmd-action-btn secondary" onclick={handleToggleOne}>
				<span class="z-tmd-action-label">{toggleOneLabel}</span>
			</button>
		</Tooltip>
		<Tooltip text={toggleAllLabel} position="top">
			<button
				class="z-tmd-action-btn {isDisabling ? 'danger' : 'accent'}"
				onclick={handleToggleAll}
			>
				<Icon icon={isDisabling ? 'mdi:eye-off' : 'mdi:eye'} />
				<span class="z-tmd-action-label">{toggleAllLabel}</span>
			</button>
		</Tooltip>
	{/snippet}
</Modal>

<style>
	.z-tmd-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.z-tmd-warning {
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

	.z-tmd-warning.enabling {
		background: var(--bg-active);
		border-color: var(--border-accent);
		color: var(--text-accent);
	}

	:global(.z-tmd-warning-icon) {
		flex-shrink: 0;
		font-size: 18px;
		margin-top: 1px;
	}

	.z-tmd-list {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
		max-height: 220px;
		overflow-y: auto;
	}

	.z-tmd-item {
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

	.z-tmd-item :global(svg) {
		color: var(--text-muted);
		font-size: 14px;
		flex-shrink: 0;
	}

	.z-tmd-hint {
		font-size: 12px;
		color: var(--text-muted);
		line-height: 1.5;
	}

	/* Action buttons with truncation */
	.z-tmd-action-btn {
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

	.z-tmd-action-btn :global(svg) {
		flex-shrink: 0;
		font-size: 1.1em;
	}

	.z-tmd-action-label {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.z-tmd-action-btn.secondary {
		background: var(--bg-elevated);
		color: var(--text-primary);
		border-color: var(--border-default);
	}

	.z-tmd-action-btn.secondary:hover {
		background: var(--bg-overlay);
		border-color: var(--border-strong);
	}

	.z-tmd-action-btn.danger {
		background: rgba(255, 92, 92, 0.1);
		color: var(--error);
		border-color: rgba(255, 92, 92, 0.2);
	}

	.z-tmd-action-btn.danger:hover {
		background: rgba(255, 92, 92, 0.2);
		border-color: rgba(255, 92, 92, 0.4);
	}

	.z-tmd-action-btn.accent {
		background: var(--accent-400);
		color: var(--text-inverse);
	}

	.z-tmd-action-btn.accent:hover {
		background: var(--accent-300);
		box-shadow: var(--shadow-glow);
	}
</style>
