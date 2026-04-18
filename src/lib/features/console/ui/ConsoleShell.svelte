<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import Icon from '@iconify/svelte';

	import CompanionToggle from './CompanionToggle.svelte';
	import ServerView from '../server/ServerView.svelte';
	import ClientView from '../client/ClientView.svelte';

	type Mode = 'server' | 'client';
	const STORAGE_KEY = 'zephyr-console-mode';

	let mode: Mode = $state('server');
	const isStandalone =
		typeof window !== 'undefined' &&
		new URLSearchParams(window.location.search).get('standalone') === '1';

	onMount(() => {
		const saved = localStorage.getItem(STORAGE_KEY);
		if (saved === 'server' || saved === 'client') mode = saved;
		window.addEventListener('keydown', onKey);
	});

	onDestroy(() => {
		window.removeEventListener('keydown', onKey);
	});

	function switchMode(next: Mode) {
		mode = next;
		try {
			localStorage.setItem(STORAGE_KEY, next);
		} catch {
			// ignore
		}
	}

	function onKey(e: KeyboardEvent) {
		if (!e.ctrlKey || !e.shiftKey) return;
		const k = e.key.toLowerCase();
		if (k === 's') {
			e.preventDefault();
			switchMode('server');
		} else if (k === 'c') {
			e.preventDefault();
			switchMode('client');
		} else if (k === 't') {
			e.preventDefault();
			switchMode(mode === 'server' ? 'client' : 'server');
		}
	}
</script>

<div class="zc-shell" class:standalone={isStandalone}>
	<header class="zc-header">
		{#if !isStandalone}
			<a class="zc-back" href="/randomizer" aria-label="Back">
				<Icon icon="mdi:arrow-left" />
			</a>
		{/if}

		<div class="zc-brand">
			<span class="zc-brand-dot"></span>
			<strong>Zephyr</strong>
			<span class="zc-brand-sep">⟩</span>
			<span class="zc-brand-title">Console</span>
		</div>

		<div class="zc-header-center">
			<CompanionToggle {mode} onchange={switchMode} />
		</div>

		<div class="zc-header-right">
			<kbd>Ctrl</kbd><kbd>Shift</kbd><kbd>T</kbd>
			<span class="zc-right-label">swap</span>
		</div>
	</header>

	<!-- Keep both views mounted at all times so switching tabs doesn't unmount
	     the inactive session — which would fire onDestroy → dispose() and
	     drop its live WebSocket. We just hide the background view with CSS;
	     its log keeps accumulating in the background and bridge-log events
	     stay subscribed. -->
	<main class="zc-body">
		<div class="zc-pane" class:zc-hidden={mode !== 'server'}>
			<ServerView />
		</div>
		<div class="zc-pane" class:zc-hidden={mode !== 'client'}>
			<ClientView />
		</div>
	</main>

	<footer class="zc-footer">
		<span><kbd>Ctrl</kbd>+<kbd>/</kbd> help</span>
		<span><kbd>↑</kbd>/<kbd>↓</kbd> history</span>
		<span><kbd>Ctrl+Shift+S</kbd>/<kbd>C</kbd> mode</span>
		<span class="zc-footer-spacer"></span>
		<span>mode · <strong>{mode}</strong></span>
		{#if isStandalone}
			<span class="zc-footer-pill">standalone</span>
		{/if}
	</footer>
</div>

<style>
	.zc-shell {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		height: 100%;
		background: var(--bg-base);
		color: var(--text-primary);
		overflow: hidden;
	}

	.zc-shell.standalone {
		/* a touch tighter header in standalone mode since the OS already draws a title bar */
		/* reserved for future tweaks */
	}

	.zc-header {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 8px 14px;
		background: var(--bg-surface);
		border-bottom: 1px solid var(--border-subtle);
		flex-shrink: 0;
	}

	.zc-back {
		width: 30px;
		height: 30px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		color: var(--text-secondary);
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 18px;
		text-decoration: none;
	}
	.zc-back:hover {
		border-color: var(--accent-400);
		color: var(--accent-400);
	}

	.zc-brand {
		display: flex;
		align-items: center;
		gap: 8px;
		font-family: var(--font-display, var(--font-body));
		font-size: 14px;
	}

	.zc-brand-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--accent-400);
		box-shadow: 0 0 6px var(--accent-400);
	}

	.zc-brand strong {
		color: var(--text-primary);
		font-weight: 700;
	}

	.zc-brand-sep {
		color: var(--text-muted);
	}

	.zc-brand-title {
		color: var(--text-secondary);
	}

	.zc-header-center {
		flex: 1;
		display: flex;
		justify-content: center;
	}

	.zc-header-right {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.zc-right-label {
		font-size: 10px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.08em;
		margin-left: 4px;
	}

	.zc-body {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-height: 0;
		overflow: hidden;
	}

	.zc-pane {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-height: 0;
		overflow: hidden;
	}

	.zc-hidden {
		display: none;
	}

	.zc-footer {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 4px 14px;
		border-top: 1px solid var(--border-subtle);
		background: var(--bg-surface);
		font-size: 11px;
		color: var(--text-muted);
		flex-shrink: 0;
	}

	.zc-footer-spacer {
		flex: 1;
	}

	.zc-footer strong {
		color: var(--accent-400);
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.08em;
	}

	.zc-footer-pill {
		background: color-mix(in srgb, var(--accent-400) 12%, transparent);
		color: var(--accent-400);
		padding: 1px 8px;
		border-radius: var(--radius-full);
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.08em;
	}

	kbd {
		display: inline-flex;
		align-items: center;
		padding: 1px 6px;
		border: 1px solid var(--border-default);
		border-radius: 3px;
		background: var(--bg-elevated);
		color: var(--text-secondary);
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		font-size: 10px;
		font-weight: 600;
	}
</style>
