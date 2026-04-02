import type { Game } from '$lib/types';
import games from '$lib/state/game.svelte';
import {
	THUNDERSTORE_MOD_URL,
	THUNDERSTORE_ICON_URL,
	ZEPHYR_GAME_ICON_URL,
	DISCORD_AVATAR_URL
} from '$lib/constants/api.constants';

export function communityUrl(path: string) {
	return THUNDERSTORE_MOD_URL(games.active?.slug ?? '', path);
}

export function gameIconSrc(game: Game) {
	return ZEPHYR_GAME_ICON_URL(game.slug);
}

export function thunderstoreIconUrl(fullName: string) {
	return THUNDERSTORE_ICON_URL(fullName);
}

export function discordAvatarUrl(discordId: string, avatar: string) {
	return DISCORD_AVATAR_URL(discordId, avatar);
}
