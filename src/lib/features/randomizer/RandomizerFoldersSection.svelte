<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';
	import { open as selectDirectory } from '@tauri-apps/plugin-dialog';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import { pushInfoToast } from '$lib/toast.svelte';
	import * as api from './api';
	import type { RandomizerDirSetting, RandomizerDirs } from './types';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Kind = 'runtime' | 'players';

	type Props = {
		/** Fired after a directory actually changed, so the panel can re-list. */
		onchange?: () => void;
	};

	let { onchange }: Props = $props();

	let dirs: RandomizerDirs | null = $state(null);
	let busy = $state(false);
	/** Set once a target is chosen, while we ask what to do with the old files. */
	let pending: { kind: Kind; from: string; to: string } | null = $state(null);
	let moveModalOpen = $state(false);

	function askAboutMove(kind: Kind, from: string, to: string) {
		pending = { kind, from, to };
		moveModalOpen = true;
	}

	function cancelMove() {
		moveModalOpen = false;
		pending = null;
	}

	async function load() {
		try {
			dirs = await api.randomizerDirs();
		} catch {
			// invoke() already surfaced the error
		}
	}

	onMount(load);

	function setting(kind: Kind): RandomizerDirSetting | null {
		if (!dirs) return null;
		return kind === 'runtime' ? dirs.runtime : dirs.players;
	}

	async function pick(kind: Kind) {
		const current = setting(kind);
		if (!current) return;
		const selected = await selectDirectory({
			directory: true,
			multiple: false,
			defaultPath: current.effective
		});
		if (!selected || typeof selected !== 'string' || selected === current.effective) return;
		askAboutMove(kind, current.effective, selected);
	}

	function resetToDefault(kind: Kind) {
		const current = setting(kind);
		if (!current || current.effective === current.default) return;
		askAboutMove(kind, current.effective, current.default);
	}

	async function apply(moveExisting: boolean) {
		if (!pending) return;
		const { kind, to } = pending;
		const current = setting(kind);
		cancelMove();
		busy = true;
		try {
			// null restores the default location on the backend.
			const target = to === current?.default ? null : to;
			const result =
				kind === 'runtime'
					? await api.setRandomizerRuntimeDir(target, moveExisting)
					: await api.setRandomizerPlayersDir(target, moveExisting);
			dirs = result.dirs;
			const effective = kind === 'runtime' ? result.dirs.runtime : result.dirs.players;
			if (result.moved) {
				pushInfoToast({ message: m.randomizer_folders_moved({ path: effective.effective }) });
			} else if (result.left_behind) {
				pushInfoToast({ message: m.randomizer_folders_leftBehind({ path: result.left_behind }) });
			}
			onchange?.();
		} catch {
			// invoke() already surfaced the error
		} finally {
			busy = false;
		}
	}

	async function copyPath(path: string) {
		await writeText(path);
		pushInfoToast({ message: m.prefs_copyPath_success() });
	}

	type Row = { kind: Kind; label: string; value: RandomizerDirSetting };

	const rows = $derived.by((): Row[] => {
		const current = dirs;
		if (!current) return [];
		return [
			{
				kind: 'runtime',
				label: (i18nState.locale && m.randomizer_folders_runtime()) ?? '',
				value: current.runtime
			},
			{
				kind: 'players',
				label: (i18nState.locale && m.randomizer_folders_players()) ?? '',
				value: current.players
			}
		];
	});
</script>

