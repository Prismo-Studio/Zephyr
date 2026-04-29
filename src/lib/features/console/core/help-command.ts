import { m } from '$lib/paraglide/messages';
import type { CommandDef } from './command-registry';
import type { CommandRegistry } from './command-registry';
import type { LogStore } from './log-store.svelte';

/**
 * The server (`/`) and client (`!`) command registries both ship an
 * identical `help` command. List all ready commands for the prefix, or
 * show the signature + summary of a specific one. Factor the logic here
 * so both sides register the same definition with only the prefix and
 * example strings varying.
 */
export type HelpCapableSession = {
	registry: CommandRegistry;
	log: LogStore;
};

export function helpCommand(
	prefix: '/' | '!',
	session: HelpCapableSession,
	examples: string[]
): CommandDef {
	return {
		prefix,
		name: 'help',
		group: 'info',
		status: 'ready',
		summary: 'List commands or show details on one.',
		args: [{ name: 'cmd', optional: true }],
		examples,
		run: async (args) => {
			if (args.length === 0) {
				const all = session.registry.all(prefix).filter((c) => c.status === 'ready');
				session.log.appendSystem(
					m.console_msg_commandsList({
						count: String(all.length),
						names: all.map((c) => c.name).join(', ')
					}),
					'info'
				);
				return;
			}
			const def = session.registry.lookup(prefix, args[0]);
			if (!def) {
				session.log.appendSystem(m.console_msg_unknownCmd({ prefix, name: args[0] }), 'warn');
				return;
			}
			session.log.appendSystem(`${session.registry.signature(def)}  ·  ${def.summary}`, 'info');
		}
	};
}
