<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { onMount } from 'svelte';
	import { randomizerStore } from './randomizer.store.svelte';
	import { runtimeInstallStore } from './runtimeInstall.svelte';
	import { pushToast } from '$lib/toast';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	const store = runtimeInstallStore;
	const status = $derived(store.status);
	const loading = $derived(store.loading);
	const installing = $derived(store.installing);
	const progress = $derived(store.progress);

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
				return m.randomizer_runtime_progress_downloading({ got, total });
			}
			case 'extracting':
				return m.randomizer_runtime_progress_extracting({
					done: String(progress.done),
					total: String(progress.total)
				});
			case 'provisioning_venv':
				return progress.message;
			case 'installing_deps':
				return progress.message;
			case 'installed':
				return m.randomizer_runtime_progress_installed();
			case 'failed':
				return m.randomizer_runtime_progress_failed({ error: progress.error });
		}
	});

	const progressIsIndeterminate = $derived.by(() => {
		if (!progress) return false;
		return progress.stage === 'provisioning_venv' || progress.stage === 'installing_deps';
	});

	async function install() {
		try {
			await store.install();
			await randomizerStore.loadCatalog();
		} catch (err) {
			console.error(err);
			pushToast({
				type: 'error',
				name: m.randomizer_runtime_installFailed(),
				message: (err as { message?: string })?.message ?? String(err)
			});
		}
	}

	async function provisionDeps() {
		try {
			await store.provisionDeps();
			await randomizerStore.loadCatalog();
		} catch (err) {
			console.error(err);
			pushToast({
				type: 'error',
				name: m.randomizer_runtime_depsInstallFailed(),
				message: (err as { message?: string })?.message ?? String(err)
			});
		}
	}

	onMount(async () => {
		await store.startListener();
		if (store.status === null) {
			await store.refresh();
		}
	});
</script>

{#snippet progressBlock()}
	{#if installing && progress}
		<div class="rim-progress">
			<div class="rim-progress-bar">
				<div
					class="rim-progress-fill"
					style:width={progressIsIndeterminate || progressPercent == null
						? '100%'
						: `${progressPercent}%`}
					class:rim-indeterminate={progressIsIndeterminate || progressPercent == null}
				></div>
			</div>
			<span class="rim-progress-label">
				{progressLabel}
				{#if !progressIsIndeterminate && progressPercent != null}· {progressPercent}%{/if}
			</span>
		</div>
	{/if}
{/snippet}

{#if !loading && status && !status.installed}
	<div
		class="rim-banner"
		role="region"
		aria-label={i18nState.locale && m.randomizer_runtime_notInstalled_title()}
	>
		<Icon icon="mdi:cloud-download-outline" />
		<div class="rim-body">
			<strong>{i18nState.locale && m.randomizer_runtime_notInstalled_title()}</strong>
			<small>{i18nState.locale && m.randomizer_runtime_notInstalled_desc()}</small>
			{@render progressBlock()}
		</div>
		<div class="rim-actions">
			<Button size="md" variant="primary" onclick={install} disabled={installing}>
				{#snippet icon()}<Icon
						icon={installing ? 'mdi:loading' : 'mdi:download'}
						class={installing ? 'rim-spin' : ''}
					/>{/snippet}
				{installing
					? i18nState.locale && m.randomizer_runtime_installing()
					: i18nState.locale && m.randomizer_runtime_download()}
			</Button>
		</div>
	</div>
{:else if !loading && status && status.installed && !status.venv_ready}
	<div
		class="rim-banner rim-warn"
		role="region"
		aria-label={i18nState.locale && m.randomizer_runtime_venvMissing_title()}
	>
		<Icon icon="mdi:alert-circle-outline" />
		<div class="rim-body">
			<strong>{i18nState.locale && m.randomizer_runtime_venvMissing_title()}</strong>
			<small>{i18nState.locale && m.randomizer_runtime_venvMissing_desc()}</small>
			{@render progressBlock()}
		</div>
		<div class="rim-actions">
			<Button size="md" variant="primary" onclick={provisionDeps} disabled={installing}>
				{#snippet icon()}<Icon
						icon={installing ? 'mdi:loading' : 'mdi:language-python'}
						class={installing ? 'rim-spin' : ''}
					/>{/snippet}
				{installing
					? i18nState.locale && m.randomizer_runtime_installing()
					: i18nState.locale && m.randomizer_runtime_installDeps()}
			</Button>
		</div>
	</div>
{:else if !loading && status && status.installed}
	<div class="rim-ok">
		<Icon icon="mdi:check-circle-outline" />
		<span>{i18nState.locale && m.randomizer_runtime_installed()} ·</span>
		<code>{status.path}</code>
		<span class="rim-ok-meta">
			{i18nState.locale && m.randomizer_runtime_worlds({ count: String(status.world_count) })} · {formatSize(
				status.bytes_on_disk
			)}
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

	.rim-banner.rim-warn {
		border-color: #f0b43c;
		background: rgba(240, 180, 60, 0.08);
	}
	.rim-banner.rim-warn > :global(svg) {
		color: #f0b43c;
	}
	.rim-banner code {
		background: var(--bg-surface);
		padding: 1px 5px;
		border-radius: var(--radius-sm);
		font-size: 11px;
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
		background: linear-gradient(90deg, transparent 0%, var(--accent-400) 50%, transparent 100%);
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
		gap: 6px;
		padding: 4px 0;
		font-size: 11px;
		color: var(--text-muted);
	}

	.rim-ok :global(svg) {
		font-size: 14px;
		color: #65c88a;
		flex-shrink: 0;
	}

	.rim-ok code {
		background: transparent;
		padding: 0;
		font-family: var(--font-mono, monospace);
		font-size: 10px;
		color: var(--text-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		min-width: 0;
	}

	.rim-ok-meta {
		margin-left: auto;
		flex-shrink: 0;
		white-space: nowrap;
	}
</style>
