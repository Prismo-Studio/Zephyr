<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Dropdown from '$lib/components/ui/Dropdown.svelte';
	import RemoteServerPanel from './RemoteServerPanel.svelte';
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
			{@const selectedIp =
				connTarget === 'local'
					? '127.0.0.1'
					: connTarget === 'lan'
						? (server.local_ip ?? '127.0.0.1')
						: (server.public_ip ?? '127.0.0.1')}
			{@const selectedAddr = `${selectedIp}:${server.port}`}
			{@const selectedIcon =
				connTarget === 'local' ? 'mdi:laptop' : connTarget === 'lan' ? 'mdi:lan' : 'mdi:earth'}
			{@const selectedLabel =
				connTarget === 'local'
					? (i18nState.locale && m.randomizer_thisMachine()) || 'Cette machine'
					: connTarget === 'lan'
						? (i18nState.locale && m.randomizer_lanTailscale()) || 'LAN / Tailscale'
						: (i18nState.locale && m.randomizer_publicInternet()) || 'Internet public'}
			{@const selectedDesc =
				connTarget === 'local'
					? (i18nState.locale && m.randomizer_thisMachineDesc()) || ''
					: connTarget === 'lan'
						? (i18nState.locale && m.randomizer_lanDesc()) || ''
						: (i18nState.locale &&
								m.randomizer_publicDesc({ port: (server.port ?? 0).toString() })) ||
							''}
			{@const connOptions = [
				{
					value: 'local',
					label: (i18nState.locale && m.randomizer_thisMachine()) || 'Cette machine'
				},
				...(server.local_ip
					? [
							{
								value: 'lan',
								label: (i18nState.locale && m.randomizer_lanTailscale()) || 'LAN / Tailscale'
							}
						]
					: []),
				...(server.public_ip
					? [
							{
								value: 'public',
								label: (i18nState.locale && m.randomizer_publicInternet()) || 'Internet public'
							}
						]
					: [])
			]}
			<div class="rdz-server-running">
				<div class="rdz-running-line">
					<span class="rdz-live-dot"></span>
					<span
						>{i18nState.locale &&
							m.randomizer_runningOnPort({ port: (server.port ?? 0).toString() })}</span
					>
				</div>

				<div class="rdz-conn-picker">
					<Dropdown
						options={connOptions}
						value={connTarget}
						onchange={(v) => (connTarget = v as ConnTarget)}
					/>
					<button
						class="rdz-conn-card rdz-conn-recommended"
						title={i18nState.locale && m.randomizer_clickToCopy()}
						onclick={() => onCopyText(selectedAddr, 'addr')}
					>
						<span class="rdz-label">
							<Icon icon={selectedIcon} />
							{selectedLabel}
							<span class="rdz-conn-copy-hint">
								<Icon icon={copiedKey === 'addr' ? 'mdi:check' : 'mdi:content-copy'} />
							</span>
						</span>
						<code>{selectedAddr}</code>
						{#if selectedDesc}
							<small>{selectedDesc}</small>
						{/if}
					</button>
				</div>

				{#if server.password}
					<div class="rdz-conn-line">
						<span class="rdz-label">{i18nState.locale && m.randomizer_password()}</span>
						<code>{server.password}</code>
					</div>
				{/if}
				<div class="rdz-conn-line">
					<span class="rdz-label">{i18nState.locale && m.randomizer_pid()}</span>
					<code>{server.pid}</code>
				</div>

				<div class="rdz-inline-field rdz-running-port" aria-disabled="true">
					<span>{i18nState.locale && m.randomizer_port()}</span>
					<div class="rdz-inline-control">
						<Input value={String(server.port ?? portStr)} placeholder="38281" />
					</div>
					<div class="rdz-port-presets">
						{#each PRESET_PORTS as preset}
							<button
								type="button"
								class="rdz-preset-chip"
								class:active={server.port === preset.p}
								disabled
							>
								<span class="rdz-preset-port">{preset.p}</span>
								<span class="rdz-preset-label">{preset.label}</span>
							</button>
						{/each}
					</div>
					<p class="rdz-muted rdz-running-hint">
						<Icon icon="mdi:information-outline" />
						Arrêtez le serveur pour changer le port.
					</p>
				</div>

				<div class="rdz-server-actions">
					<Button variant="primary" onclick={() => api.openConsoleWindow()}>
						{#snippet icon()}<Icon icon="mdi:console" />{/snippet}
						{i18nState.locale && m.randomizer_openConsole()}
					</Button>
					<Button variant="danger" onclick={onStop}>
						{#snippet icon()}<Icon icon="mdi:stop" />{/snippet}
						{i18nState.locale && m.randomizer_stopServer()}
					</Button>
				</div>
			</div>
		{:else}
			<div class="rdz-host-form">
				<div class="rdz-inline-field">
					<span>{i18nState.locale && m.randomizer_port()}</span>
					<div class="rdz-inline-control">
						<Input bind:value={portStr} placeholder="38281" />
					</div>
					<div class="rdz-port-presets">
						{#each PRESET_PORTS as preset}
							<button
								type="button"
								class="rdz-preset-chip"
								class:active={port === preset.p}
								onclick={() => (portStr = String(preset.p))}
							>
								<span class="rdz-preset-port">{preset.p}</span>
								<span class="rdz-preset-label">{preset.label}</span>
							</button>
						{/each}
					</div>
				</div>
				<div class="rdz-inline-field">
					<span>{i18nState.locale && m.randomizer_passwordOptional()}</span>
					<div class="rdz-inline-control">
						<Input bind:value={password} placeholder={i18nState.locale && m.randomizer_none()} />
					</div>
				</div>
				<Button
					variant="primary"
					disabled={!selectedSeed || starting || !python?.available || !portValid}
					onclick={onStartHost}
					loading={starting}
				>
					{#snippet icon()}<Icon icon="mdi:play" />{/snippet}
					{i18nState.locale && m.randomizer_startServer()}
				</Button>
			</div>
			{#if !portValid}
				<p class="rdz-warn-text">{i18nState.locale && m.randomizer_portInvalid()}</p>
			{/if}
			{#if !selectedSeed}
				<p class="rdz-muted">{i18nState.locale && m.randomizer_selectSeedFirst()}</p>
			{:else}
				<p class="rdz-muted">
					{i18nState.locale && m.randomizer_willHost()}
					<strong>{selectedSeed.split(/[/\\]/).pop()}</strong>
				</p>
			{/if}

			<div class="rdz-cgnat-note">
				<Icon icon="mdi:information-outline" />
				<div class="rdz-cgnat-content">
					<strong>{i18nState.locale && m.randomizer_cantConnect()}</strong>
					<span>{i18nState.locale && m.randomizer_cgnatExplanation()}</span>
				</div>
			</div>
		{/if}

		{#if server?.recent_log && server.recent_log.length > 0}
			<details class="rdz-log-details" open>
				<summary>
					<span
						>{i18nState.locale && m.randomizer_serverLog()} ({server.recent_log.length} lines)</span
					>
					<button
						class="rdz-log-copy"
						title={i18nState.locale && m.randomizer_copy()}
						onclick={(e) => {
							e.preventDefault();
							onCopyText(server?.recent_log.join('\n') ?? '', 'srv');
						}}
					>
						<Icon icon={copiedKey === 'srv' ? 'mdi:check' : 'mdi:content-copy'} />
						{copiedKey === 'srv'
							? i18nState.locale && m.randomizer_copied()
							: i18nState.locale && m.randomizer_copy()}
					</button>
				</summary>
				<pre class="rdz-log">{server.recent_log.join('\n')}</pre>
			</details>
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

	.rdz-muted {
		margin: 0;
		color: var(--text-muted);
		font-size: 12px;
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

	.rdz-running-line > .rdz-live-dot {
		margin-top: 4px;
		flex-shrink: 0;
	}

	.rdz-running-line > span:not(.rdz-live-dot) {
		flex-shrink: 0;
	}

	.rdz-live-dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: var(--success, var(--accent-400));
		box-shadow: 0 0 0 0 color-mix(in srgb, var(--success, var(--accent-400)) 60%, transparent);
		animation: rdz-live-pulse 2s ease-in-out infinite;
		flex-shrink: 0;
	}

	@keyframes rdz-live-pulse {
		0%,
		100% {
			box-shadow: 0 0 0 0 color-mix(in srgb, var(--success, var(--accent-400)) 50%, transparent);
		}
		50% {
			box-shadow: 0 0 0 5px transparent;
		}
	}

	.rdz-conn-picker {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.rdz-conn-copy-hint {
		margin-left: auto;
		display: inline-flex;
		align-items: center;
	}

	.rdz-conn-copy-hint :global(svg) {
		font-size: 14px;
		color: var(--text-muted);
		transition: color var(--transition-fast);
	}

	.rdz-conn-card:hover .rdz-conn-copy-hint :global(svg) {
		color: var(--accent-400);
	}

	.rdz-host-form {
		display: flex;
		gap: var(--space-md);
		align-items: flex-end;
		flex-wrap: wrap;
	}

	.rdz-inline-field {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.rdz-inline-field span {
		font-size: 9px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-muted);
	}

	.rdz-inline-control {
		width: 160px;
	}

	.rdz-port-presets {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		margin-top: 4px;
	}

	.rdz-preset-chip {
		display: inline-flex;
		align-items: baseline;
		gap: 6px;
		padding: 3px 8px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-subtle);
		background: var(--bg-base);
		color: var(--text-muted);
		cursor: pointer;
		transition:
			color var(--transition-fast),
			background var(--transition-fast),
			border-color var(--transition-fast);
	}

	.rdz-preset-port {
		font-family: var(--font-mono, 'JetBrains Mono', monospace);
		font-size: 11px;
		font-weight: 600;
		color: var(--text-secondary);
	}

	.rdz-preset-label {
		font-size: 10px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.rdz-preset-chip:hover {
		background: var(--bg-hover);
		border-color: var(--border-default);
	}
	.rdz-preset-chip:hover .rdz-preset-port {
		color: var(--text-primary);
	}

	.rdz-preset-chip.active {
		background: var(--bg-base);
		border-color: var(--accent-400);
	}
	.rdz-preset-chip.active .rdz-preset-port {
		color: var(--accent-400);
	}
	.rdz-preset-chip.active .rdz-preset-label {
		color: var(--accent-400);
	}

	.rdz-preset-chip:disabled {
		cursor: not-allowed;
	}

	.rdz-running-port {
		opacity: 0.65;
		pointer-events: none;
	}
	.rdz-running-port .rdz-preset-chip.active {
		pointer-events: none;
	}

	.rdz-running-hint {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		margin-top: 6px !important;
		opacity: 0.85;
	}
	.rdz-running-hint :global(svg) {
		font-size: 13px;
		color: var(--accent-400);
	}

	.rdz-warn-text {
		margin: 0;
		color: #ef9a9a;
		font-size: 11px;
	}

	.rdz-cgnat-note {
		display: flex;
		align-items: flex-start;
		gap: 6px;
		margin-top: var(--space-sm);
		padding: 0;
		color: var(--text-muted);
		font-size: 11px;
		line-height: 1.5;
	}

	.rdz-cgnat-note :global(svg) {
		font-size: 16px;
		color: var(--accent-400);
		flex-shrink: 0;
		margin-top: 1px;
	}

	.rdz-cgnat-content {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
		flex: 1;
	}

	.rdz-cgnat-note strong {
		color: var(--text-primary);
		font-weight: 600;
	}

	.rdz-cgnat-note em {
		color: var(--accent-400);
		font-style: normal;
		font-weight: 600;
	}

	.rdz-server-running {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.rdz-server-actions {
		display: flex;
		gap: var(--space-sm);
		flex-wrap: wrap;
		align-items: center;
	}

	.rdz-conn-card {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: var(--space-sm) var(--space-md);
		background: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		text-align: left;
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.rdz-conn-card:hover {
		border-color: var(--accent-400);
		background: var(--bg-elevated);
	}

	.rdz-conn-recommended {
		border-left: 2px solid var(--accent-400);
	}

	.rdz-conn-card small {
		font-size: 10px;
		color: var(--text-muted);
		line-height: 1.3;
	}

	.rdz-conn-line {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: 4px 8px;
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

	code {
		font-family: var(--font-mono, monospace);
		font-size: 12px;
		color: var(--text-accent);
		background: var(--bg-active);
		padding: 2px 6px;
		border-radius: var(--radius-sm);
	}

	.rdz-log {
		margin: 0;
		padding: var(--space-sm);
		background: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-sm);
		font-family: var(--font-mono, monospace);
		font-size: 11px;
		color: var(--text-secondary);
		max-height: 240px;
		overflow: auto;
		white-space: pre-wrap;
		word-break: break-all;
		user-select: text !important;
		-webkit-user-select: text !important;
		cursor: text !important;
	}

	.rdz-log::selection {
		background: var(--accent-400);
		color: var(--text-primary);
	}

	.rdz-log-details summary {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		cursor: pointer;
		font-size: 11px;
		color: var(--text-muted);
		padding: 4px 0;
		list-style: none;
	}

	.rdz-log-details summary::-webkit-details-marker {
		display: none;
	}

	.rdz-log-details summary::before {
		content: '▶';
		display: inline-block;
		font-size: 9px;
		transition: transform var(--transition-fast);
		color: var(--text-muted);
	}

	.rdz-log-details[open] summary::before {
		transform: rotate(90deg);
	}

	.rdz-log-details summary span {
		flex: 1;
	}

	.rdz-log-copy {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 3px 8px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		color: var(--text-secondary);
		font-size: 10px;
		font-weight: 600;
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.rdz-log-copy :global(svg) {
		font-size: 12px;
	}

	.rdz-log-copy:hover {
		border-color: var(--accent-400);
		color: var(--accent-400);
	}
</style>
