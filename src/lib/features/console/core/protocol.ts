/**
 * Console protocol — Phase 1 surface.
 *
 * Only the Server half is wired to real transport today. The Client half is
 * typed so the UI can be built against stable shapes, but the WebSocket AP
 * client ships in Phase 3 (Stage 3 in CONSOLE_ARCHITECTURE.md §3).
 */

export type LogStream = 'stdout' | 'stderr';

/** Raw line emitted by the Rust runtime for every MultiServer.py output. */
export type ServerLogEvent = {
	stream: LogStream;
	line: string;
	ts_ms: number;
};

/** The Tauri event name Rust emits for every server log line. */
export const SERVER_LOG_EVENT = 'console://server-log';

/**
 * Log entry shape used inside the UI. Derived from either (a) a raw
 * ServerLogEvent parsed by the console, or (b) a synthetic echo from the
 * command input ("you typed this").
 */
export type LogLevel = 'info' | 'warn' | 'error' | 'hint' | 'item' | 'chat' | 'admin' | 'system';

export type LogEntry = {
	id: string;
	ts: number; // epoch ms
	level: LogLevel;
	source?: string; // e.g. "SRV", "YOU", "Slot#3", "CHAT"
	text: string;
	/** Origin of the entry — useful for filtering/debug. */
	origin:
		| 'server-stdout'
		| 'server-stderr'
		| 'echo'
		| 'client'
		| 'system'
		| 'bridge-stdout'
		| 'bridge-stderr';
};

/**
 * Parse a raw AP MultiServer stdout/stderr line into a classified feed entry.
 *
 * AP's MultiServer.py prints human-readable lines with a few recognizable
 * shapes. We recover the most useful structure (slot, level, event type)
 * from regex matching — knowing it's fragile, and that Phase 2 WebSocket
 * observer will replace most of this once we connect as a real tracker
 * client.
 *
 * Recognized shapes (non-exhaustive, ordered by specificity):
 *
 *   "[Team #N] Alice: hello"                    → chat
 *   "Alice (Team #N) sent Sword to Bob"         → item (source = Alice)
 *   "[Hint]: Alice's Sword is at X in Bob's world"
 *                                                → hint (source = HINT)
 *   "Notice (all): blah"                        → admin broadcast
 *   "Player Alice (Team #1) has connected"      → system (source = Alice)
 *   "Player Alice (Team #1) has disconnected"   → system
 *   "(INFO) [Server]: blah"                     → info
 *   "(WARNING) [Server]: blah"                  → warn
 *   "[err] stderr passthrough"                  → error
 */
export function classifyLine(raw: string, stream: LogStream): { level: LogLevel; source?: string; text: string } {
	if (stream === 'stderr' || raw.startsWith('[err]')) {
		return { level: 'error', source: 'ERR', text: raw.replace(/^\[err\]\s*/, '') };
	}

	// Chat: "[Team #N] <player>: <text>" or "<player>: <text>"
	const chat = raw.match(/^(?:\[Team\s*#\d+\]\s*)?([A-Za-z0-9_ \-]+):\s(.+)$/);
	if (chat && !/^(INFO|WARNING|ERROR|Notice)\b/i.test(chat[1]) && chat[1].length < 40) {
		return { level: 'chat', source: chat[1].trim(), text: chat[2] };
	}

	// Item send: "Alice sent X to Bob" or "Alice (Team #N) sent X to Bob"
	const item = raw.match(/^([A-Za-z0-9_ \-]+?)(?:\s*\(Team\s*#\d+\))?\s+sent\s+(.+?)\s+to\s+([A-Za-z0-9_ \-]+)\s*(?:\(.*\))?$/);
	if (item) {
		return {
			level: 'item',
			source: item[1].trim(),
			text: `sent ${item[2]} → ${item[3].trim()}`
		};
	}

	// Found: "Alice found X"
	const found = raw.match(/^([A-Za-z0-9_ \-]+?)(?:\s*\(Team\s*#\d+\))?\s+found\s+their\s+(.+)$/);
	if (found) {
		return {
			level: 'item',
			source: found[1].trim(),
			text: `found ${found[2]}`
		};
	}

	// Hint: "[Hint]: ...", "X's Y is at Z"
	if (/^\[Hint\]|hinted that|hint\b.*\blocated\b/i.test(raw)) {
		return { level: 'hint', source: 'HINT', text: raw.replace(/^\[Hint\]:\s*/, '') };
	}

	// Admin notice
	const notice = raw.match(/^Notice\s*(?:\([^)]+\))?:\s*(.+)$/);
	if (notice) {
		return { level: 'admin', source: 'SRV', text: notice[1] };
	}

	// Connect / Disconnect / Goal
	const conn = raw.match(/^Player\s+([A-Za-z0-9_ \-]+)\s*(?:\(Team\s*#\d+\))?\s+has\s+(connected|disconnected|finished|reached their goal).*$/i);
	if (conn) {
		const action = conn[2].toLowerCase();
		return {
			level: action === 'disconnected' ? 'warn' : 'admin',
			source: conn[1].trim(),
			text: action
		};
	}

	// Countdown
	const count = raw.match(/^(\d+)\.?$/);
	if (count) {
		return { level: 'admin', source: 'COUNT', text: count[1] };
	}

	// "(INFO) [Server]: ..." — catch-all AP log prefix
	const infoTag = raw.match(/^\((INFO|WARNING|ERROR)\)\s*(?:\[([^\]]+)\])?\s*:?\s*(.*)$/);
	if (infoTag) {
		const [, level, source, rest] = infoTag;
		const lv: LogLevel = level === 'ERROR' ? 'error' : level === 'WARNING' ? 'warn' : 'info';
		return { level: lv, source: source || undefined, text: rest };
	}

	// ERROR / WARN fallbacks
	if (/^ERROR\b/i.test(raw)) return { level: 'error', source: 'ERR', text: raw };
	if (/^WARN(ING)?\b/i.test(raw)) return { level: 'warn', source: 'WARN', text: raw };

	return { level: 'info', text: raw };
}
