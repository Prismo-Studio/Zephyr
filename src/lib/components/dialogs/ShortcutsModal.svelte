<script lang="ts">
	import Modal from '$lib/components/ui/Modal.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Icon from '@iconify/svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import {
		shortcuts,
		SHORTCUT_DEFS,
		formatBinding,
		bindingFromEvent,
		type Binding,
		type ShortcutId,
		type GroupId
	} from '$lib/state/shortcuts.svelte';

	type Props = {
		open?: boolean;
		onclose?: () => void;
	};

	let { open = $bindable(false), onclose }: Props = $props();

	const groupOrder: GroupId[] = ['general', 'profiles', 'display', 'randomizer', 'console'];
	const groupTitleKey: Record<GroupId, string> = {
		general: 'shortcuts_group_general',
		profiles: 'shortcuts_group_profiles',
		display: 'shortcuts_group_display',
		randomizer: 'shortcuts_group_randomizer',
		console: 'shortcuts_group_console'
	};

	const groups = $derived(
		groupOrder.map((g) => ({
			id: g,
			defs: SHORTCUT_DEFS.filter((d) => d.group === g)
		}))
	);

	let editingId = $state<ShortcutId | null>(null);
	let pending = $state<Binding | null>(null);
	let conflictId = $state<ShortcutId | null>(null);

	function startEdit(id: ShortcutId) {
		editingId = id;
		pending = null;
		conflictId = null;
	}

	function cancelEdit() {
		editingId = null;
		pending = null;
		conflictId = null;
	}

	function save() {
		if (!editingId || !pending) return;
		if (conflictId) return;
		shortcuts.set(editingId, pending);
		cancelEdit();
	}

	function onCaptureKeydown(e: KeyboardEvent) {
		if (!editingId) return;
		e.preventDefault();
		e.stopPropagation();
		if (e.key === 'Escape') {
			cancelEdit();
			return;
		}
		const b = bindingFromEvent(e);
		if (!b) return;
		pending = b;
		conflictId = shortcuts.conflictWith(editingId, b);
	}

	function labelFor(key: string): string {
		// Dynamic lookup into the paraglide messages bag.
		const fn = (m as unknown as Record<string, () => string>)[key];
		return fn ? fn() : key;
	}
</script>

{#if i18nState.locale}
	<Modal bind:open title={m.shortcuts_modalTitle()} {onclose}>
		{#snippet children()}
			<div class="z-shortcuts">
				<p class="z-shortcuts-intro">{m.shortcuts_intro()}</p>
				{#each groups as group}
					<section class="z-shortcuts-group">
						<h4 class="z-shortcuts-group-title">{labelFor(groupTitleKey[group.id])}</h4>
						<ul class="z-shortcuts-list">
							{#each group.defs as def}
								{@const isEditing = editingId === def.id}
								{@const binding = shortcuts.get(def.id)}
								<li class="z-shortcuts-row">
									<span class="z-shortcuts-label">{labelFor(def.labelKey)}</span>
									{#if isEditing}
										<span class="z-shortcuts-edit">
											<!-- svelte-ignore a11y_autofocus -->
											<input
												type="text"
												class="z-shortcuts-capture"
												class:has-conflict={!!conflictId}
												readonly
												autofocus
												onkeydown={onCaptureKeydown}
												value={pending
													? formatBinding(pending)
													: m.shortcuts_listening()}
												aria-label={m.shortcuts_listening()}
											/>
											<button
												class="z-shortcuts-icon-btn"
												onclick={save}
												disabled={!pending || !!conflictId}
												title={m.shortcuts_save()}
												aria-label={m.shortcuts_save()}
											>
												<Icon icon="mdi:check" />
											</button>
											<button
												class="z-shortcuts-icon-btn"
												onclick={cancelEdit}
												title={m.shortcuts_cancel()}
												aria-label={m.shortcuts_cancel()}
											>
												<Icon icon="mdi:close" />
											</button>
										</span>
									{:else}
										<span class="z-shortcuts-keys">
											<kbd class="z-shortcuts-key">{formatBinding(binding)}</kbd>
											<button
												class="z-shortcuts-icon-btn"
												onclick={() => startEdit(def.id)}
												title={m.shortcuts_edit()}
												aria-label={m.shortcuts_edit()}
											>
												<Icon icon="mdi:pencil" />
											</button>
											<button
												class="z-shortcuts-icon-btn"
												onclick={() => shortcuts.resetOne(def.id)}
												title={m.shortcuts_resetOne()}
												aria-label={m.shortcuts_resetOne()}
											>
												<Icon icon="mdi:restore" />
											</button>
										</span>
									{/if}
								</li>
								{#if isEditing && conflictId}
									<li class="z-shortcuts-conflict">
										{m.shortcuts_conflict()}: {labelFor(
											SHORTCUT_DEFS.find((d) => d.id === conflictId)?.labelKey ?? ''
										)}
									</li>
								{/if}
							{/each}
						</ul>
					</section>
				{/each}
			</div>
		{/snippet}
		{#snippet actions()}
			<Button variant="secondary" onclick={() => shortcuts.resetAll()}>
				{#snippet icon()}<Icon icon="mdi:restore" />{/snippet}
				{m.shortcuts_resetAll()}
			</Button>
		{/snippet}
	</Modal>
{/if}

<style>
	.z-shortcuts {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg);
		min-width: 460px;
		max-width: 560px;
	}

	.z-shortcuts-intro {
		color: var(--text-secondary);
		font-size: 13px;
		margin: 0 0 var(--space-xs);
	}

	.z-shortcuts-group-title {
		margin: 0 0 var(--space-sm);
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--accent-400);
		font-weight: 700;
	}

	.z-shortcuts-list {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
	}

	.z-shortcuts-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 12px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-sm);
	}

	.z-shortcuts-label {
		color: var(--text-primary);
		font-size: 13px;
	}

	.z-shortcuts-keys,
	.z-shortcuts-edit {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}

	.z-shortcuts-key {
		display: inline-block;
		padding: 2px 8px;
		min-width: 24px;
		text-align: center;
		font-family: var(--font-mono, monospace);
		font-size: 12px;
		color: var(--accent-300, var(--accent-400));
		background: var(--bg-base);
		border: 1px solid var(--accent-400);
		border-radius: var(--radius-sm);
		box-shadow: 0 1px 0 var(--accent-700, rgba(0, 0, 0, 0.2));
	}

	.z-shortcuts-icon-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		padding: 0;
		border: 1px solid transparent;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		border-radius: var(--radius-sm);
		font-size: 14px;
		transition: all var(--transition-fast);
	}

	.z-shortcuts-icon-btn:hover:not(:disabled) {
		color: var(--accent-400);
		border-color: var(--accent-400);
		background: var(--bg-hover);
	}

	.z-shortcuts-icon-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.z-shortcuts-capture {
		font-family: var(--font-mono, monospace);
		font-size: 12px;
		padding: 3px 10px;
		min-width: 180px;
		color: var(--accent-300, var(--accent-400));
		background: var(--bg-base);
		border: 1px solid var(--accent-400);
		border-radius: var(--radius-sm);
		outline: none;
		text-align: center;
	}

	.z-shortcuts-capture.has-conflict {
		border-color: var(--color-danger, #e57373);
		color: var(--color-danger, #e57373);
	}

	.z-shortcuts-conflict {
		font-size: 11px;
		color: var(--color-danger, #e57373);
		padding: 0 12px 4px;
	}
</style>
