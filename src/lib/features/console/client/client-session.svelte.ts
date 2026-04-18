import { LogStore } from '../core/log-store.svelte';
import { CommandRegistry } from '../core/command-registry';
import { parseLine } from '../core/command-parser';
import {
	PROTOCOL_VERSION,
	TRACKER_ITEMS_HANDLING,
	flattenPrintJSON,
	generateUuid,
	parseFrame,
	serializeFrame,
	type ClientPacket,
	type ConnectPacket,
	type ConnectedPacket,
	type PrintJSONPacket,
	type RoomInfoPacket,
	type RoomUpdatePacket,
	type SayPacket,
	type ServerPacket
} from '../core/ap-protocol';
import { registerClientCommands } from './client-commands';

export type ClientStatus =
	| 'disconnected'
	| 'connecting'
	| 'authenticating'
	| 'connected'
	| 'error';

export type ConnectParams = {
	host: string; // "127.0.0.1:38281"
	slot: string; // "Player1"
	game: string; // "" for Tracker, or the real game name
	password: string;
	useTracker: boolean;
};

export type PlayerEntry = { team: number; slot: number; alias: string; name: string };

/** One ClientSession owns a single WebSocket connection to an AP MultiServer,
 *  a command registry, a log store, and a reactive slice of the room state. */
export class ClientSession {
	log = new LogStore();
	registry = new CommandRegistry();
	history: string[] = $state([]);

	status: ClientStatus = $state('disconnected');
	statusMessage: string = $state('');

	/** Slot id once connected. */
	mySlot: number | null = $state(null);
	/** Team id once connected. */
	myTeam: number | null = $state(null);
	/** Hint budget (points) reported by the server. */
	hintPoints: number = $state(0);
	/** Connected players roster. */
	players: PlayerEntry[] = $state([]);
	/** Most recently received items (cap 100). */
	receivedItems: { item: number; location: number; player: number; flags: number }[] = $state([]);

	/** Seed name from RoomInfo. */
	seedName = $state('');

	private ws: WebSocket | null = null;
	private params: ConnectParams | null = null;
	private reconnectAttempts = 0;
	private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
	private historyKey = 'zephyr-console-history-client';

	constructor() {
		registerClientCommands(this);
		this.loadHistory();
	}

	async connect(params: ConnectParams) {
		this.params = params;
		await this.openSocket();
	}

	private async openSocket() {
		if (!this.params) return;
		this.cancelReconnect();

		const { host } = this.params;
		const url = `ws://${host}/`;

		this.status = 'connecting';
		this.statusMessage = `Opening ${url}…`;
		this.log.appendSystem(`connecting to ${url}`, 'system');

		try {
			this.ws = new WebSocket(url);
		} catch (err: unknown) {
			this.status = 'error';
			this.statusMessage = `failed to open: ${(err as Error).message ?? err}`;
			this.log.appendSystem(this.statusMessage, 'error');
			return;
		}

		this.ws.addEventListener('open', () => {
			this.reconnectAttempts = 0;
			this.log.appendSystem('socket open, awaiting RoomInfo…', 'system');
		});

		this.ws.addEventListener('message', (evt) => {
			const text = typeof evt.data === 'string' ? evt.data : '';
			const packets = parseFrame(text);
			for (const p of packets) this.handlePacket(p);
		});

		this.ws.addEventListener('error', () => {
			this.log.appendSystem('socket error', 'error');
		});

		this.ws.addEventListener('close', (evt) => {
			this.log.appendSystem(
				`socket closed (${evt.code}${evt.reason ? ` ${evt.reason}` : ''})`,
				'system'
			);
			const wasConnected = this.status === 'connected';
			this.status = 'disconnected';
			this.statusMessage = evt.reason || 'disconnected';
			this.ws = null;

			if (wasConnected && this.params) {
				// Opportunistic reconnect with backoff, capped at 10 tries.
				if (this.reconnectAttempts < 10) {
					const delay = Math.min(10_000, 500 * Math.pow(2, this.reconnectAttempts));
					this.reconnectAttempts += 1;
					this.log.appendSystem(`reconnecting in ${Math.round(delay / 100) / 10}s…`, 'warn');
					this.reconnectTimer = setTimeout(() => void this.openSocket(), delay);
				}
			}
		});
	}

