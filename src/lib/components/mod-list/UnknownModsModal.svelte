<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import * as api from '$lib/api';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { pushToast } from '$lib/toast.svelte';

	type UnknownMod = { fullName: string; uuid: string };

	type Props = {
		open: boolean;
		mods: UnknownMod[];
		onclose: () => void;
		onchange: (next: UnknownMod[]) => void;
		onafterRemove: () => Promise<void> | void;
	};

	let { open = $bindable(), mods, onclose, onchange, onafterRemove }: Props = $props();

	let busy = $state(false);

	async function removeOne(uuid: string) {
		busy = true;
		try {
			await api.profile.forceRemoveMods([uuid]);
			const next = mods.filter((u) => u.uuid !== uuid);
			onchange(next);
			if (next.length === 0) open = false;
			await onafterRemove();
		} catch (err) {
			pushToast({
				type: 'error',
				name: m.mods_unknownMods_removeFailed(),
				message: err instanceof Error ? err.message : String(err)
			});
		} finally {
			busy = false;
		}
	}

	async function removeAll() {
		if (mods.length === 0) return;
		busy = true;
		try {
			await api.profile.forceRemoveMods(mods.map((u) => u.uuid));
			onchange([]);
			open = false;
			await onafterRemove();
		} catch (err) {
			pushToast({
				type: 'error',
				name: m.mods_unknownMods_removeFailed(),
				message: err instanceof Error ? err.message : String(err)
			});
		} finally {
			busy = false;
		}
	}
</script>

<Modal bind:open title={m.mods_unknownMods({ count: mods.length.toString() })} {onclose}>
	<div class="z-unknown-modal">
		<p class="z-unknown-desc">
			{i18nState.locale && m.mods_unknownMods_desc()}
		</p>
		<ul class="z-unknown-list">
			{#each mods as unknown (unknown.uuid)}
				<li class="z-unknown-item">
					<Icon icon="mdi:help-circle" />
					<code>{unknown.fullName}</code>
					<button
						class="z-unknown-remove"
						disabled={busy}
						onclick={() => removeOne(unknown.uuid)}
						title={i18nState.locale && m.mods_unknownMods_removeTooltip()}
					>
						<Icon icon="mdi:delete" />
					</button>
				</li>
			{/each}
		</ul>
	</div>
	{#snippet actions()}
		<Button variant="ghost" onclick={() => (open = false)}>
			{i18nState.locale && m.mods_unknownMods_close()}
		</Button>
		<Button variant="danger" disabled={busy} onclick={removeAll}>
			{#snippet icon()}<Icon icon="mdi:delete-sweep" />{/snippet}
			{i18nState.locale && m.mods_unknownMods_removeAll()}
		</Button>
	{/snippet}
</Modal>

<style>
	.z-unknown-modal {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.z-unknown-desc {
		margin: 0;
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.5;
	}

	.z-unknown-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
		max-height: 360px;
		overflow-y: auto;
	}

	.z-unknown-item {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: 8px 12px;
		border-radius: var(--radius-sm);
		background: var(--bg-base);
		border: 1px solid var(--border-subtle);
	}

	.z-unknown-item > :global(svg:first-child) {
		color: var(--warning);
		flex-shrink: 0;
	}

	.z-unknown-item code {
		flex: 1;
		min-width: 0;
		font-size: 12px;
		color: var(--text-primary);
		background: transparent;
		padding: 0;
		overflow-wrap: anywhere;
		word-break: break-all;
	}

	.z-unknown-remove {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
		flex-shrink: 0;
	}

	.z-unknown-remove:hover {
		color: var(--error);
		border-color: var(--error);
	}

	.z-unknown-remove:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
</style>
