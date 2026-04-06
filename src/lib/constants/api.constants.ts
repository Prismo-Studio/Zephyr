export const THUNDERSTORE_BASE_URL = 'https://thunderstore.io';
export const THUNDERSTORE_CDN_URL = 'https://gcdn.thunderstore.io';

export const THUNDERSTORE_COMMUNITY_URL = (slug: string) => `${THUNDERSTORE_BASE_URL}/c/${slug}`;

export const THUNDERSTORE_MOD_URL = (slug: string, path: string) =>
	`${THUNDERSTORE_BASE_URL}/c/${slug}/p/${path}/`;

export const THUNDERSTORE_CATEGORY_URL = (slug: string) =>
	`${THUNDERSTORE_BASE_URL}/api/experimental/community/${slug}/category/`;

export const THUNDERSTORE_ICON_URL = (fullName: string) =>
	`${THUNDERSTORE_CDN_URL}/live/repository/icons/${fullName}.png`;

export const ZEPHYR_GAME_ICON_URL = (slug: string) =>
	`https://raw.githubusercontent.com/Prismo-Studio/Zephyr/refs/heads/dev/images/games/${slug}.webp`;

export const DISCORD_AVATAR_URL = (discordId: string, avatar: string) =>
	`https://cdn.discordapp.com/avatars/${discordId}/${avatar}.png`;

export const ZEPHYR_SERVER_DEFAULT_URL = 'http://localhost:3000';

export const CURSEFORGE_GAME_IDS: Record<string, number> = {
	minecraft: 432,
	'minecraft-bedrock': 78022,
	'minecraft-dungeons': 69271,
	worldofwarcraft: 1,
	stardewvalley: 669,
	terraria: 431,
	palworld: 85196,
	'helldivers-2': 85440,
	'among-us': 69761,
	'the-elder-scrolls-online': 455,
	'starcraft-ii': 65,
	'civilization-vi': 727,
	'kerbal-space-program': 4401,
	'darkest-dungeon': 608,
	'surviving-mars': 61489,
	sims4: 78062
};