<div class="rdz-folders">
	{#each rows as row (row.kind)}
		<div class="rdz-folder">
			<div class="rdz-folder-head">
				<span class="rdz-folder-label">{row.label}</span>
				<div class="rdz-folder-actions">
					<Tooltip text={i18nState.locale && m.randomizer_copy()} position="top" delay={200}>
						<button
							class="rdz-icon-btn"
							onclick={() => copyPath(row.value.effective)}
							aria-label={i18nState.locale && m.randomizer_copy()}
						>
							<Icon icon="mdi:content-copy" />
						</button>
					</Tooltip>
					<Tooltip text={i18nState.locale && m.randomizer_openFolder()} position="top" delay={200}>
						<button
							class="rdz-icon-btn"
							onclick={() => api.openRandomizerDir(row.kind)}
							aria-label={i18nState.locale && m.randomizer_openFolder()}
						>
							<Icon icon="mdi:folder-open" />
						</button>
					</Tooltip>
					<Tooltip
						text={i18nState.locale && m.randomizer_folders_change()}
						position="top"
						delay={200}
					>
						<button
							class="rdz-icon-btn"
							onclick={() => pick(row.kind)}
							disabled={busy}
							aria-label={i18nState.locale && m.randomizer_folders_change()}
						>
							<Icon icon="mdi:folder-edit" />
						</button>
					</Tooltip>
					{#if row.value.effective !== row.value.default}
						<Tooltip
							text={i18nState.locale && m.randomizer_folders_reset()}
							position="top"
							delay={200}
						>
							<button
								class="rdz-icon-btn"
								onclick={() => resetToDefault(row.kind)}
								disabled={busy}
								aria-label={i18nState.locale && m.randomizer_folders_reset()}
							>
								<Icon icon="mdi:backup-restore" />
							</button>
						</Tooltip>
					{/if}
				</div>
			</div>
			<code>{row.value.effective}</code>
			{#if row.value.effective === row.value.default}
				<span class="rdz-folder-note">{i18nState.locale && m.randomizer_folders_default()}</span>
			{/if}
			{#if row.value.fallback_reason}
				<p class="rdz-folder-err">
					<Icon icon="mdi:alert-circle-outline" />
					{i18nState.locale && m.randomizer_folders_fallback({ reason: row.value.fallback_reason })}
				</p>
			{:else if !row.value.writable}
				<p class="rdz-folder-err">
					<Icon icon="mdi:alert-circle-outline" />
					{i18nState.locale && m.randomizer_folders_notWritable()}
				</p>
			{/if}
		</div>
	{/each}
</div>

<Modal
	bind:open={moveModalOpen}
	title={(i18nState.locale && m.randomizer_folders_moveTitle()) ?? ''}
	onclose={cancelMove}
>
	{#if pending}
		<p class="rdz-folder-prompt">
			{i18nState.locale && m.randomizer_folders_movePrompt({ from: pending.from, to: pending.to })}
		</p>
	{/if}
	{#snippet actions()}
		<Button variant="ghost" onclick={() => apply(false)}>
			{i18nState.locale && m.randomizer_folders_leaveFiles()}
		</Button>
		<Button variant="primary" onclick={() => apply(true)}>
			{i18nState.locale && m.randomizer_folders_moveFiles()}
		</Button>
	{/snippet}
</Modal>

<style>
	.rdz-folders {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.rdz-folder {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.rdz-folder-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-sm);
	}

	.rdz-folder-label {
		font-size: 12px;
		color: var(--text-muted);
	}

	.rdz-folder-actions {
		display: flex;
		gap: var(--space-xs);
	}

	.rdz-folder code {
		font-family: var(--font-mono, monospace);
		font-size: 12px;
		color: var(--text-secondary);
		padding: var(--space-sm) var(--space-md);
		background: var(--bg-elevated);
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-subtle);
		word-break: break-all;
	}

	.rdz-folder-note {
		font-size: 11px;
		color: var(--text-muted);
		font-style: italic;
	}

	.rdz-folder-err {
		display: flex;
		align-items: flex-start;
		gap: 4px;
		margin: 0;
		font-size: 11px;
		color: #ef9a9a;
	}

	.rdz-folder-err :global(svg) {
		font-size: 13px;
		flex-shrink: 0;
		margin-top: 1px;
	}

	.rdz-folder-prompt {
		margin: 0;
		font-size: 13px;
		line-height: 1.5;
		color: var(--text-secondary);
		word-break: break-all;
	}

	.rdz-icon-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition:
			color var(--transition-fast),
			background var(--transition-fast);
	}

	.rdz-icon-btn:hover:not(:disabled) {
		color: var(--text-primary);
		background: var(--bg-hover);
	}

	.rdz-icon-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
</style>
