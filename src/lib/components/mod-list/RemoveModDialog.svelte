<script lang="ts">
	import Modal from '$lib/components/ui/Modal.svelte';
	import Button from '$lib/components/ui/Button.svelte';
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
		<Button variant="secondary" onclick={handleRemoveOne}>
			{m.removeModDialog_removeOne({ name: modName })}
		</Button>
		<Button variant="danger" onclick={handleRemoveAll}>
			{#snippet icon()}<Icon icon="mdi:delete-sweep" />{/snippet}
			{m.removeModDialog_removeAll({ count: (dependants.length + 1).toString() })}
		</Button>
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
</style>
