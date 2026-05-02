<script lang="ts">
	import Modal from '$lib/components/ui/Modal.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		open: boolean;
		profileName: string;
		alsoDelete: boolean;
		busy: boolean;
		onconfirm: () => void;
		onclose: () => void;
	};

	let { open, profileName, alsoDelete = $bindable(), busy, onconfirm, onclose }: Props = $props();
</script>

<Modal {open} {onclose} title={(i18nState.locale && m.sync_unsyncTitle()) || ''}>
	<div class="z-modal-form">
		<p class="z-unsync-text">
			{i18nState.locale && m.sync_unsyncText({ name: profileName })}
		</p>
		<div
			class="z-unsync-option"
			role="button"
			tabindex="0"
			onclick={() => {
				if (!busy) alsoDelete = !alsoDelete;
			}}
			onkeydown={(e) => {
				if (e.key === ' ' || e.key === 'Enter') {
					e.preventDefault();
					if (!busy) alsoDelete = !alsoDelete;
				}
			}}
		>
			<Checkbox bind:checked={alsoDelete} disabled={busy} />
			<span>
				{i18nState.locale && m.sync_unsyncAlsoDelete()}
				<span class="z-unsync-hint">
					{i18nState.locale && m.sync_unsyncAlsoDeleteHint()}
				</span>
			</span>
		</div>
	</div>

	{#snippet actions()}
		<Button variant="ghost" onclick={onclose} disabled={busy}>
			{i18nState.locale && m.dialog_cancel()}
		</Button>
		<Button
			variant={alsoDelete ? 'danger' : 'primary'}
			onclick={onconfirm}
			loading={busy}
			disabled={busy}
		>
			{i18nState.locale && (alsoDelete ? m.sync_unsyncAndDeleteAction() : m.sync_unsyncAction())}
		</Button>
	{/snippet}
</Modal>

<style>
	.z-modal-form {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.z-unsync-text {
		font-size: 13px;
		color: var(--text-secondary);
		margin: 0;
	}

	.z-unsync-text :global(strong) {
		color: var(--text-primary);
	}

	.z-unsync-option {
		display: flex;
		align-items: flex-start;
		gap: var(--space-sm);
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		border: 1px solid var(--border-subtle);
		cursor: pointer;
		font-size: 13px;
	}

	.z-unsync-option:hover {
		border-color: var(--border-default);
	}

	.z-unsync-hint {
		display: block;
		font-size: 11px;
		color: var(--text-muted);
		margin-top: 2px;
		line-height: 1.4;
	}
</style>
