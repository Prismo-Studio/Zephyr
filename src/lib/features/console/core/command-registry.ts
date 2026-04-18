/**
 * Command registry.
 *
 * Both Server (`/`) and Client (`!`) use this same shape. Each registered
 * CommandDef is keyed by `{prefix, name}`; help-palette rendering comes
 * straight out of this registry.
 */

export type CommandGroup = 'info' | 'action' | 'social' | 'admin' | 'zephyr-ext';

export type CommandStatus = 'ready' | 'coming-soon';

export type CommandArg = {
	name: string;
	/** If true, arg may be missing. */
	optional?: boolean;
	/** Hint text used for autocomplete. */
	placeholder?: string;
};

export type CommandDef = {
	prefix: '/' | '!';
	name: string; // "players", "hint", ...
	group: CommandGroup;
	status: CommandStatus;
	summary: string;
	description?: string;
	examples?: string[];
	args: CommandArg[];
	/** Populated at dispatch time. */
	run?: (args: string[], rawLine: string) => Promise<void> | void;
};

export class CommandRegistry {
	private map = new Map<string, CommandDef>();

	register(def: CommandDef) {
		this.map.set(`${def.prefix}${def.name}`, def);
	}

	lookup(prefix: '/' | '!', name: string): CommandDef | undefined {
		return this.map.get(`${prefix}${name}`);
	}

	all(prefix?: '/' | '!'): CommandDef[] {
		const list = [...this.map.values()];
		return prefix ? list.filter((c) => c.prefix === prefix) : list;
	}

	/** Autocomplete candidates for a partial command name. Status filter lets
	 *  the help palette optionally hide `coming-soon` entries. */
	autocomplete(prefix: '/' | '!', partial: string, includeComingSoon = false): CommandDef[] {
		const p = partial.toLowerCase();
		return this.all(prefix)
			.filter((c) => (includeComingSoon ? true : c.status === 'ready'))
			.filter((c) => c.name.toLowerCase().startsWith(p))
			.sort((a, b) => a.name.localeCompare(b.name));
	}

	/** Format a command's signature for display, e.g. "/send <slot> <item>". */
	signature(def: CommandDef): string {
		const args = def.args.map((a) => (a.optional ? `[${a.name}]` : `<${a.name}>`)).join(' ');
		return `${def.prefix}${def.name}${args ? ' ' + args : ''}`;
	}
}
