import type { Mod } from '$lib/types';
import * as api from '$lib/api';
import { handleMultiSelect } from './multiSelect';

export function createModSelection() {
	let selectedModIds: string[] = $state([]);
	let lastClickedIndex = -1;
	let cachedSelectedMods: Map<string, Mod> = new Map();
	let multiViewIndex = $state(0);

	function getSelectedMod(uuid: string, mods: Mod[]): Mod | null {
		const fresh = mods.find((m) => m.uuid === uuid);
		if (fresh) {
			cachedSelectedMods.set(uuid, fresh);
			return fresh;
		}
		return cachedSelectedMods.get(uuid) ?? null;
	}

	function handleModClick(evt: MouseEvent, mod: Mod, index: number, sortedIds: string[]) {
		cachedSelectedMods.set(mod.uuid, mod);
		const result = handleMultiSelect(
			evt,
			mod.uuid,
			index,
			selectedModIds,
			sortedIds,
			lastClickedIndex
		);
		selectedModIds = result.selection;
		lastClickedIndex = result.lastIndex;
	}

	async function handleDepClick(author: string, name: string, mods: Mod[]): Promise<boolean> {
		let found = mods.find(
			(m) =>
				m.name.toLowerCase() === name.toLowerCase() &&
				m.author?.toLowerCase() === author.toLowerCase()
		);
		if (!found) {
			found = mods.find((m) => m.name.toLowerCase() === name.toLowerCase());
		}
		if (!found) {
			try {
				const results = await api.thunderstore.query({
					searchTerm: name,
					includeCategories: [],
					excludeCategories: [],
					includeNsfw: true,
					includeDeprecated: true,
					includeDisabled: true,
					includeEnabled: true,
					sortBy: 'downloads',
					sortOrder: 'descending',
					maxCount: 10
				});
				found =
					results.find(
						(m) =>
							m.name.toLowerCase() === name.toLowerCase() &&
							m.author?.toLowerCase() === author.toLowerCase()
					) ?? results.find((m) => m.name.toLowerCase() === name.toLowerCase());
			} catch {}
		}
		if (found) {
			cachedSelectedMods.set(found.uuid, found);
			selectedModIds = [found.uuid];
			return true;
		}
		return false;
	}

	function clearSelection() {
		selectedModIds = [];
		cachedSelectedMods.clear();
		multiViewIndex = 0;
		lastClickedIndex = -1;
	}

	function refreshCachedMods(mods: Mod[]) {
		const cachedIds = [...cachedSelectedMods.entries()].filter(
			([uuid]) => !mods.find((m) => m.uuid === uuid)
		);
		if (cachedIds.length > 0) {
			Promise.all(
				cachedIds.map(([uuid, cached]) =>
					api.thunderstore
						.query({
							searchTerm: cached.name,
							includeCategories: [],
							excludeCategories: [],
							includeNsfw: true,
							includeDeprecated: true,
							includeDisabled: true,
							includeEnabled: true,
							sortBy: 'downloads',
							sortOrder: 'descending',
							maxCount: 5
						})
						.then((results) => {
							const updated = results.find((m) => m.uuid === uuid);
							if (updated) cachedSelectedMods.set(uuid, updated);
						})
				)
			).then(() => {
				selectedModIds = [...selectedModIds];
			});
		}
	}

	return {
		get selectedModIds() {
			return selectedModIds;
		},
		set selectedModIds(v: string[]) {
			selectedModIds = v;
		},
		get multiViewIndex() {
			return multiViewIndex;
		},
		set multiViewIndex(v: number) {
			multiViewIndex = v;
		},
		get cachedSelectedMods() {
			return cachedSelectedMods;
		},
		getSelectedMod,
		handleModClick,
		handleDepClick,
		clearSelection,
		refreshCachedMods
	};
}
