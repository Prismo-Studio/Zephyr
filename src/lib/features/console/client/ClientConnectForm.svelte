<script lang="ts">
	import Icon from '@iconify/svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import type { ConnectParams } from './client-session.svelte';

	type Props = {
		/** Called when the user submits the form. */
		onsubmit: (params: ConnectParams) => void;
		initialHost?: string;
		initialSlot?: string;
		loading?: boolean;
		error?: string;
	};
	let { onsubmit, initialHost = '127.0.0.1:38281', initialSlot = '', loading = false, error = '' }: Props = $props();

	const STORAGE_KEY = 'zephyr-console-last-connect';

	let host = $state(initialHost);
	let slot = $state(initialSlot);
	let password = $state('');
	let useTracker = $state(true);
	let game = $state('');

	// Prefill from localStorage if available.
	try {
		const raw = typeof localStorage !== 'undefined' ? localStorage.getItem(STORAGE_KEY) : null;
		if (raw) {
			const p = JSON.parse(raw);
			if (typeof p.host === 'string') host = p.host;
			if (typeof p.slot === 'string') slot = p.slot;
			if (typeof p.useTracker === 'boolean') useTracker = p.useTracker;
			if (typeof p.game === 'string') game = p.game;
		}
	} catch {
		// ignore
	}

	function submit() {
		const h = host.trim();
		const s = slot.trim();
		if (!h || !s) return;
		try {
			localStorage.setItem(STORAGE_KEY, JSON.stringify({ host: h, slot: s, useTracker, game: game.trim() }));
		} catch {
			// ignore
		}
		onsubmit({
			host: h,
			slot: s,
			game: game.trim(),
			password: password.trim(),
			useTracker
		});
	}
</script>

<div class="zc-cf">
	<div class="zc-cf-card">
		<header>
			<Icon icon="mdi:connection" />
			<h2>{i18nState.locale && m.console_client_connectTitle()}</h2>
		</header>

		<p class="zc-cf-desc">{i18nState.locale && m.console_client_connectDesc()}</p>

		<div class="zc-cf-row">
			<label>
				<span>{i18nState.locale && m.console_client_host()}</span>
				<Input bind:value={host} placeholder="127.0.0.1:38281" />
			</label>
			<label>
				<span>{i18nState.locale && m.console_client_slotName()}</span>
				<Input bind:value={slot} placeholder="Player1" />
			</label>
		</div>

		<div class="zc-cf-row">
			<label>
				<span>
					{i18nState.locale && m.console_client_password()}
					<small>({i18nState.locale && m.console_client_optional()})</small>
				</span>
				<Input bind:value={password} placeholder="" />
			</label>
			<label class:disabled={useTracker}>
				<span>
					{i18nState.locale && m.console_client_game()}
					<small>
						({useTracker
							? i18nState.locale && m.console_client_gameIgnoredTracker()
							: i18nState.locale && m.console_client_gameRequired()})
					</small>
				</span>
				<Input bind:value={game} placeholder="A Link to the Past" />
			</label>
		</div>

		<label class="zc-cf-toggle">
			<input type="checkbox" bind:checked={useTracker} />
			<div>
				<strong>{i18nState.locale && m.console_client_trackerMode()}</strong>
				<small>{i18nState.locale && m.console_client_trackerModeDesc()}</small>
			</div>
		</label>

		{#if error}
			<div class="zc-cf-error">
				<Icon icon="mdi:alert-circle" />
				{error}
			</div>
		{/if}

		<div class="zc-cf-actions">
			<Button
				variant="primary"
				onclick={submit}
				disabled={loading || !host.trim() || !slot.trim()}
				{loading}
			>
				{#snippet icon()}<Icon icon="mdi:login" />{/snippet}
				{loading
					? i18nState.locale && m.console_client_connecting()
					: i18nState.locale && m.console_client_connect()}
			</Button>
		</div>
	</div>
</div>

<style>
	.zc-cf {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 32px;
		background: var(--bg-base);
	}

	.zc-cf-card {
		width: min(560px, 100%);
		background: var(--bg-surface);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
		padding: 28px 32px;
		display: flex;
		flex-direction: column;
		gap: 16px;
		box-shadow: var(--shadow-lg);
	}

	.zc-cf-card header {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.zc-cf-card header :global(svg) {
		font-size: 22px;
		color: var(--accent-400);
	}

	.zc-cf-card h2 {
		margin: 0;
		font-size: 18px;
		color: var(--text-primary);
	}

	.zc-cf-desc {
		margin: 0;
		font-size: 12px;
		color: var(--text-muted);
		line-height: 1.6;
	}

	.zc-cf-desc code {
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		background: var(--bg-active);
		padding: 1px 6px;
		border-radius: var(--radius-sm);
		font-size: 11px;
	}
	.zc-cf-desc kbd {
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: 3px;
		padding: 1px 5px;
		font-size: 10px;
		font-weight: 600;
	}

	.zc-cf-row {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
	}

	.zc-cf-row label {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.zc-cf-row label span {
		font-size: 11px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--text-muted);
	}
	.zc-cf-row label span small {
		font-size: 10px;
		font-weight: 400;
		text-transform: none;
		letter-spacing: 0;
		color: var(--text-muted);
		opacity: 0.7;
	}
	.zc-cf-row label.disabled {
		opacity: 0.5;
		pointer-events: none;
	}

	.zc-cf-toggle {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		padding: 10px 12px;
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		background: var(--bg-base);
		cursor: pointer;
	}

	.zc-cf-toggle input {
		margin-top: 4px;
		accent-color: var(--accent-400);
	}

	.zc-cf-toggle strong {
		display: block;
		font-size: 13px;
		color: var(--text-primary);
		margin-bottom: 3px;
	}

	.zc-cf-toggle small {
		font-size: 11px;
		color: var(--text-muted);
		line-height: 1.5;
	}

	.zc-cf-toggle code {
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		background: var(--bg-active);
		padding: 0 4px;
		border-radius: 3px;
	}

	.zc-cf-error {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		border-radius: var(--radius-md);
		background: color-mix(in srgb, var(--error) 12%, transparent);
		color: var(--error);
		border: 1px solid color-mix(in srgb, var(--error) 35%, transparent);
		font-size: 12px;
	}

	.zc-cf-error :global(svg) {
		font-size: 16px;
	}

	.zc-cf-actions {
		display: flex;
		justify-content: flex-end;
	}
</style>
