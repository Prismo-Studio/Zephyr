<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount, onDestroy } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	import LogFeed from '../ui/LogFeed.svelte';
	import CommandInput from '../ui/CommandInput.svelte';
	import HelpPalette from '../ui/HelpPalette.svelte';
	import { ServerSession } from './server-session.svelte';

	const session = new ServerSession();
	let inputRef: CommandInput | undefined = $state();
	let helpOpen = $state(false);

	onMount(() => {
		void session.start();
	});

	onDestroy(() => {
		session.dispose();
	});

	function handleTryTemplate(line: string) {
		// Simulate typing it so the ghost hint fires.
		// CommandInput has no public setter. We cheat via a CustomEvent bridge
		// in the future; for now, directly submit once.
		void session.submit(line);
	}
</script>

<section class="zc-server-view">
	<div class="zc-players">
		<header>
			<Icon icon="mdi:account-group" />
			<span>{i18nState.locale && m.console_server_activeSources()}</span>
			<small>{session.log.distinctSources.length}</small>
		</header>

		{#if session.log.distinctSources.length === 0}
			<p class="zc-players-empty">{i18nState.locale && m.console_client_noRoster()}</p>
		{:else}
			<ul>
				{#each session.log.distinctSources as source (source)}
					<li>
						<button
							class="zc-source-btn"
							class:active={session.log.sourceFilter === source}
							onclick={() =>
								(session.log.sourceFilter = session.log.sourceFilter === source ? null : source)}
						>
							<span class="zc-source-dot"></span>
							<span class="zc-source-name">{source}</span>
						</button>
					</li>
				{/each}
			</ul>
		{/if}

		<footer>
			<div class="zc-conn" class:on={session.connected}>
				<span class="zc-conn-dot"></span>
				{session.connected
					? i18nState.locale && m.console_server_live()
					: i18nState.locale && m.console_server_idle()}
			</div>
			<button class="zc-clear-btn" onclick={() => session.log.clear()}>
				<Icon icon="mdi:broom" />
				{i18nState.locale && m.console_server_clear()}
			</button>
		</footer>
	</div>

	<div class="zc-main">
		<LogFeed store={session.log} />
		<CommandInput
			bind:this={inputRef}
			registry={session.registry}
			history={session.history}
			prefix="/"
			onsubmit={(line) => session.submit(line)}
			onhelp={() => (helpOpen = true)}
		/>
	</div>
</section>

<HelpPalette
	bind:open={helpOpen}
	registry={session.registry}
	prefix="/"
	onclose={() => (helpOpen = false)}
	ontry={handleTryTemplate}
/>

<style>
	.zc-server-view {
		display: flex;
		flex: 1;
		min-height: 0;
		overflow: hidden;
	}

	.zc-players {
		flex: 0 0 260px;
		display: flex;
		flex-direction: column;
		background: var(--bg-surface);
		border-right: 1px solid var(--border-subtle);
		min-height: 0;
	}

	.zc-players header {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 12px 14px 8px;
		color: var(--text-muted);
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.1em;
		font-weight: 700;
		border-bottom: 1px solid var(--border-subtle);
	}
	.zc-players header :global(svg) {
		font-size: 14px;
		color: var(--accent-400);
	}
	.zc-players header small {
		margin-left: auto;
		font-size: 11px;
		color: var(--text-secondary);
	}

	.zc-players ul {
		flex: 1;
		overflow-y: auto;
		list-style: none;
		padding: 6px 8px;
		margin: 0;
	}

	.zc-players-empty {
		padding: 24px 14px;
		margin: 0;
		font-size: 12px;
		color: var(--text-muted);
		font-style: italic;
	}

	.zc-source-btn {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 6px 10px;
		border: none;
		background: transparent;
		color: var(--text-secondary);
		border-radius: var(--radius-sm);
		cursor: pointer;
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		font-size: 12px;
	}
	.zc-source-btn:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}
	.zc-source-btn.active {
		background: var(--bg-active);
		color: var(--accent-400);
	}

	.zc-source-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--text-muted);
	}
	.zc-source-btn.active .zc-source-dot {
		background: var(--accent-400);
	}

	.zc-players footer {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 12px;
		border-top: 1px solid var(--border-subtle);
	}

	.zc-conn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		font-weight: 700;
		color: var(--text-muted);
	}
	.zc-conn-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--text-muted);
	}
	.zc-conn.on {
		color: var(--success, var(--accent-400));
	}
	.zc-conn.on .zc-conn-dot {
		background: var(--success, var(--accent-400));
	}

	.zc-clear-btn {
		margin-left: auto;
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 4px 8px;
		border: 1px solid var(--border-default);
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--text-muted);
		font-size: 11px;
		cursor: pointer;
	}
	.zc-clear-btn:hover {
		border-color: var(--accent-400);
		color: var(--accent-400);
	}

	.zc-main {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-width: 0;
		min-height: 0;
		position: relative;
	}
</style>
