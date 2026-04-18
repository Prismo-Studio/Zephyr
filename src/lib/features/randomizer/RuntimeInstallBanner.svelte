<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { onDestroy, onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { installRuntime, runtimeStatus } from './api';
	import type { RuntimeProgress, RuntimeStatus } from './types';
	import { randomizerStore } from './randomizer.store.svelte';
	import { pushToast } from '$lib/toast';

	let status: RuntimeStatus | null = $state(null);
	let loading = $state(true);
	let installing = $state(false);
	let progress: RuntimeProgress | null = $state(null);
	let unlisten: UnlistenFn | null = null;

	function formatSize(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
		return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
	}

	const progressPercent = $derived.by(() => {
		if (!progress) return null;
		if (progress.stage === 'downloading' && progress.total) {
			return Math.min(100, Math.round((progress.received / progress.total) * 100));
		}
		if (progress.stage === 'extracting' && progress.total) {
			return Math.min(100, Math.round((progress.done / progress.total) * 100));
		}
		if (progress.stage === 'installed') return 100;
		return null;
	});

	const progressLabel = $derived.by(() => {
		if (!progress) return '';
		switch (progress.stage) {
			case 'downloading': {
				const got = formatSize(progress.received);
				const total = progress.total ? ` / ${formatSize(progress.total)}` : '';
				return `Downloading ${got}${total}`;
			}
			case 'extracting':
				return `Extracting ${progress.done}/${progress.total}`;
			case 'installed':
				return 'Installed';
			case 'failed':
				return `Failed: ${progress.error}`;
		}
	});

	async function refresh() {
		loading = true;
		try {
			status = await runtimeStatus();
		} finally {
			loading = false;
		}
	}

	async function install() {
		if (installing) return;
		installing = true;
		progress = null;
		try {
			status = await installRuntime();
			await randomizerStore.loadCatalog();
		} catch (err) {
			console.error(err);
			pushToast({
				type: 'error',
				name: 'Runtime install failed',
				message: (err as any)?.message ?? String(err)
			});
		} finally {
			installing = false;
		}
	}

	onMount(async () => {
		await refresh();
		const un = await listen<RuntimeProgress>('randomizer://runtime-progress', (event) => {
			progress = event.payload;
		});
		unlisten = un;
	});

	onDestroy(() => {
		unlisten?.();
	});
</script>

{#if !loading && status && !status.installed}
	<div class="rim-banner" role="region" aria-label="Archipelago runtime not installed">
		<Icon icon="mdi:cloud-download-outline" />
		<div class="rim-body">
			<strong>Archipelago runtime not installed</strong>
			<small>
				Zephyr can generate seeds locally, but it needs the Archipelago Python runtime. Releases
				ship without it to keep downloads small.
			</small>
			{#if installing && progress}
				<div class="rim-progress">
					<div class="rim-progress-bar">
						<div
							class="rim-progress-fill"
							style:width={progressPercent == null ? '100%' : `${progressPercent}%`}
							class:rim-indeterminate={progressPercent == null}
						></div>
					</div>
					<span class="rim-progress-label">
						{progressLabel}
						{#if progressPercent != null}· {progressPercent}%{/if}
					</span>
				</div>
			{/if}
		</div>
		<div class="rim-actions">
			<Button size="md" variant="primary" onclick={install} disabled={installing}>
				{#snippet icon()}<Icon
						icon={installing ? 'mdi:loading' : 'mdi:download'}
						class={installing ? 'rim-spin' : ''}
					/>{/snippet}
				{installing ? 'Installing…' : 'Download & install'}
			</Button>
		</div>
	</div>
{:else if !loading && status && status.installed}
	<!-- show a compact chip so the user knows where the runtime lives -->
	<div class="rim-ok">
		<Icon icon="mdi:check-circle-outline" />
		<span>Archipelago runtime installed ·</span>
		<code>{status.path}</code>
		<span class="rim-ok-meta">
			{status.world_count} worlds · {formatSize(status.bytes_on_disk)}
		</span>
	</div>
{/if}

<style>
	.rim-banner {
		display: flex;
		gap: var(--space-md);
		align-items: flex-start;
		padding: var(--space-md);
		border: 1px solid var(--accent-400);
		border-radius: var(--radius-lg);
		background: var(--bg-active);
		color: var(--text-primary);
	}

	.rim-banner > :global(svg) {
		font-size: 28px;
		color: var(--accent-400);
		flex-shrink: 0;
	}

	.rim-body {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.rim-body strong {
		font-size: 13px;
	}

	.rim-body small {
		color: var(--text-muted);
		font-size: 11px;
		max-width: 70ch;
	}

	.rim-actions {
		display: flex;
		align-items: center;
	}

	.rim-progress {
		margin-top: var(--space-sm);
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.rim-progress-bar {
		height: 6px;
		background: var(--bg-surface);
		border-radius: var(--radius-full);
		overflow: hidden;
		position: relative;
	}

	.rim-progress-fill {
		height: 100%;
		background: var(--accent-400);
		transition: width 120ms linear;
	}

	.rim-indeterminate {
		background: linear-gradient(
			90deg,
			transparent 0%,
			var(--accent-400) 50%,
			transparent 100%
		);
		background-size: 200% 100%;
		animation: rim-slide 1.4s ease-in-out infinite;
	}

	@keyframes rim-slide {
		from {
			background-position: 200% 0;
		}
		to {
			background-position: -200% 0;
		}
	}

	.rim-progress-label {
		font-size: 11px;
		color: var(--text-muted);
		font-family: var(--font-mono, monospace);
	}

	:global(.rim-spin) {
		animation: rim-spin 1s linear infinite;
	}

	@keyframes rim-spin {
		to {
			transform: rotate(360deg);
		}
	}

	.rim-ok {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		padding: 6px 10px;
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		font-size: 11px;
		color: var(--text-muted);
	}

	.rim-ok :global(svg) {
		font-size: 14px;
		color: #65c88a;
	}

	.rim-ok code {
		background: var(--bg-base);
		padding: 1px 6px;
		border-radius: var(--radius-sm);
		font-family: var(--font-mono, monospace);
		font-size: 10px;
		color: var(--text-secondary);
	}

	.rim-ok-meta {
		margin-left: auto;
	}
</style>
