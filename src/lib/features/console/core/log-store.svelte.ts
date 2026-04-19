import type { LogEntry, LogLevel } from './protocol';

const MAX_ENTRIES = 2000; // performance budget per CONSOLE_ARCHITECTURE.md §11

let seq = 0;
function nextId(): string {
	seq = (seq + 1) | 0;
	return `e${Date.now().toString(36)}${seq.toString(36)}`;
}

/** Reactive bounded log buffer + derived filters. Shared between the Server
 *  and Client views via distinct instances (see server-session / client-session). */
export class LogStore {
	entries: LogEntry[] = $state([]);
	/** Active source filter — when set, only entries with that source are visible. */
	sourceFilter: string | null = $state(null);
	/** Active free-text search. */
	search = $state('');

	append(entry: Omit<LogEntry, 'id' | 'ts'> & { ts?: number }) {
		this.entries.push({
			...entry,
			id: nextId(),
			ts: entry.ts ?? Date.now()
		});
		if (this.entries.length > MAX_ENTRIES) {
			this.entries.splice(0, this.entries.length - MAX_ENTRIES);
		}
	}

	appendSystem(text: string, level: LogLevel = 'system') {
		this.append({ level, source: 'SYS', text, origin: 'system' });
	}

	clear() {
		this.entries = [];
	}

	filtered = $derived.by(() => {
		const src = this.sourceFilter;
		const q = this.search.trim().toLowerCase();
		if (!src && !q) return this.entries;
		return this.entries.filter((e) => {
			if (src && e.source !== src) return false;
			if (q && !e.text.toLowerCase().includes(q) && !(e.source ?? '').toLowerCase().includes(q))
				return false;
			return true;
		});
	});

	distinctSources = $derived.by(() => {
		const seen = new Set<string>();
		for (const e of this.entries) {
			if (e.source) seen.add(e.source);
		}
		return [...seen];
	});
}
