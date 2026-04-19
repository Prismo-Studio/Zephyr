import { invoke } from '$lib/invoke';
import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

import { LogStore } from '../core/log-store.svelte';
import { parseLine } from '../core/command-parser';
import { CommandRegistry } from '../core/command-registry';
import { loadCommandHistory, pushCommandHistory } from '../core/history';
import { classifyLine, SERVER_LOG_EVENT, type ServerLogEvent } from '../core/protocol';
import { registerServerCommands } from './server-commands';
import { m } from '$lib/paraglide/messages';

/**
 * Owns the log store, the command registry, and the Tauri event subscription
 * for one Server view. Created by `ServerView.svelte` on mount, disposed on
 * destroy. Not a singleton — the companion window creates its own instance.
 */
export class ServerSession {
	log = new LogStore();
	registry = new CommandRegistry();
	/** Command history, most-recent first. Capped at 100, persisted globally. */
	history: string[] = $state([]);
	/** Whether the Rust runtime reports a live MultiServer right now. */
	connected = $state(false);

	private unlisten: UnlistenFn | null = null;
	private historyKey = 'zephyr-console-history-server';

	constructor() {
		registerServerCommands(this);
		this.loadHistory();
	}

	async start() {
		// 1. Backfill from the recent-log ring.
		try {
			const snapshot = await tauriInvoke<string[]>('console_server_recent_log');
			for (const raw of snapshot) {
				const stream = raw.startsWith('[err]') ? 'stderr' : 'stdout';
				const { level, source, text } = classifyLine(raw, stream);
				this.log.append({
					level,
					source,
					text,
					origin: stream === 'stderr' ? 'server-stderr' : 'server-stdout'
				});
			}
			this.connected = snapshot.length > 0;
		} catch {
			// no server running yet — fine, we just keep an empty feed
		}

		// 2. Subscribe to live events.
		this.unlisten = await listen<ServerLogEvent>(SERVER_LOG_EVENT, (event) => {
			const { stream, line, ts_ms } = event.payload;
			const { level, source, text } = classifyLine(line, stream);
			this.log.append({
				level,
				source,
				text,
				origin: stream === 'stderr' ? 'server-stderr' : 'server-stdout',
				ts: ts_ms
			});
			this.connected = true;
		});

		this.log.appendSystem(m.console_msg_ready(), 'system');
	}

	dispose() {
		this.unlisten?.();
		this.unlisten = null;
	}

	/** Execute what the user just typed. Dispatches via registry if recognized,
	 *  otherwise writes the raw line to MultiServer stdin (AP parses everything
	 *  that starts with `/` server-side — but we already handled that above). */
	async submit(raw: string) {
		const line = raw.trim();
		if (!line) return;

		this.pushHistory(line);

		const parsed = parseLine(line);

		if (parsed.kind === 'chat') {
			// Echo locally; on the Server console there's no "chat send" — it
			// gets dropped. We echo so the user knows we saw them type it.
			this.log.append({
				level: 'chat',
				source: 'YOU',
				text: parsed.text,
				origin: 'echo'
			});
			this.log.appendSystem(m.console_msg_noServerChat(), 'warn');
			return;
		}

		if (parsed.kind !== 'command') return;

		if (parsed.prefix !== '/') {
			this.log.appendSystem(m.console_msg_clientCmd(), 'warn');
			return;
		}

		// Echo what we're sending
		this.log.append({
			level: 'admin',
			source: 'YOU',
			text: line,
			origin: 'echo'
		});

		const def = this.registry.lookup('/', parsed.name);

		if (def?.status === 'coming-soon') {
			this.log.appendSystem(m.console_msg_comingSoon({ name: parsed.name }), 'warn');
			return;
		}

		if (def?.run) {
			try {
				await def.run(parsed.args, line);
			} catch (err: unknown) {
				this.log.appendSystem(
					m.console_msg_cmdFailed({ error: String((err as Error).message ?? err) }),
					'error'
				);
			}
			return;
		}

		// Unknown or no handler → forward raw line to MultiServer stdin and
		// let its own parser deal with it.
		await this.sendStdin(line);
	}

	/** Write a line to MultiServer's stdin. Error surfaced as a system line. */
	async sendStdin(line: string) {
		try {
			await invoke('console_server_send_stdin', { line });
		} catch (err: unknown) {
			this.log.appendSystem(
				m.console_msg_sendFailed({ error: String((err as Error).message ?? err) }),
				'error'
			);
		}
	}

	// ── Command history persistence ────────────────────────────────────
	private loadHistory() {
		this.history = loadCommandHistory(this.historyKey);
	}

	private pushHistory(line: string) {
		this.history = pushCommandHistory(this.historyKey, this.history, line);
	}
}
