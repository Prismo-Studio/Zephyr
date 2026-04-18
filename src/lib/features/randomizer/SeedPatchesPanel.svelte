<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import { onMount } from 'svelte';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';
	import {
		applyPatch,
		clearRomPath,
		deletePatch,
		getRomPaths,
		launchApComponent,
		listPatches,
		setRomPath
	} from './api';
	import type { PatchFile } from './types';
	import { pushInfoToast, pushToast } from '$lib/toast';

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

	async function refresh() {
		loading = true;
		try {
			[all, romPaths] = await Promise.all([listPatches(), getRomPaths()]);
		} finally {
			loading = false;
		}
	}

	onMount(refresh);

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
			await applyPatch(patch.path);
			pushInfoToast({ message: `Launching ${patch.file_name}…` });
		} catch (err) {
			pushToast({
				type: 'error',
				name: 'Apply failed',
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

	async function removePatch(patch: PatchFile) {
		if (!confirm(`Delete ${patch.file_name}?`)) return;
		await deletePatch(patch.path);
		await refresh();
	}

	async function launchTextClient() {
		try {
			await launchApComponent('Text Client');
		} catch (err) {
			pushToast({
				type: 'error',
				name: 'Launch failed',
				message: (err as any)?.message ?? String(err)
			});
		}
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
		<div>
			<h3>
				<Icon icon="mdi:puzzle-check" />
				Patches & clients
				{#if selectedSeedPath && visible.length !== all.length}
					<small class="sp-filter">filtered to current seed</small>
				{/if}
			</h3>
			<p class="sp-sub">
				Per-player patch files extracted from the generated seed. Click <strong>Play</strong>
				to apply a patch and launch its custom client.
			</p>
		</div>
		<div class="sp-actions">
			<Tooltip text="Universal server chat client" position="top" delay={200}>
				<Button size="sm" variant="ghost" onclick={launchTextClient}>
					{#snippet icon()}<Icon icon="mdi:console" />{/snippet}
					Text Client
				</Button>
			</Tooltip>
			<Button size="sm" variant="ghost" onclick={refresh} disabled={loading}>
				{#snippet icon()}<Icon icon="mdi:refresh" />{/snippet}
				Reload
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
							<Icon icon="mdi:disc" /> {romPath.split(/[\\/]/).pop()}
						</span>
						<button
							class="sp-rom-clear"
							aria-label="Forget base ROM"
							onclick={() => forgetRom(ext)}
							disabled={busyExt === ext}
						>
							<Icon icon="mdi:close" />
						</button>
					{:else}
						<button
							class="sp-rom-set"
							onclick={() => pickRom(ext)}
							disabled={busyExt === ext}
						>
							<Icon icon="mdi:disc-alert" />
							Set base ROM
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
					? 'This seed has no patch files — game is server-only.'
					: 'No patches yet. Generate a seed for a game that requires patching.'}
			</span>
		</div>
	{:else}
		<ul class="sp-list">
			{#each visible as patch (patch.path)}
				<li class="sp-item">
					<Icon icon="mdi:package-variant" class="sp-item-icon" />
					<div class="sp-item-body">
						<div class="sp-item-title">
							{patch.player_label ?? patch.file_name}
							<span class="sp-item-ext">.{patch.extension}</span>
							{#if patch.output_rom_path}
								<span class="sp-chip sp-chip-ok">
									<Icon icon="mdi:check" /> applied
								</span>
							{/if}
						</div>
						<div class="sp-item-meta">
							<code>{patch.file_name}</code>
							<span>{fmtSize(patch.size)}</span>
						</div>
					</div>
					<div class="sp-item-actions">
						<Button size="sm" variant="primary" onclick={() => play(patch)}>
							{#snippet icon()}<Icon icon="mdi:play" />{/snippet}
							Play
						</Button>
						<Tooltip text="Delete patch" position="top" delay={200}>
							<button
								class="sp-icon-btn"
								aria-label="Delete patch"
								onclick={() => removePatch(patch)}
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
	}

	.sp-item-meta code {
		background: var(--bg-active);
		padding: 1px 6px;
		border-radius: var(--radius-sm);
		font-size: 11px;
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
</style>
