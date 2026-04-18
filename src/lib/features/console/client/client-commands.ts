import type { ClientSession } from './client-session.svelte';

/**
 * Register every Client-side command (`!` prefix) into the session's
 * registry. Most are forwarded as `Say` packets — AP's server parses them
 * natively, so we don't duplicate logic here.
 */
export function registerClientCommands(session: ClientSession) {
	const r = session.registry;

	// ── Info ────────────────────────────────────────────────────────────
	r.register({
		prefix: '!',
		name: 'help',
		group: 'info',
		status: 'ready',
		summary: 'List commands or show details on one.',
		args: [{ name: 'cmd', optional: true }],
		examples: ['!help', '!help hint'],
		run: async (args) => {
			if (args.length === 0) {
				const all = session.registry.all('!').filter((c) => c.status === 'ready');
				session.log.appendSystem(
					`${all.length} commands: ${all.map((c) => c.name).join(', ')}. Press Ctrl+/ for the palette.`,
					'info'
				);
			} else {
				const def = session.registry.lookup('!', args[0]);
				if (!def) {
					session.log.appendSystem(`unknown command: !${args[0]}`, 'warn');
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
		prefix: '!',
		name: 'remaining',
		group: 'info',
		status: 'ready',
		summary: 'List locations still open for your slot.',
		args: []
	});

	r.register({
		prefix: '!',
		name: 'missing',
		group: 'info',
		status: 'ready',
		summary: 'List your unchecked locations.',
		args: []
	});

	r.register({
		prefix: '!',
		name: 'items',
		group: 'info',
		status: 'ready',
		summary: 'List items you have received.',
		args: []
	});

	r.register({
		prefix: '!',
		name: 'locations',
		group: 'info',
		status: 'ready',
		summary: 'List all locations in your seed.',
		args: []
	});

	// ── Action ──────────────────────────────────────────────────────────
	r.register({
		prefix: '!',
		name: 'hint',
		group: 'action',
		status: 'ready',
		summary: 'Request a hint for an item (spends hint points).',
		args: [{ name: 'item' }],
		examples: ['!hint "Bow of Light"']
	});

	r.register({
		prefix: '!',
		name: 'hint_location',
		group: 'action',
		status: 'ready',
		summary: 'Request a hint on what a location contains.',
		args: [{ name: 'location' }],
		examples: ['!hint_location "Hyrule Castle"']
	});

	r.register({
		prefix: '!',
		name: 'release',
		group: 'action',
		status: 'ready',
		summary: 'Release your remaining items to everyone.',
		args: []
	});

	r.register({
		prefix: '!',
		name: 'collect',
		group: 'action',
		status: 'ready',
		summary: 'Collect your own remaining items from the server.',
		args: []
	});

	r.register({
		prefix: '!',
		name: 'countdown',
		group: 'social',
		status: 'ready',
		summary: 'Start a shared countdown.',
		args: [{ name: 'seconds', optional: true }],
		examples: ['!countdown', '!countdown 10']
	});

	r.register({
		prefix: '!',
		name: 'admin',
		group: 'social',
		status: 'ready',
		summary: 'Send an admin command request to the server host.',
		args: [{ name: 'msg' }],
		examples: ['!admin release me']
	});

	// ── Zephyr Extras (v2) ─────────────────────────────────────────────
	r.register({
		prefix: '!',
		name: 'focus',
		group: 'zephyr-ext',
		status: 'coming-soon',
		summary: 'Filter your feed to events from one mod.',
		args: [{ name: 'mod' }]
	});

	r.register({
		prefix: '!',
		name: 'mod_status',
		group: 'zephyr-ext',
		status: 'coming-soon',
		summary: 'Show each slot\u2019s loaded mods with hash match/mismatch.',
		args: [{ name: 'slot', optional: true }]
	});

	r.register({
		prefix: '!',
		name: 'timeline',
		group: 'zephyr-ext',
		status: 'coming-soon',
		summary: 'Show a scrubbable timeline of your session events.',
		args: []
	});

	r.register({
		prefix: '!',
		name: 'replay',
		group: 'zephyr-ext',
		status: 'coming-soon',
		summary: 'Start/stop recording a .zephyr-run of this session.',
		args: [{ name: 'start|stop' }]
	});
}