	disconnect() {
		this.cancelReconnect();
		this.params = null;
		if (this.ws) {
			try {
				this.ws.close(1000, 'user disconnect');
			} catch {
				// ignore
			}
			this.ws = null;
		}
		this.status = 'disconnected';
		this.statusMessage = '';
		this.log.appendSystem('disconnected', 'system');
	}

	dispose() {
		this.disconnect();
	}

	private cancelReconnect() {
		if (this.reconnectTimer) {
			clearTimeout(this.reconnectTimer);
			this.reconnectTimer = null;
		}
	}

	private send(packets: ClientPacket[]) {
		if (!this.ws || this.ws.readyState !== WebSocket.OPEN) return;
		try {
			this.ws.send(serializeFrame(packets));
		} catch (err: unknown) {
			this.log.appendSystem(`send failed: ${(err as Error).message ?? err}`, 'error');
		}
	}

	// ── Packet dispatch ─────────────────────────────────────────────────
	private handlePacket(p: ServerPacket) {
		switch (p.cmd) {
			case 'RoomInfo':
				this.onRoomInfo(p as RoomInfoPacket);
				return;
			case 'Connected':
				this.onConnected(p as ConnectedPacket);
				return;
			case 'ConnectionRefused': {
				const errors = (p as { errors?: string[] }).errors ?? ['unknown'];
				this.status = 'error';
				this.statusMessage = `refused: ${errors.join(', ')}`;
				this.log.appendSystem(this.statusMessage, 'error');
				try {
					this.ws?.close(1000);
				} catch {
					// ignore
				}
				return;
			}
			case 'PrintJSON':
				this.onPrintJSON(p as PrintJSONPacket);
				return;
			case 'RoomUpdate':
				this.onRoomUpdate(p as RoomUpdatePacket);
				return;
			case 'ReceivedItems': {
				const items =
					(p as { items?: { item: number; location: number; player: number; flags: number }[] })
						.items ?? [];
				this.receivedItems = [...this.receivedItems, ...items].slice(-100);
				for (const it of items) {
					const fromName = this.nameOfSlot(it.player) ?? `Slot#${it.player}`;
					this.log.append({
						level: 'item',
						source: 'ITEM',
						origin: 'client',
						text: `received Item#${it.item} from ${fromName}`
					});
				}
				return;
			}
			case 'InvalidPacket':
				this.log.appendSystem(
					`server rejected packet: ${(p as { text?: string }).text ?? ''}`,
					'error'
				);
				return;
			case 'Bounced':
				// Not surfaced in v1; Phase 5 will use this for Zephyr sidecar events.
				return;
			default:
				// Unknown cmd — dump to system so we can debug.
				this.log.appendSystem(`unknown packet: ${p.cmd}`, 'system');
				return;
		}
	}

	private onRoomInfo(p: RoomInfoPacket) {
		this.seedName = p.seed_name ?? '';
		this.status = 'authenticating';
		this.statusMessage = `authenticating as ${this.params?.slot ?? '?'}…`;
		this.log.appendSystem(
			`connected to seed ${this.seedName || '(unnamed)'}, sending Connect`,
			'system'
		);

		// Build and send Connect.
		const params = this.params!;
		const connect: ConnectPacket = {
			cmd: 'Connect',
			game: params.useTracker ? '' : params.game,
			name: params.slot,
			password: params.password ?? '',
			uuid: generateUuid(),
			version: PROTOCOL_VERSION,
			items_handling: TRACKER_ITEMS_HANDLING,
			tags: params.useTracker ? ['Tracker'] : [],
			slot_data: false
		};
		this.send([connect]);
	}

	private onConnected(p: ConnectedPacket) {
		this.status = 'connected';
		this.statusMessage = `connected as slot ${p.slot}`;
		this.mySlot = p.slot;
		this.myTeam = p.team;
		this.players = p.players ?? [];
		this.hintPoints = p.hint_points ?? 0;
		this.log.appendSystem(
			`connected as slot ${p.slot} (team ${p.team}) — ${p.players?.length ?? 0} players online`,
			'admin'
		);
	}

