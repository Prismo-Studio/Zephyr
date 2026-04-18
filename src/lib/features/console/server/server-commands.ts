import type { ServerSession } from './server-session.svelte';

/**
 * Register every Server-side command into the session's registry.
 *
 * v1 covers the AP-native admin commands. Each one dispatches by writing the
 * raw line to `MultiServer.py`'s stdin — we trust AP's own parser, we only
 * layer UX (echo, validation-light, help) on top.
 *
 * Zephyr extensions (Phase 2+) are registered as `status: 'coming-soon'` so
 * they show up in the help palette but can't actually run yet.
 */
export function registerServerCommands(session: ServerSession) {
	const r = session.registry;

	// ── AP-native info commands ─────────────────────────────────────────
	r.register({
		prefix: '/',
		name: 'help',
		group: 'info',
		status: 'ready',
		summary: 'List commands or show details on one.',
		args: [{ name: 'cmd', optional: true }],
		examples: ['/help', '/help send'],
		run: async (args) => {
			if (args.length === 0) {
				// The help palette already renders the full list. A textual fallback
				// in the feed stays useful for muscle-memory.
				const all = session.registry.all('/').filter((c) => c.status === 'ready');
				session.log.appendSystem(
					`${all.length} commands: ${all.map((c) => c.name).join(', ')}. Press Ctrl+/ for the palette.`,
					'info'
				);
			} else {
				const def = session.registry.lookup('/', args[0]);
				if (!def) {
					session.log.appendSystem(`unknown command: /${args[0]}`, 'warn');
					return;
				}
				session.log.appendSystem(
					`${session.registry.signature(def)}  —  ${def.summary}`,
					'info'
				);
			}
		}
	});

	r.register({
		prefix: '/',
		name: 'players',
		group: 'info',
		status: 'ready',
		summary: 'List connected slots with their status.',
		args: [],
		examples: ['/players']
	});

	r.register({
		prefix: '/',
		name: 'status',
		group: 'info',
		status: 'ready',
		summary: 'Show server status, hosted seed, and uptime.',
		args: [],
		examples: ['/status']
	});

	// ── AP-native action commands ──────────────────────────────────────
	r.register({
		prefix: '/',
		name: 'send',
		group: 'action',
		status: 'ready',
		summary: 'Grant an item to a slot (admin).',
		description: 'Dispatches the named item to the slot as if they had found it normally.',
		args: [
			{ name: 'slot' },
			{ name: 'item' }
		],
		examples: ['/send 3 "Progressive Sword"']
	});

	r.register({
		prefix: '/',
		name: 'send_location',
		group: 'action',
		status: 'ready',
		summary: 'Force-check a location on behalf of a slot.',
		args: [{ name: 'slot' }, { name: 'location' }],
		examples: ['/send_location 3 "Hyrule Castle Dungeon"']
	});

	r.register({
		prefix: '/',
		name: 'hint',
		group: 'action',
		status: 'ready',
		summary: 'Create a hint for a slot at admin cost.',
		args: [{ name: 'slot' }, { name: 'item' }],
		examples: ['/hint 1 "Bow of Light"']
	});

	r.register({
		prefix: '/',
		name: 'release',
		group: 'action',
		status: 'ready',
		summary: 'Release a slot\u2019s remaining items to everyone.',
		args: [{ name: 'slot' }]
	});

	r.register({
		prefix: '/',
		name: 'collect',
		group: 'action',
		status: 'ready',
		summary: 'Force-collect a slot\u2019s remaining items for them.',
		args: [{ name: 'slot' }]
	});

	r.register({
		prefix: '/',
		name: 'forfeit',
		group: 'action',
		status: 'ready',
		summary: 'Mark a slot as forfeited.',
		args: [{ name: 'slot' }]
	});

	// ── AP-native admin commands ───────────────────────────────────────
	r.register({
		prefix: '/',
		name: 'save',
		group: 'admin',
		status: 'ready',
		summary: 'Persist multidata state to disk now.',
		args: []
	});

	r.register({
		prefix: '/',
		name: 'exit',
		group: 'admin',
		status: 'ready',
		summary: 'Shut down the server cleanly.',
		args: []
	});

	r.register({
		prefix: '/',
		name: 'option',
		group: 'admin',
		status: 'ready',
		summary: 'Read or set a server option at runtime.',
		args: [{ name: 'key' }, { name: 'value', optional: true }],
		examples: ['/option password hunter2', '/option release_mode auto']
	});

	r.register({
		prefix: '/',
		name: 'broadcast',
		group: 'social',
		status: 'ready',
		summary: 'Send a Notice line to all connected clients.',
		args: [{ name: 'msg' }],
		examples: ['/broadcast "break in 5"']
	});

	r.register({
		prefix: '/',
		name: 'kick',
		group: 'admin',
		status: 'ready',
		summary: 'Disconnect a slot.',
		args: [{ name: 'slot' }]
	});

	r.register({
		prefix: '/',
		name: 'unkick',
		group: 'admin',
		status: 'ready',
		summary: 'Allow a previously kicked slot to reconnect.',
		args: [{ name: 'slot' }]
	});

	// ── Zephyr v2 extensions — picked per Phase-0 §12 ──────────────────
	r.register({
		prefix: '/',
		name: 'chapter',
		group: 'zephyr-ext',
		status: 'coming-soon',
		summary: 'Emit a streamer chapter marker (OBS/Streamlabs-exploitable).',
		args: [{ name: 'title' }],
		examples: ['/chapter "Ch. 2 — Dark World Unlocked"']
	});

	r.register({
		prefix: '/',
		name: 'bounty',
		group: 'zephyr-ext',
		status: 'coming-soon',
		summary: 'Put a bounty on an item; first finder gets a reward.',
		args: [{ name: 'put|clear' }, { name: 'item', optional: true }, { name: 'reward', optional: true }]
	});

	r.register({
		prefix: '/',
		name: 'sync_profile',
		group: 'zephyr-ext',
		status: 'coming-soon',
		summary: 'Push a Zephyr mod profile to every connected client.',
		args: [{ name: 'profile_id' }]
	});

	r.register({
		prefix: '/',
		name: 'rewind',
		group: 'zephyr-ext',
		status: 'coming-soon',
		summary: 'Rollback server state to a named checkpoint.',
		args: [{ name: 'checkpoint' }]
	});
}
