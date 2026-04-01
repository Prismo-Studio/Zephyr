import type { ConfigEntryId, QueryModsArgsWithoutMax } from '$lib/types';
import { PersistedState } from 'runed';
import profiles from './profile.svelte';

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

// Locally pinned mod UUIDs per profile (always shown at top of mod list)
export const pinnedModsMap = new PersistedState<Record<string, string[]>>('pinnedModsPerProfile', {});

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
