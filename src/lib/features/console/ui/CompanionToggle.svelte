<script lang="ts">
	import Icon from '@iconify/svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Mode = 'server' | 'client';

	type Props = {
		mode: Mode;
		onchange: (mode: Mode) => void;
		clientDisabled?: boolean;
	};
	let { mode, onchange, clientDisabled = false }: Props = $props();
</script>

<div class="zc-ct" role="tablist" aria-label={i18nState.locale && m.console_shell_mode()}>
	<button
		role="tab"
		aria-selected={mode === 'server'}
		class="zc-ct-btn"
		class:active={mode === 'server'}
		onclick={() => onchange('server')}
	>
		<Icon icon="mdi:server" />
		{i18nState.locale && m.console_companion_server()}
	</button>
	<button
		role="tab"
		aria-selected={mode === 'client'}
		class="zc-ct-btn"
		class:active={mode === 'client'}
		disabled={clientDisabled}
		onclick={() => onchange('client')}
		title={clientDisabled
			? i18nState.locale && m.console_companion_clientDisabled()
			: i18nState.locale && m.console_companion_switchClient()}
	>
		<Icon icon="mdi:account" />
		{i18nState.locale && m.console_companion_client()}
	</button>
</div>

<style>
	.zc-ct {
		display: inline-flex;
		gap: 4px;
	}

	.zc-ct-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 6px 14px;
		border: none;
		background: transparent;
		color: var(--text-muted);
		font-family: var(--font-body);
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		border-radius: var(--radius-sm);
		transition: color var(--transition-fast);
	}

	.zc-ct-btn :global(svg) {
		font-size: 15px;
	}

	.zc-ct-btn:hover:not(:disabled):not(.active) {
		color: var(--text-secondary);
	}

	.zc-ct-btn.active {
		color: var(--accent-400);
	}

	.zc-ct-btn:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}
</style>
