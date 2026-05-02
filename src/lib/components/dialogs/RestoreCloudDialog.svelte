<script lang="ts">
	import Modal from '$lib/components/ui/Modal.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import Icon from '@iconify/svelte';
	import * as api from '$lib/api';
	import profiles from '$lib/state/profile.svelte';
	import { pushToast } from '$lib/toast.svelte';
	import type { ListedSyncProfile } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		open: boolean;
		items: ListedSyncProfile[];
		onclose: () => void;
	};

	let { open = $bindable(), items = $bindable(), onclose }: Props = $props();

	let selected = $state(new Set<string>());
	let busy = $state(false);
	let deletingId: string | null = $state(null);

	$effect(() => {
		if (open) {
			selected = new Set(items.map((p) => p.id));
		}
	});

	function toggle(id: string) {
		const next = new Set(selected);
		if (next.has(id)) next.delete(id);
		else next.add(id);
		selected = next;
	}

	function toggleAll() {
		if (selected.size === items.length) {
			selected = new Set();
		} else {
			selected = new Set(items.map((p) => p.id));
		}
	}

	async function restore() {
		if (selected.size === 0) return;
		busy = true;
		let ok = 0;
		let fail = 0;
		for (const p of items) {
			if (!selected.has(p.id)) continue;
			try {
				await api.profile.sync.clone(p.id, p.name);
				ok++;
			} catch (e) {
				console.error('Failed to clone', p.id, e);
				fail++;
			}
		}
		await profiles.refresh();
		busy = false;
		open = false;
		onclose();
		pushToast({
			type: fail === 0 ? 'info' : 'error',
			name: m.restore_complete(),
			message:
				fail === 0
					? m.restore_completeSuccess({ count: ok.toString() })
					: m.restore_completePartial({ ok: ok.toString(), fail: fail.toString() })
		});
	}

	function close() {
		if (busy) return;
		open = false;
		onclose();
	}

	async function deleteCloud(id: string, name: string) {
		if (deletingId) return;
		deletingId = id;
		try {
			await api.profile.sync.deleteProfile(id);
			items = items.filter((p) => p.id !== id);
			const next = new Set(selected);
			next.delete(id);
			selected = next;
			pushToast({
				type: 'info',
				name: m.sync_unsynced(),
				message: m.sync_removedFromCloud({ name })
			});
			if (items.length === 0) {
				open = false;
				onclose();
			}
		} catch (e: any) {
			pushToast({
				type: 'error',
				name: m.sync_cloudDeleteFailed(),
				message: e?.message ?? String(e)
			});
		} finally {
			deletingId = null;
		}
	}

	function fmtDate(s: string): string {
		try {
			return new Date(s).toLocaleDateString(undefined, {
				year: 'numeric',
				month: 'short',
				day: 'numeric'
			});
		} catch {
			return s;
		}
	}
</script>

<Modal bind:open onclose={close} title={(i18nState.locale && m.restore_title()) || ''}>
	<div class="z-restore">
		<p class="z-restore-hint">
			{i18nState.locale && m.restore_description({ count: items.length.toString() })}
		</p>

		<div class="z-restore-toolbar">
			<button class="z-restore-toggle-all" onclick={toggleAll}>
				<Icon
					icon={selected.size === items.length
						? 'mdi:checkbox-marked'
						: 'mdi:checkbox-blank-outline'}
				/>
				<span>
					{i18nState.locale &&
						(selected.size === items.length ? m.restore_unselectAll() : m.restore_selectAll())}
				</span>
			</button>
			<span class="z-restore-count">{selected.size} / {items.length}</span>
		</div>

		<div class="z-restore-list">
			{#each items as item (item.id)}
				<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
				<div
					class="z-restore-item"
					class:checked={selected.has(item.id)}
					role="button"
					tabindex="0"
					onclick={() => toggle(item.id)}
					onkeydown={(e) => {
						if (e.key === ' ' || e.key === 'Enter') {
							e.preventDefault();
							toggle(item.id);
						}
					}}
				>
					<Checkbox checked={selected.has(item.id)} onchange={() => toggle(item.id)} />
					<div class="z-restore-item-body">
						<div class="z-restore-item-name">{item.name}</div>
						<div class="z-restore-item-meta">
							{#if item.community}<span>{item.community}</span>{/if}
							<span>
								{i18nState.locale && m.restore_updatedPrefix({ date: fmtDate(item.updatedAt) })}
							</span>
						</div>
					</div>
					<button
						class="z-restore-item-delete"
						disabled={deletingId === item.id}
						onclick={(e) => {
							e.stopPropagation();
							deleteCloud(item.id, item.name);
						}}
						title={(i18nState.locale && m.sync_removedFromCloud({ name: item.name })) || ''}
					>
						<Icon icon={deletingId === item.id ? 'mdi:loading' : 'mdi:delete'} />
					</button>
				</div>
			{/each}
		</div>
	</div>

	{#snippet actions()}
		<Button variant="ghost" onclick={close} disabled={busy}>
			{i18nState.locale && m.restore_skip()}
		</Button>
		<Button
			variant="primary"
			onclick={restore}
			loading={busy}
			disabled={busy || selected.size === 0}
		>
			{#snippet icon()}<Icon icon="mdi:cloud-download" />{/snippet}
			{i18nState.locale && m.restore_action()}
			{selected.size > 0 ? `(${selected.size})` : ''}
		</Button>
	{/snippet}
</Modal>

<style>
	.z-restore {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		min-width: 420px;
		max-width: 560px;
	}

	.z-restore-hint {
		font-size: 13px;
		color: var(--text-secondary);
		margin: 0;
	}

	.z-restore-toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.z-restore-toggle-all {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 4px 8px;
		border: none;
		background: transparent;
		color: var(--text-secondary);
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
		border-radius: var(--radius-sm);
		transition: all var(--transition-fast);
	}

	.z-restore-toggle-all:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.z-restore-count {
		font-size: 12px;
		color: var(--text-muted);
	}

	.z-restore-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
		max-height: 320px;
		overflow-y: auto;
		padding-right: 4px;
	}

	.z-restore-item {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: 10px 12px;
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		border: 1px solid var(--border-subtle);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.z-restore-item:hover {
		border-color: var(--border-default);
		background: var(--bg-elevated);
	}

	.z-restore-item.checked {
		border-color: var(--border-accent);
		background: var(--bg-active);
	}

	.z-restore-item-body {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
		flex: 1;
	}

	.z-restore-item-name {
		font-weight: 600;
		font-size: 13px;
		color: var(--text-primary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.z-restore-item-meta {
		display: flex;
		gap: 8px;
		font-size: 11px;
		color: var(--text-muted);
	}

	.z-restore-item-meta span + span::before {
		content: '·';
		margin-right: 8px;
	}

	.z-restore-item-delete {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		background: transparent;
		color: var(--text-muted);
		border-radius: var(--radius-sm);
		cursor: pointer;
		flex-shrink: 0;
		font-size: 15px;
		transition: all var(--transition-fast);
	}

	.z-restore-item-delete:hover:not(:disabled) {
		background: rgba(255, 92, 92, 0.1);
		color: var(--error);
	}

	.z-restore-item-delete:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
