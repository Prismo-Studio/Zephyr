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

	.rdz-remote-host :global(.rdz-running-line code) {
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
</style>
