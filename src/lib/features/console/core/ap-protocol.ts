/**
 * Archipelago network protocol (subset we actually use).
 *
 * Reference: https://github.com/ArchipelagoMW/Archipelago/blob/main/docs/network%20protocol.md
 *
 * We stay on the client-only half of the protocol. The packet shapes below
 * cover enough to:
 *   - Connect to a server
 *   - Receive chat + item + hint events
 *   - Send chat and `!commands`
 *
 * The `Tracker` tag lets us connect without needing the seed to have a slot
 * marked as spectator, which keeps the UX simple (no apworld edits).
 */

/** Arbitrary JSON value — used in packet payloads we don't type fully. */
type JsonValue = string | number | boolean | null | JsonValue[] | { [k: string]: JsonValue };

// ── Client → Server packets ──────────────────────────────────────────

export type ConnectPacket = {
	cmd: 'Connect';
	game: string;
	name: string;
	password: string;
	uuid: string;
	version: { major: number; minor: number; build: number; class: 'Version' };
	items_handling: number;
	tags: string[];
	slot_data?: boolean;
};

export type SayPacket = {
	cmd: 'Say';
	text: string;
};

export type SyncPacket = {
	cmd: 'Sync';
};

export type LocationScoutsPacket = {
	cmd: 'LocationScouts';
	locations: number[];
	create_as_hint: 0 | 1 | 2;
};

export type StatusUpdatePacket = {
	cmd: 'StatusUpdate';
	status: number; // 5 = ClientGoal, 20 = ClientReady, 30 = ClientPlaying
};

export type ClientPacket =
	| ConnectPacket
	| SayPacket
	| SyncPacket
	| LocationScoutsPacket
	| StatusUpdatePacket;

// ── Server → Client packets ──────────────────────────────────────────

export type RoomInfoPacket = {
	cmd: 'RoomInfo';
	version: { major: number; minor: number; build: number };
	tags: string[];
	password: boolean;
	permissions: Record<string, number>;
	hint_cost: number;
	location_check_points: number;
	games: string[];
	datapackage_checksums: Record<string, string>;
	seed_name: string;
	time: number;
};

export type ConnectedPacket = {
	cmd: 'Connected';
	team: number;
	slot: number;
	players: { team: number; slot: number; alias: string; name: string }[];
	missing_locations: number[];
	checked_locations: number[];
	slot_data: JsonValue;
	slot_info: Record<string, { name: string; game: string; type: number; group_members: number[] }>;
	hint_points: number;
};

export type ConnectionRefusedPacket = {
	cmd: 'ConnectionRefused';
	errors?: string[];
};

export type ReceivedItemsPacket = {
	cmd: 'ReceivedItems';
	index: number;
	items: { item: number; location: number; player: number; flags: number }[];
};

export type PrintJSONPacket = {
	cmd: 'PrintJSON';
	type?: string; // "ItemSend", "Hint", "Chat", "ServerChat", "Countdown", ...
	data: {
		type?: string;
		text?: string;
		player?: number;
		item?: number;
		location?: number;
		flags?: number;
	}[];
	receiving?: number;
	item?: { item: number; location: number; player: number; flags: number };
	slot?: number;
	team?: number;
	message?: string;
	tags?: string[];
	countdown?: number;
};

export type RoomUpdatePacket = {
	cmd: 'RoomUpdate';
} & Partial<Omit<RoomInfoPacket, 'cmd'>> & {
		players?: { team: number; slot: number; alias: string; name: string }[];
		checked_locations?: number[];
		hint_points?: number;
	};

export type BouncedPacket = {
	cmd: 'Bounced';
	games?: string[];
	slots?: number[];
	tags?: string[];
	data: JsonValue;
};

export type InvalidPacketPacket = {
	cmd: 'InvalidPacket';
	type: string;
	original_cmd?: string;
	text: string;
};

