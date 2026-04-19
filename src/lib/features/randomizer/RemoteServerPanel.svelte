<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import * as api from './api';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		remote: api.RemoteStatus | null;
		selectedSeed: string | null;
		uploading: boolean;
		remoteStarting: boolean;
		remoteLog: string[];
		copiedKey: string | null;
		onCopyAddr: () => void;
		onCopyLog: () => void;
		onUploadAndStart: () => void;
	};

	const {
		remote,
		selectedSeed,
		uploading,
		remoteStarting,
		remoteLog,
		copiedKey,
		onCopyAddr,
		onCopyLog,
		onUploadAndStart
	}: Props = $props();

	const REMOTE_ADDRESS = 'nozomi.proxy.rlwy.net:45465';
</script>

<div class="rdz-remote-host">
	{#if remote?.running}
		<div class="rdz-running-line">
			<span class="rdz-live-dot"></span>
			<span>{i18nState.locale && m.randomizer_running()}</span>
			<code>{remote.seed}</code>
		</div>
		<button class="rdz-conn-card" onclick={onCopyAddr}>
			<span class="rdz-label"
				><Icon icon="mdi:cloud" /> {i18nState.locale && m.randomizer_connectAddress()}</span
			>
			<code>{REMOTE_ADDRESS}</code>
			<small
				>{copiedKey === 'addr'
					? i18nState.locale && m.randomizer_copiedExcl()
					: i18nState.locale && m.randomizer_clickToCopy()}</small
			>
		</button>
	{:else}
		<p class="rdz-muted">
			{#if !selectedSeed}
				{i18nState.locale && m.randomizer_selectSeed()}
			{:else}
				{i18nState.locale && m.randomizer_readyToUpload()}
			{/if}
		</p>
		<Button
			variant="primary"
			disabled={!selectedSeed || uploading || remoteStarting}
			loading={uploading || remoteStarting}
			onclick={onUploadAndStart}
		>
			{#snippet icon()}<Icon icon="mdi:cloud-upload" />{/snippet}
			{uploading
				? i18nState.locale && m.randomizer_uploading()
				: remoteStarting
					? i18nState.locale && m.randomizer_starting()
					: i18nState.locale && m.randomizer_uploadAndStart()}
		</Button>
	{/if}
	{#if remoteLog.length > 0}
		<details class="rdz-log-details" open>
			<summary>
				<span>{i18nState.locale && m.randomizer_remoteLog()}</span>
				<button
					class="rdz-log-copy"
					onclick={(e) => {
						e.preventDefault();
						onCopyLog();
					}}
				>
					<Icon icon={copiedKey === 'remote' ? 'mdi:check' : 'mdi:content-copy'} />
					{copiedKey === 'remote'
						? i18nState.locale && m.randomizer_copied()
						: i18nState.locale && m.randomizer_copy()}
				</button>
			</summary>
			<pre class="rdz-log">{remoteLog.join('\n')}</pre>
		</details>
	{/if}
</div>

<style>
	.rdz-remote-host {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.rdz-running-line {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		font-size: 12px;
		color: var(--text-secondary);
		padding: 2px 2px 6px;
		flex-wrap: wrap;
	}

	.rdz-live-dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: var(--success, var(--accent-400));
		box-shadow: 0 0 6px color-mix(in srgb, var(--success, var(--accent-400)) 50%, transparent);
		margin-top: 4px;
		flex-shrink: 0;
	}

	.rdz-running-line > span:not(.rdz-live-dot) {
		flex-shrink: 0;
	}

	.rdz-running-line code {
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		color: var(--text-primary);
		background: transparent;
		padding: 0;
		font-size: 12px;
		flex: 1 1 100%;
		min-width: 0;
		overflow-wrap: anywhere;
		word-break: break-all;
		line-height: 1.4;
	}

	.rdz-conn-card {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: var(--space-sm) var(--space-md);
		background: var(--bg-active);
		border: 1px solid var(--border-accent);
		border-radius: var(--radius-md);
		text-align: left;
		cursor: pointer;
		transition: all var(--transition-fast);
		font-family: inherit;
		color: inherit;
		width: 100%;
	}

	.rdz-conn-card:hover {
		border-color: var(--accent-400);
		box-shadow: var(--shadow-glow);
	}

	.rdz-conn-card code {
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		font-size: 13px;
		font-weight: 700;
		color: var(--accent-400);
		background: transparent;
		padding: 0;
		overflow-wrap: anywhere;
		word-break: break-all;
	}

	.rdz-conn-card small {
		font-size: 10px;
		color: var(--text-muted);
		line-height: 1.3;
	}

	.rdz-label {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 9px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-muted);
	}

	.rdz-label :global(svg) {
		font-size: 11px;
	}

	.rdz-muted {
		margin: 0;
		font-size: 12px;
		color: var(--text-muted);
	}

	.rdz-log-details {
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		background: var(--bg-base);
		padding: var(--space-sm);
		font-size: 11px;
	}

	.rdz-log-details summary {
		display: flex;
		align-items: center;
		justify-content: space-between;
		cursor: pointer;
		color: var(--text-muted);
		list-style: none;
	}

	.rdz-log-details summary::-webkit-details-marker {
		display: none;
	}

	.rdz-log-copy {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		background: transparent;
		border: 1px solid var(--border-default);
		border-radius: var(--radius-sm);
		padding: 2px 8px;
		color: var(--text-muted);
		font-size: 10px;
		cursor: pointer;
	}

	.rdz-log-copy:hover {
		border-color: var(--accent-400);
		color: var(--accent-400);
	}

	.rdz-log {
		margin: var(--space-xs) 0 0;
		max-height: 200px;
		overflow: auto;
		white-space: pre-wrap;
		word-break: break-word;
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		font-size: 11px;
		color: var(--text-secondary);
	}
</style>
