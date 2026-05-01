import type { Mod } from '$lib/types';
import type { UnifiedMod } from '$lib/api/sources';
import type { CurseForgeMod } from '$lib/api/zephyrServer';

/**
 * Strip simple HTML tags and BBCode-style brackets from API descriptions.
 * Different mod sources return descriptions with inconsistent formatting.
 */
export function cleanDescription(desc: string | null): string | null {
	if (!desc) return null;
	return desc
		.replace(/<br\s*\/?>/gi, ' ')
		.replace(/<[^>]*>/g, '')
		.replace(/\[.*?\]/g, '')
		.trim();
}

export function curseForgeModToMod(cfMod: CurseForgeMod): Mod {
	const latestFile = cfMod.latestFiles[0];
	return {
		name: cfMod.name,
		description: cleanDescription(cfMod.summary),
		categories: cfMod.categories.map((c) => c.name),
		version: latestFile?.displayName ?? null,
		author: cfMod.authors[0]?.name ?? null,
		rating: cfMod.thumbsUpCount > 0 ? cfMod.thumbsUpCount : null,
		downloads: cfMod.downloadCount,
		fileSize: latestFile?.fileLength ?? 0,
		websiteUrl: cfMod.links.websiteUrl,
		donateUrl: null,
		dependencies:
			latestFile?.dependencies?.filter((d) => d.relationType === 3).map((d) => String(d.modId)) ??
			[],
		isPinned: false,
		isDeprecated: false,
		isInstalled: undefined,
		containsNsfw: false,
		uuid: `zephyr-server:${cfMod.id}`,
		versionUuid: String(latestFile?.id ?? cfMod.id),
		lastUpdated: cfMod.dateModified,
		versions: cfMod.latestFiles.map((f) => ({ name: f.displayName, uuid: String(f.id) })),
		type: 'remote' as Mod['type'],
		enabled: null,
		icon: cfMod.logo?.thumbnailUrl ?? null,
		configFile: null
	};
}

export function unifiedToMod(u: UnifiedMod, source?: string): Mod {
	const sourceId = source ?? u.sourceId ?? 'thunderstore';
	return {
		name: u.name,
		description: cleanDescription(u.description),
		categories: u.categories,
		version: u.version,
		author: u.author,
		rating: u.rating ? Number(u.rating) : null,
		downloads: u.downloads ? Number(u.downloads) : null,
		fileSize: u.fileSize,
		websiteUrl: u.websiteUrl,
		donateUrl: null,
		dependencies: u.dependencies,
		isPinned: false,
		isDeprecated: u.isDeprecated,
		isInstalled: undefined,
		containsNsfw: u.isNsfw,
		uuid: `${sourceId}:${u.externalId}`,
		versionUuid: u.versions[0]?.externalId ?? u.externalId,
		lastUpdated: u.dateUpdated,
		versions: u.versions.map((v) => ({ name: v.version, uuid: v.externalId })),
		type: 'remote' as Mod['type'],
		enabled: null,
		icon: u.iconUrl,
		configFile: null
	};
}

export function isCurseForgeMod(mod: Mod): boolean {
	return mod.uuid.startsWith('curseforge:');
}

export function isServerMod(mod: Mod): boolean {
	return mod.uuid.startsWith('zephyr-server:');
}