export type ServerPacket =
	| RoomInfoPacket
	| ConnectedPacket
	| ConnectionRefusedPacket
	| ReceivedItemsPacket
	| PrintJSONPacket
	| RoomUpdatePacket
	| BouncedPacket
	| InvalidPacketPacket
	| { cmd: string; [k: string]: JsonValue };

// ── Protocol constants ───────────────────────────────────────────────

/** AP protocol version we target. Bumped to 0.6.0 to match current AP. */
export const PROTOCOL_VERSION = { major: 0, minor: 6, build: 0, class: 'Version' as const };

/**
 * items_handling bitmasks per AP network protocol:
 *   0b000 (0) — server does NOT send ReceivedItems. Correct for trackers /
 *               admin consoles that sit alongside a real game client. The
 *               real client (SNIClient etc.) handles actual item delivery.
 *   0b111 (7) — full remote-items mode. Correct only when this client is the
 *               slot's actual item receiver and delivers them in-game.
 *
 * Using 0b111 for a tracker lies to the server about the slot's capabilities
 * and can interfere with server-side bookkeeping. Always use
 * TRACKER_ITEMS_HANDLING when connecting as a tracker.
 */
export const TRACKER_ITEMS_HANDLING = 0b000;
export const PLAYER_ITEMS_HANDLING = 0b111;

// ── Parsing helpers ──────────────────────────────────────────────────

/** AP sends JSON arrays of packets. Parse defensively — drop anything that
 *  isn't an object with a string `cmd`. */
export function parseFrame(raw: string): ServerPacket[] {
	try {
		const parsed = JSON.parse(raw);
		if (!Array.isArray(parsed)) return [];
		return parsed.filter(
			(p): p is ServerPacket => !!p && typeof p === 'object' && typeof p.cmd === 'string'
		);
	} catch {
		return [];
	}
}

/** Serialize a list of outbound packets for the wire. */
export function serializeFrame(packets: ClientPacket[]): string {
	return JSON.stringify(packets);
}

/** Generate a UUID-ish string that AP accepts as a client identifier. We
 *  don't need cryptographic strength — just uniqueness per session. */
export function generateUuid(): string {
	if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
		return crypto.randomUUID();
	}
	return `zephyr-${Date.now()}-${Math.floor(Math.random() * 1e9).toString(36)}`;
}

// ── PrintJSON flattener ──────────────────────────────────────────────

/**
 * Take a PrintJSON packet and return a plain human-readable string, resolving
 * player/item/location IDs via a lookup callback (stubbed to IDs if absent).
 *
 * AP's `data` is an array of typed chunks: `{type: 'player_id', text: '3'}`,
 * `{type: 'item_id', text: '42', player: 3}`, `{type: 'text', text: 'found'}`,
 * etc. We resolve and join. This is enough for feed rendering.
 */
export function flattenPrintJSON(
	p: PrintJSONPacket,
	resolvers: {
		player?: (slot: number) => string;
		item?: (id: number, slot: number) => string;
		location?: (id: number, slot: number) => string;
	} = {}
): string {
	if (typeof p.message === 'string') return p.message;
	if (!Array.isArray(p.data)) return '';
	const parts: string[] = [];
	for (const chunk of p.data) {
		if (!chunk || typeof chunk !== 'object') continue;
		const text = chunk.text ?? '';
		switch (chunk.type) {
			case 'player_id': {
				const id = Number(text);
				parts.push(resolvers.player?.(id) ?? `Slot#${id}`);
				break;
			}
			case 'item_id': {
				const id = Number(text);
				const playerSlot = typeof chunk.player === 'number' ? chunk.player : 0;
				parts.push(resolvers.item?.(id, playerSlot) ?? `Item#${id}`);
				break;
			}
			case 'location_id': {
				const id = Number(text);
				const playerSlot = typeof chunk.player === 'number' ? chunk.player : 0;
				parts.push(resolvers.location?.(id, playerSlot) ?? `Location#${id}`);
				break;
			}
			case 'player_name':
			case 'item_name':
			case 'location_name':
			case 'entrance_name':
			case 'text':
			default:
				parts.push(text);
				break;
		}
	}
	return parts.join('');
}