	private onRoomUpdate(p: RoomUpdatePacket) {
		if (p.players) this.players = p.players;
		if (typeof p.hint_points === 'number') this.hintPoints = p.hint_points;
	}

	private onPrintJSON(p: PrintJSONPacket) {
		const slotName = (slot: number) =>
			this.players.find((pl) => pl.slot === slot)?.alias ??
			this.players.find((pl) => pl.slot === slot)?.name ??
			`Slot#${slot}`;

		const text = flattenPrintJSON(p, {
			player: slotName,
			item: (id) => `Item#${id}`,
			location: (id) => `Location#${id}`
		});

		// Map AP PrintJSON types to our LogLevels.
		let level: 'info' | 'chat' | 'item' | 'hint' | 'admin' = 'info';
		let source: string | undefined;
		switch (p.type) {
			case 'Chat':
				level = 'chat';
				source = typeof p.slot === 'number' ? slotName(p.slot) : 'CHAT';
				break;
			case 'ServerChat':
				level = 'admin';
				source = 'SRV';
				break;
			case 'ItemSend':
			case 'ItemCheat':
				level = 'item';
				source = 'ITEM';
				break;
			case 'Hint':
				level = 'hint';
				source = 'HINT';
				break;
			case 'Countdown':
				level = 'admin';
				source = 'COUNT';
				break;
			default:
				level = 'info';
				break;
		}

		this.log.append({ level, source, origin: 'client', text });
	}

	// ── User input ──────────────────────────────────────────────────────

	/** Submit what the user typed in the Client command input. */
	async submit(raw: string) {
		const line = raw.trim();
		if (!line) return;

		this.pushHistory(line);

		const parsed = parseLine(line);
		if (parsed.kind === 'empty') return;

		// Echo everything locally so the user sees their input.
		if (parsed.kind === 'chat') {
			this.log.append({
				level: 'chat',
				source: 'YOU',
				origin: 'echo',
				text: parsed.text
			});
			if (this.status !== 'connected') {
				this.log.appendSystem('not connected — type !help to see commands', 'warn');
				return;
			}
			this.send([{ cmd: 'Say', text: parsed.text } as SayPacket]);
			return;
		}

		// Command path — must be `!` prefix for Client.
		if (parsed.prefix !== '!') {
			this.log.appendSystem(`Client uses ! prefix. Did you mean !${parsed.name}?`, 'warn');
			return;
		}

		this.log.append({
			level: 'admin',
			source: 'YOU',
			origin: 'echo',
			text: line
		});

		const def = this.registry.lookup('!', parsed.name);
		if (def?.status === 'coming-soon') {
			this.log.appendSystem(`!${parsed.name} is a v2 Zephyr extension, not yet wired up.`, 'warn');
			return;
		}

		if (def?.run) {
			try {
				await def.run(parsed.args, line);
			} catch (err: unknown) {
				this.log.appendSystem(`command failed: ${(err as Error).message ?? err}`, 'error');
			}
			return;
		}

		// Unknown ! commands: forward raw as Say — AP's own parser handles
		// them server-side (same path as Text Client does).
		if (this.status !== 'connected') {
			this.log.appendSystem('not connected — cannot send command', 'warn');
			return;
		}
		this.send([{ cmd: 'Say', text: line } as SayPacket]);
	}

	/** Low-level helper the client commands can use. */
	sendSay(text: string) {
		if (this.status !== 'connected') return false;
		this.send([{ cmd: 'Say', text } as SayPacket]);
		return true;
	}

	nameOfSlot(slot: number): string | undefined {
		return this.players.find((p) => p.slot === slot)?.alias ?? this.players.find((p) => p.slot === slot)?.name;
	}

	// ── Command history ─────────────────────────────────────────────────
	private loadHistory() {
		try {
			const raw = localStorage.getItem(this.historyKey);
			if (!raw) return;
			const parsed = JSON.parse(raw);
			if (Array.isArray(parsed)) this.history = parsed.filter((x) => typeof x === 'string');
		} catch {
			// ignore
		}
	}

	private pushHistory(line: string) {
		this.history = [line, ...this.history.filter((h) => h !== line)].slice(0, 100);
		try {
			localStorage.setItem(this.historyKey, JSON.stringify(this.history));
		} catch {
			// ignore
		}
	}
}
