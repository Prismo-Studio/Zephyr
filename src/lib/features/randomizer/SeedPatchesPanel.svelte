<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import { onMount, onDestroy } from 'svelte';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';
	import {
		applyPatch,
		clearRomPath,
		deletePatch,
		getRomPaths,
		listPatches,
		openConsoleWindow,
		setRomPath
	} from './api';
	import type { PatchFile } from './types';
	import { pushInfoToast, pushToast } from '$lib/toast';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		/** Path to the currently-selected .archipelago seed. When set, the
		 *  panel filters patches to only those belonging to that seed. */
		selectedSeedPath?: string | null;
		/** Bumped by the parent after generate/upload to force a refresh. */
		reloadToken?: number;
	};
	let { selectedSeedPath = null, reloadToken = 0 }: Props = $props();

	let all: PatchFile[] = $state([]);
	let romPaths: Record<string, string> = $state({});
	let loading = $state(false);
	let busyExt: string | null = $state(null);
	/** Patch pending delete-confirmation. `null` = modal closed. */
	let pendingDelete: PatchFile | null = $state(null);

	async function refresh() {
		loading = true;
		try {
			const [nextPatches, nextRoms] = await Promise.all([listPatches(), getRomPaths()]);
			all = nextPatches;
			romPaths = nextRoms;
		} finally {
			loading = false;
		}
	}

	let pollHandle: ReturnType<typeof setInterval> | null = null;

	onMount(() => {
		refresh();
		pollHandle = setInterval(refresh, 3000);
		window.addEventListener('focus', refresh);
	});

	onDestroy(() => {
		if (pollHandle) clearInterval(pollHandle);
		window.removeEventListener('focus', refresh);
	});

	$effect(() => {
		// React to the external reload signal.
		if (reloadToken > 0) refresh();
	});

	const visible = $derived.by(() => {
		if (!selectedSeedPath) return all;
		const seedStem = selectedSeedPath
			.split(/[\\/]/)
			.pop()
			?.replace(/\.archipelago$/i, '');
		if (!seedStem) return all;
		return all.filter((p) => p.seed_stem === seedStem);
	});

	function fmtSize(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
	}

	async function play(patch: PatchFile) {
		try {
			// Open the console BEFORE applying the patch so the frontend
			// listener is attached by the time apply_and_launch starts
			// emitting `[zephyr] ...` diagnostic lines over bridge-log.
			// Otherwise those early events fire into the void and the user
			// only sees the later Launcher.py stdout stream.
			try {
				await openConsoleWindow();
				// Give the Console window a moment to mount + register its
				// `randomizer://bridge-log` listener. Without this pause the
				// early `[zephyr] Auto-connecting client to ...` diagnostic
				// emits before anyone is listening and the user never sees
				// it — only the later Launcher.py stdout stream survives.
				await new Promise((r) => setTimeout(r, 500));
			} catch {
				// Non-fatal — the patch will still apply; the console just
				// didn't pop up, user can open it manually.
			}
			await applyPatch(patch.path);
			pushInfoToast({
				message: m.randomizer_patches_launching({ name: patch.file_name })
			});
		} catch (err) {
			pushToast({
				type: 'error',
				name: m.randomizer_patches_applyFailed(),
				message: (err as any)?.message ?? String(err)
			});
		}
	}

	async function pickRom(ext: string) {
		const picked = await openDialog({
			filters: [{ name: 'Base ROM / EXE', extensions: ['*'] }],
			multiple: false
		});
		if (!picked || Array.isArray(picked)) return;
		busyExt = ext;
		try {
			await setRomPath(ext, picked);
			romPaths = { ...romPaths, [ext]: picked };
			await refresh();
		} finally {
			busyExt = null;
		}
	}

	async function forgetRom(ext: string) {
		busyExt = ext;
		try {
			await clearRomPath(ext);
			const next = { ...romPaths };
			delete next[ext];
			romPaths = next;
			await refresh();
		} finally {
			busyExt = null;
		}
	}

	function askDelete(patch: PatchFile) {
		pendingDelete = patch;
	}

	async function confirmDelete() {
		if (!pendingDelete) return;
		const target = pendingDelete;
		pendingDelete = null;
		try {
			await deletePatch(target.path);
		} catch {
			// already toasted
		}
		await refresh();
	}

	// Unique extensions seen in the current patch list — these are the games
	// for which the user might want to register a base ROM.
	const extensionsSeen = $derived.by(() => {
		const set = new Set<string>();
		for (const p of visible) set.add(p.extension);
		return [...set].sort();
	});
</script>

