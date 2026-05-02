<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Dropdown from '$lib/components/ui/Dropdown.svelte';
	import * as api from './api';
	import type { ServerStatus } from './types';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type ConnTarget = 'local' | 'lan' | 'public';

	type Props = {
		server: ServerStatus;
		connTarget: ConnTarget;
		portStr: string;
		copiedKey: string | null;
		PRESET_PORTS: { p: number; label: string }[];
		oncopy: (text: string, key: string) => void;
		onstop: () => void;
	};

	let {
		server,
		connTarget = $bindable(),
		portStr,
		copiedKey,
		PRESET_PORTS,
		oncopy,
		onstop
	}: Props = $props();

	let selectedIp = $derived(
		connTarget === 'local'
			? '127.0.0.1'
			: connTarget === 'lan'
				? (server.local_ip ?? '127.0.0.1')
				: (server.public_ip ?? '127.0.0.1')
	);
	let selectedAddr = $derived(`${selectedIp}:${server.port}`);
	let selectedIcon = $derived(
		connTarget === 'local' ? 'mdi:laptop' : connTarget === 'lan' ? 'mdi:lan' : 'mdi:earth'
	);
	let selectedLabel = $derived(
		connTarget === 'local'
			? (i18nState.locale && m.randomizer_thisMachine()) || 'Cette machine'
			: connTarget === 'lan'
				? (i18nState.locale && m.randomizer_lanTailscale()) || 'LAN / Tailscale'
				: (i18nState.locale && m.randomizer_publicInternet()) || 'Internet public'
	);
	let selectedDesc = $derived(
		connTarget === 'local'
			? (i18nState.locale && m.randomizer_thisMachineDesc()) || ''
			: connTarget === 'lan'
				? (i18nState.locale && m.randomizer_lanDesc()) || ''
				: (i18nState.locale && m.randomizer_publicDesc({ port: (server.port ?? 0).toString() })) ||
					''
	);

	let connOptions = $derived([
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
	]);
</script>

<div class="rdz-server-running">
	<div class="rdz-running-line">
		<span class="rdz-live-dot"></span>
		<span>
			{i18nState.locale && m.randomizer_runningOnPort({ port: (server.port ?? 0).toString() })}
		</span>
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
			onclick={() => oncopy(selectedAddr, 'addr')}
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
		<Button variant="danger" onclick={onstop}>
			{#snippet icon()}<Icon icon="mdi:stop" />{/snippet}
			{i18nState.locale && m.randomizer_stopServer()}
		</Button>
	</div>
</div>

<style>
	.rdz-server-running {
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

	.rdz-muted {
		margin: 0;
		color: var(--text-muted);
		font-size: 12px;
	}
</style>
