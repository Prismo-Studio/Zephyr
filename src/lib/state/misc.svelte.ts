import type { ConfigEntryId, QueryModsArgsWithoutMax } from '$lib/types';
import { PersistedState } from 'runed';

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

export const modQuery = new PersistedState<QueryModsArgsWithoutMax>('modQuery', {
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

// Locally pinned mod UUIDs (always shown at top of mod list)
export const pinnedMods = new PersistedState<string[]>('pinnedMods', []);

export function togglePin(uuid: string) {
	const current = pinnedMods.current;
	if (current.includes(uuid)) {
		pinnedMods.current = current.filter((id) => id !== uuid);
	} else {
		pinnedMods.current = [...current, uuid];
	}
}

export function isModPinned(uuid: string): boolean {
	return pinnedMods.current.includes(uuid);
}

export const profileQuery = new PersistedState<QueryModsArgsWithoutMax>('profileQuery', {
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
