import type { Mod, MarkdownType } from '$lib/types';
import { ModType } from '$lib/types';
import { convertFileSrc } from '@tauri-apps/api/core';
import games from '$lib/state/game.svelte';
import * as api from '$lib/api';
import { marked } from 'marked';
import { THUNDERSTORE_ICON_URL, ZEPHYR_GAME_ICON_URL } from '$lib/constants/api.constants';

export function isOutdated(mod: Mod): boolean {
	if (mod.versions.length === 0) {
		return false;
	}
	return mod.version !== mod.versions[0].name;
}

export function modIconSrc(mod: Mod) {
	if (mod.icon && (mod.icon.startsWith('http://') || mod.icon.startsWith('https://'))) {
		return mod.icon;
	}
	if (mod.type === 'remote') {
		const fullName = `${mod.author}-${mod.name}-${mod.version}`;
		return THUNDERSTORE_ICON_URL(fullName);
	} else if (mod.icon !== null) {
		const path = mod.enabled === false ? mod.icon + '.old' : mod.icon;
		return convertFileSrc(path);
	} else {
		return ZEPHYR_GAME_ICON_URL(games.active?.slug ?? '');
	}
}

export async function getMarkdown(mod: Mod, type: MarkdownType, useLatest = false) {
	let raw: string | null = null;

	switch (mod.type) {
		case ModType.Remote:
			raw = await api.thunderstore.getMarkdown(
				{
					packageUuid: mod.uuid,
					versionUuid: useLatest ? mod.versions[0].uuid : mod.versionUuid
				},
				type
			);
			break;

		case ModType.Local:
			raw = await api.profile.getLocalMarkdown(mod.uuid, type);
			break;
	}

	if (!raw) return raw;
	return await marked(raw);
}
