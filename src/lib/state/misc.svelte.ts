import type { ConfigEntryId, QueryModsArgsWithoutMax } from '$lib/types';
import { PersistedState } from 'runed';
import profiles from './profile.svelte';

/**
 * Creates a PersistedState that merges saved values with defaults on load,
 * so new keys added between versions are always present.
 */
function createMergedPersistedState<T extends Record<string, unknown>>(
	key: string,
	defaults: T
): PersistedState<T> {
	const raw = localStorage.getItem(key);
	if (raw) {
		try {
			const saved = JSON.parse(raw) as Partial<T>;
			const merged = { ...defaults, ...saved };
			// Always reset transient filters to avoid stale values across sessions
			if ('searchTerm' in defaults) (merged as any).searchTerm = '';
			if ('includeCategories' in defaults) (merged as any).includeCategories = [];
			if ('excludeCategories' in defaults) (merged as any).excludeCategories = [];
			localStorage.setItem(key, JSON.stringify(merged));
		} catch {
			localStorage.setItem(key, JSON.stringify(defaults));
		}
	}
	return new PersistedState<T>(key, defaults);
}

export const apiKeyDialog = $state({
	open: false
});

export const config: {
	expandedEntry: ConfigEntryId | null;
} = $state({
	expandedEntry: null
});

export const updateBanner = $state({
	threshold: 0
});

export const modQuery = createMergedPersistedState<QueryModsArgsWithoutMax>('modQuery', {
	searchTerm: '',
	includeCategories: [],
	excludeCategories: [],
	includeNsfw: false,
	includeDeprecated: false,
	includeEnabled: false,
	includeDisabled: false,
	sortBy: 'rating',
	sortOrder: 'descending'
});

// Locally pinned mod UUIDs per profile (always shown at top of mod list)
export const pinnedModsMap = new PersistedState<Record<string, string[]>>(
	'pinnedModsPerProfile',
	{}
);

function getActiveProfileKey(): string {
	// Lazy import to avoid circular deps
	const id = profiles?.activeId;
	return id != null ? String(id) : '_default';
}

export const pinnedMods = {
	get current(): string[] {
		return pinnedModsMap.current[getActiveProfileKey()] ?? [];
	}
};

export function togglePin(uuid: string) {
	const key = getActiveProfileKey();
	const map = { ...pinnedModsMap.current };
	const current = map[key] ?? [];
	if (current.includes(uuid)) {
		map[key] = current.filter((id) => id !== uuid);
	} else {
		map[key] = [...current, uuid];
	}
	pinnedModsMap.current = map;
}

export function isModPinned(uuid: string): boolean {
	return pinnedMods.current.includes(uuid);
}

export const viewMode = new PersistedState<'list' | 'grid'>('modViewMode', 'list');

// Global install state — lets components react to install start/end
export const installState = $state({
	active: false
});

export const profileQuery = createMergedPersistedState<QueryModsArgsWithoutMax>('profileQuery', {
	searchTerm: '',
	includeCategories: [],
	excludeCategories: [],
	includeNsfw: true,
	includeDeprecated: true,
	includeEnabled: true,
	includeDisabled: true,
	sortBy: 'custom',
	sortOrder: 'descending'
});
