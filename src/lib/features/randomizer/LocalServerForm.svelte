<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import type { PythonStatus } from './types';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		portStr: string;
		password: string;
		port: number;
		portValid: boolean;
		python: PythonStatus | null;
		selectedSeed: string | null;
		starting: boolean;
		PRESET_PORTS: { p: number; label: string }[];
		onstart: () => void;
	};

	let {
		portStr = $bindable(),
		password = $bindable(),
		port,
		portValid,
		python,
		selectedSeed,
		starting,
		PRESET_PORTS,
		onstart
	}: Props = $props();
</script>

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
		onclick={onstart}
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

<style>
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

	.rdz-warn-text {
		margin: 0;
		color: #ef9a9a;
		font-size: 11px;
	}

	.rdz-muted {
		margin: 0;
		color: var(--text-muted);
		font-size: 12px;
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
</style>