<section class="sp-panel">
	<header class="sp-header">
		<div class="sp-header-info">
			<h3>
				<Icon icon="mdi:puzzle-check" />
				{i18nState.locale && m.randomizer_patches_title()}
				{#if selectedSeedPath && visible.length !== all.length}
					<small class="sp-filter">{i18nState.locale && m.randomizer_patches_filtered()}</small>
				{/if}
			</h3>
			<p class="sp-sub">{i18nState.locale && m.randomizer_patches_intro()}</p>
		</div>
		<div class="sp-actions">
			<Button size="sm" variant="ghost" onclick={refresh} disabled={loading}>
				{#snippet icon()}<Icon icon="mdi:refresh" />{/snippet}
				{i18nState.locale && m.randomizer_patches_reload()}
			</Button>
		</div>
	</header>

	{#if extensionsSeen.length > 0}
		<div class="sp-roms">
			{#each extensionsSeen as ext (ext)}
				{@const romPath = romPaths[ext]}
				<div class="sp-rom-chip">
					<code>.{ext}</code>
					{#if romPath}
						<span class="sp-rom-path" title={romPath}>
							<Icon icon="mdi:disc" />
							{romPath.split(/[\\/]/).pop()}
						</span>
						<button
							class="sp-rom-clear"
							aria-label={i18nState.locale && m.randomizer_patches_forgetRom()}
							onclick={() => forgetRom(ext)}
							disabled={busyExt === ext}
						>
							<Icon icon="mdi:close" />
						</button>
					{:else}
						<button class="sp-rom-set" onclick={() => pickRom(ext)} disabled={busyExt === ext}>
							<Icon icon="mdi:disc-alert" />
							{i18nState.locale && m.randomizer_patches_setBaseRom()}
						</button>
					{/if}
				</div>
			{/each}
		</div>
	{/if}

	{#if visible.length === 0}
		<div class="sp-empty">
			<Icon icon="mdi:puzzle-outline" />
			<span>
				{selectedSeedPath
					? i18nState.locale && m.randomizer_patches_emptyThisSeed()
					: i18nState.locale && m.randomizer_patches_emptyAll()}
			</span>
		</div>
	{:else}
		<ul class="sp-list">
			{#each visible as patch (patch.path)}
				<li class="sp-item">
					<Icon icon="mdi:package-variant" class="sp-item-icon" />
					<div class="sp-item-body">
						<div class="sp-item-title">
							<span class="sp-item-name">{patch.player_label ?? patch.file_name}</span>
							<span class="sp-item-ext">.{patch.extension}</span>
							{#if patch.output_rom_path}
								<span class="sp-chip sp-chip-ok">
									<Icon icon="mdi:check" />
									{i18nState.locale && m.randomizer_patches_applied()}
								</span>
							{/if}
						</div>
						<div class="sp-item-meta">
							<div class="sp-filename-wrap">
								<Tooltip text={patch.file_name} position="top" delay={300} block>
									<code class="sp-filename">{patch.file_name}</code>
								</Tooltip>
							</div>
							<span class="sp-item-size">{fmtSize(patch.size)}</span>
						</div>
					</div>
					<div class="sp-item-actions">
						<Button size="sm" variant="primary" onclick={() => play(patch)}>
							{#snippet icon()}<Icon icon="mdi:play" />{/snippet}
							{i18nState.locale && m.randomizer_patches_play()}
						</Button>
						<Tooltip
							text={i18nState.locale && m.randomizer_patches_deletePatch()}
							position="top"
							delay={200}
						>
							<button
								class="sp-icon-btn"
								aria-label={i18nState.locale && m.randomizer_patches_deletePatch()}
								onclick={() => askDelete(patch)}
							>
								<Icon icon="mdi:trash-can-outline" />
							</button>
						</Tooltip>
					</div>
				</li>
			{/each}
		</ul>
	{/if}
</section>

<Modal
	open={pendingDelete !== null}
	onclose={() => (pendingDelete = null)}
	title={i18nState.locale && m.randomizer_patches_deleteTitle()}
>
	{#if pendingDelete}
		<p class="sp-modal-text">
			{i18nState.locale && m.randomizer_patches_deleteConfirm({ name: pendingDelete.file_name })}
		</p>
		<p class="sp-modal-hint">{i18nState.locale && m.randomizer_patches_deleteHint()}</p>
	{/if}
	{#snippet actions()}
		<Button variant="ghost" onclick={() => (pendingDelete = null)}>
			{i18nState.locale && m.randomizer_cancel()}
		</Button>
		<Button variant="danger" onclick={confirmDelete}>
			{#snippet icon()}<Icon icon="mdi:trash-can-outline" />{/snippet}
			{i18nState.locale && m.randomizer_patches_delete()}
		</Button>
	{/snippet}
</Modal>

<style>
	.sp-panel {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		padding: var(--space-md);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-lg);
		background: var(--bg-surface);
	}

	.sp-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: var(--space-md);
		flex-wrap: wrap;
	}

	.sp-header-info {
		flex: 1;
		min-width: 0;
	}

	.sp-header h3 {
		margin: 0;
		font-size: 14px;
		display: inline-flex;
		align-items: center;
		gap: var(--space-xs);
		color: var(--text-primary);
	}

	.sp-filter {
		font-size: 10px;
		font-weight: 400;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--accent-400);
		padding: 1px 6px;
		border-radius: var(--radius-full);
		background: var(--bg-active);
	}

	.sp-sub {
		margin: 2px 0 0;
		color: var(--text-muted);
		font-size: 11px;
		max-width: 60ch;
	}

	.sp-actions {
		display: flex;
		gap: var(--space-xs);
	}

	.sp-roms {
		display: flex;
		gap: var(--space-xs);
		flex-wrap: wrap;
	}

	.sp-rom-chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 3px 8px;
		border-radius: var(--radius-full);
		border: 1px solid var(--border-subtle);
		background: var(--bg-base);
		font-size: 11px;
	}

	.sp-rom-chip code {
		background: var(--bg-active);
		padding: 1px 6px;
		border-radius: var(--radius-sm);
		font-size: 10px;
		color: var(--accent-400);
	}

	.sp-rom-path {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		color: var(--text-secondary);
		max-width: 220px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.sp-rom-path :global(svg) {
		font-size: 12px;
		color: #65c88a;
	}

	.sp-rom-set,
	.sp-rom-clear {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		background: transparent;
		border: none;
		color: var(--accent-400);
		cursor: pointer;
		font-size: 11px;
		padding: 0 4px;
	}

	.sp-rom-set :global(svg),
	.sp-rom-clear :global(svg) {
		font-size: 13px;
	}

	.sp-rom-clear {
		color: var(--text-muted);
	}
	.sp-rom-clear:hover {
		color: #ef5555;
	}

	.sp-list {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.sp-item {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-xs) var(--space-sm);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		background: var(--bg-base);
	}

	.sp-item :global(.sp-item-icon) {
		font-size: 20px;
		color: var(--accent-400);
		flex-shrink: 0;
	}

	.sp-item-body {
		flex: 1;
		min-width: 0;
	}

	.sp-item-title {
		font-weight: 600;
		color: var(--text-primary);
		font-size: 13px;
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		min-width: 0;
	}

	.sp-item-ext {
		font-size: 10px;
		font-family: var(--font-mono, monospace);
		color: var(--text-muted);
		font-weight: 400;
	}

	.sp-item-meta {
		margin-top: 2px;
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-size: 11px;
		color: var(--text-muted);
		min-width: 0;
	}

	.sp-item-meta code {
		background: var(--bg-active);
		padding: 1px 6px;
		border-radius: var(--radius-sm);
		font-size: 11px;
	}

	.sp-filename-wrap {
		flex: 1 1 0;
		min-width: 0;
		overflow: hidden;
	}

	.sp-filename {
		display: block;
		width: 100%;
		max-width: 100%;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		min-width: 0;
	}

	.sp-item-size {
		flex-shrink: 0;
	}

	.sp-item-name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		min-width: 0;
	}

	.sp-item-actions {
		flex-shrink: 0;
	}

	.sp-chip {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		padding: 1px 6px;
		border-radius: var(--radius-full);
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
	}
	.sp-chip :global(svg) {
		font-size: 11px;
	}
	.sp-chip-ok {
		background: rgba(80, 200, 120, 0.12);
		color: #65c88a;
	}

	.sp-item-actions {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
	}

	.sp-icon-btn {
		background: transparent;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		border-radius: var(--radius-sm);
		padding: 6px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.sp-icon-btn:hover {
		color: #ef5555;
		background: var(--bg-active);
	}

	.sp-icon-btn :global(svg) {
		font-size: 16px;
	}

	.sp-empty {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm);
		color: var(--text-muted);
		font-size: 12px;
	}

	.sp-empty :global(svg) {
		font-size: 18px;
		opacity: 0.7;
	}

	.sp-modal-text {
		margin: 0 0 var(--space-xs);
		font-size: 13px;
		color: var(--text-primary);
	}

	.sp-modal-text code {
		background: var(--bg-active);
		padding: 2px 6px;
		border-radius: var(--radius-sm);
		font-size: 12px;
	}

	.sp-modal-hint {
		margin: 0;
		font-size: 11px;
		color: var(--text-muted);
	}
</style>
