<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import Icon from '@iconify/svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';

	import LogFeed from '../ui/LogFeed.svelte';
	import CommandInput from '../ui/CommandInput.svelte';
	import HelpPalette from '../ui/HelpPalette.svelte';
	import ClientConnectForm from './ClientConnectForm.svelte';
	import { ClientSession, type ConnectParams } from './client-session.svelte';

	const session = new ClientSession();
	let helpOpen = $state(false);

	let unlistenBridgeStarted: UnlistenFn | null = null;
	let unlistenBridgeLog: UnlistenFn | null = null;
	let unlistenBridgeExited: UnlistenFn | null = null;

	/** Drop AP's ANSI color escapes so they don't render as garbage in the feed. */
	function stripAnsi(s: string): string {
		// eslint-disable-next-line no-control-regex
		return s.replace(/\x1b\[[0-9;]*[A-Za-z]/g, '');
	}

	onMount(async () => {
		session.log.appendSystem(
			'Zephyr Client console. Fill in the host/slot form to connect.',
			'system'
		);

		unlistenBridgeStarted = await listen<{ pid: number; patch: string }>(
			'randomizer://bridge-started',
			(e) => {
				session.log.append({
					level: 'system',
					source: 'BRIDGE',
					text: `Emulator bridge started (pid ${e.payload.pid}) for ${e.payload.patch}`,
					origin: 'system'
				});
			}
		);

		unlistenBridgeLog = await listen<{ stream: 'stdout' | 'stderr'; text: string }>(
			'randomizer://bridge-log',
			(e) => {
				const text = stripAnsi(e.payload.text);
				if (!text.trim()) return;
				const isErr = e.payload.stream === 'stderr';
				session.log.append({
					level: isErr ? 'error' : 'info',
					source: 'BRIDGE',
					text,
					origin: isErr ? 'bridge-stderr' : 'bridge-stdout'
				});
			}
		);

		unlistenBridgeExited = await listen<{ code: number | null; patch: string }>(
			'randomizer://bridge-exited',
			(e) => {
				session.log.append({
					level: e.payload.code === 0 ? 'system' : 'warn',
					source: 'BRIDGE',
					text: `Emulator bridge for ${e.payload.patch} exited (code ${e.payload.code ?? '?'}).`,
					origin: 'system'
				});
			}
		);
	});

	onDestroy(() => {
		session.dispose();
		unlistenBridgeStarted?.();
		unlistenBridgeLog?.();
		unlistenBridgeExited?.();
	});

	function handleConnect(params: ConnectParams) {
		void session.connect(params);
	}
</script>

{#if session.status === 'disconnected' || session.status === 'error'}
	<div class="zc-cv-wrap">
		<ClientConnectForm
			onsubmit={handleConnect}
			loading={false}
			error={session.status === 'error' ? session.statusMessage : ''}
		/>
	</div>
{:else}
	<section class="zc-client-view">
		<aside class="zc-slot">
			<header>
				<Icon icon="mdi:account-box" />
				<span>Slot info</span>
			</header>
			<div class="zc-slot-row">
				<span class="zc-slot-label">Seed</span>
				<code>{session.seedName || '—'}</code>
			</div>
			<div class="zc-slot-row">
				<span class="zc-slot-label">Slot</span>
				<code>{session.mySlot ?? '—'}</code>
			</div>
			<div class="zc-slot-row">
				<span class="zc-slot-label">Team</span>
				<code>{session.myTeam ?? '—'}</code>
			</div>
			<div class="zc-slot-row">
				<span class="zc-slot-label">Hint pts</span>
				<code>{session.hintPoints}</code>
			</div>
			<div class="zc-slot-row">
				<span class="zc-slot-label">Items</span>
				<code>{session.receivedItems.length}</code>
			</div>

			<div class="zc-slot-divider"></div>

			<header>
				<Icon icon="mdi:account-group" />
				<span>Players</span>
				<small>{session.players.length}</small>
			</header>
			{#if session.players.length === 0}
				<p class="zc-slot-empty">no roster yet</p>
			{:else}
				<ul>
					{#each session.players as p (p.slot)}
						<li class:me={p.slot === session.mySlot}>
							<span class="zc-slot-dot"></span>
							<span class="zc-slot-name">{p.alias || p.name}</span>
							<small>#{p.slot}</small>
						</li>
					{/each}
				</ul>
			{/if}

			<footer>
				<div class="zc-conn" class:on={session.status === 'connected'}>
					<span class="zc-conn-dot"></span>
					{session.status}
				</div>
				<button class="zc-disc" onclick={() => session.disconnect()}>
					<Icon icon="mdi:logout" />
					Disconnect
				</button>
			</footer>
		</aside>

		<div class="zc-main">
			<LogFeed store={session.log} />
			<CommandInput
				registry={session.registry}
				history={session.history}
				prefix="!"
				onsubmit={(line) => session.submit(line)}
				onhelp={() => (helpOpen = true)}
			/>
		</div>
	</section>

	<HelpPalette
		bind:open={helpOpen}
		registry={session.registry}
		prefix="!"
		onclose={() => (helpOpen = false)}
		ontry={(line) => void session.submit(line)}
	/>
{/if}

<style>
	.zc-cv-wrap {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-height: 0;
	}

	.zc-client-view {
		display: flex;
		flex: 1;
		min-height: 0;
		overflow: hidden;
	}

	.zc-slot {
		flex: 0 0 280px;
		display: flex;
		flex-direction: column;
		background: var(--bg-surface);
		border-right: 1px solid var(--border-subtle);
		min-height: 0;
		overflow-y: auto;
	}

	.zc-slot header {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 12px 14px 8px;
		color: var(--text-muted);
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.1em;
		font-weight: 700;
	}
	.zc-slot header :global(svg) {
		font-size: 14px;
		color: var(--accent-400);
	}
	.zc-slot header small {
		margin-left: auto;
		font-size: 11px;
		color: var(--text-secondary);
	}

	.zc-slot-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 14px;
		font-size: 12px;
	}
	.zc-slot-label {
		width: 70px;
		color: var(--text-muted);
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
	}
	.zc-slot-row code {
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		color: var(--text-primary);
		font-size: 12px;
	}

	.zc-slot-divider {
		height: 1px;
		background: var(--border-subtle);
		margin: 10px 12px;
	}

	.zc-slot ul {
		list-style: none;
		margin: 0;
		padding: 4px 8px;
	}

	.zc-slot li {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 8px;
		border-radius: var(--radius-sm);
		color: var(--text-secondary);
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		font-size: 12px;
	}
	.zc-slot li.me {
		background: var(--bg-active);
		color: var(--accent-400);
	}

	.zc-slot-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--accent-400);
		opacity: 0.55;
	}
	.zc-slot li.me .zc-slot-dot {
		opacity: 1;
		box-shadow: 0 0 6px var(--accent-400);
	}

	.zc-slot-name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.zc-slot li small {
		font-size: 10px;
		color: var(--text-muted);
	}

	.zc-slot-empty {
		margin: 0 14px 6px;
		color: var(--text-muted);
		font-style: italic;
		font-size: 12px;
	}

	.zc-slot footer {
		margin-top: auto;
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 12px;
		border-top: 1px solid var(--border-subtle);
		flex-shrink: 0;
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
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--text-muted);
	}
	.zc-conn.on {
		color: var(--success, var(--accent-400));
	}
	.zc-conn.on .zc-conn-dot {
		background: var(--success, var(--accent-400));
		box-shadow: 0 0 8px var(--success, var(--accent-400));
	}

	.zc-disc {
		margin-left: auto;
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 4px 10px;
		border: 1px solid var(--border-default);
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--text-muted);
		font-size: 11px;
		cursor: pointer;
	}
	.zc-disc:hover {
		border-color: var(--error);
		color: var(--error);
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
