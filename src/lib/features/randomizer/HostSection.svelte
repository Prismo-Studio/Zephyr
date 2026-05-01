<script lang="ts">
	import Icon from '@iconify/svelte';
	import RemoteServerPanel from './RemoteServerPanel.svelte';
	import LocalServerRunningView from './LocalServerRunningView.svelte';
	import LocalServerForm from './LocalServerForm.svelte';
	import ServerLogDetails from './ServerLogDetails.svelte';
	import * as api from './api';
	import type { PythonStatus, ServerStatus } from './types';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type ConnTarget = 'local' | 'lan' | 'public';

	let {
		hostMode = $bindable(),
		connTarget = $bindable(),
		portStr = $bindable(),
		password = $bindable(),
		port,
		portValid,
		python,
		server,
		selectedSeed,
		starting,
		copiedKey,
		remoteRoom,
		uploading,
		remoteStarting,
		remoteLog,
		PRESET_PORTS,
		onCopyText,
		onCopyRemoteLog,
		onUploadAndStartRemote,
		onClearRemoteRoom,
		onStartHost,
		onStop
	}: {
		hostMode: 'local' | 'remote';
		connTarget: ConnTarget;
		portStr: string;
		password: string;
		port: number;
		portValid: boolean;
		python: PythonStatus | null;
		server: ServerStatus | null;
		selectedSeed: string | null;
		starting: boolean;
		copiedKey: string | null;
		remoteRoom: api.ArchipelagoGgRoom | null;
		uploading: boolean;
		remoteStarting: boolean;
		remoteLog: string[];
		PRESET_PORTS: { p: number; label: string }[];
		onCopyText: (text: string, key: string) => void;
		onCopyRemoteLog: () => void;
		onUploadAndStartRemote: () => void;
		onClearRemoteRoom: () => void;
		onStartHost: () => void;
		onStop: () => void;
	} = $props();
</script>

<div class="rdz-block-body">
	<div class="rdz-host-toggle">
		<button
			class="rdz-toggle-btn"
			class:active={hostMode === 'remote'}
			onclick={() => (hostMode = 'remote')}
		>
			<Icon icon="mdi:cloud" />
			{i18nState.locale && m.randomizer_remote()}
		</button>
		<button
			class="rdz-toggle-btn"
			class:active={hostMode === 'local'}
			onclick={() => (hostMode = 'local')}
		>
			<Icon icon="mdi:laptop" />
			{i18nState.locale && m.randomizer_local()}
		</button>
	</div>

	{#if hostMode === 'remote'}
		<RemoteServerPanel
			{remoteRoom}
			{selectedSeed}
			{uploading}
			{remoteStarting}
			{remoteLog}
			{copiedKey}
			onCopyAddr={(text, key) => onCopyText(text, key)}
			onCopyLog={onCopyRemoteLog}
			onUploadAndStart={onUploadAndStartRemote}
			onClearRoom={onClearRemoteRoom}
		/>
	{:else}
		{#if server?.running}
			<LocalServerRunningView
				{server}
				bind:connTarget
				{portStr}
				{copiedKey}
				{PRESET_PORTS}
				oncopy={onCopyText}
				onstop={onStop}
			/>
		{:else}
			<LocalServerForm
				bind:portStr
				bind:password
				{port}
				{portValid}
				{python}
				{selectedSeed}
				{starting}
				{PRESET_PORTS}
				onstart={onStartHost}
			/>
		{/if}

		{#if server?.recent_log && server.recent_log.length > 0}
			<ServerLogDetails
				lines={server.recent_log}
				{copiedKey}
				oncopy={() => onCopyText(server?.recent_log.join('\n') ?? '', 'srv')}
			/>
		{/if}
	{/if}
</div>

<style>
	.rdz-block-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		padding: 0;
		animation: rdz-block-in 180ms ease;
	}

	@keyframes rdz-block-in {
		from {
			opacity: 0;
			transform: translateY(-2px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.rdz-host-toggle {
		display: flex;
		gap: 2px;
		padding: 2px;
		background: transparent;
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
	}

	.rdz-toggle-btn {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 5px 12px;
		border: none;
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition:
			color var(--transition-fast),
			background var(--transition-fast);
	}

	.rdz-toggle-btn:hover {
		color: var(--text-secondary);
	}

	.rdz-toggle-btn.active {
		background: var(--bg-hover);
		color: var(--text-primary);
		box-shadow: none;
	}
</style>
