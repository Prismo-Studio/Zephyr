import { invoke } from '$lib/invoke';

export type SourceId = 'thunderstore' | 'nexusmods' | 'curseforge' | 'github' | 'local';

export type SourceInfo = {
	id: SourceId;
	displayName: string;
	isEnabled: boolean;
	requiresAuth: boolean;
	isAuthenticated: boolean;
	supportedGames: string[] | null;
};

export type UnifiedMod = {
	sourceId: SourceId;
	externalId: string;
	name: string;
	author: string;
	description: string | null;
	version: string;
	versions: UnifiedModVersion[];
	categories: string[];
	downloads: number | null;
	rating: number | null;
	iconUrl: string | null;
	websiteUrl: string | null;
	dateUpdated: string | null;
	dateCreated: string | null;
	fileSize: number;
	isDeprecated: boolean;
	isNsfw: boolean;
	dependencies: string[];
};

export type UnifiedModVersion = {
	version: string;
	externalId: string;
	dateCreated: string | null;
	downloads: number | null;
	fileSize: number;
};

export type SearchFilters = {
	searchTerm: string;
	categories: string[];
	sortBy: 'downloads' | 'rating' | 'newest' | 'updated' | 'name';
	sortOrder: 'descending' | 'ascending';
	includeNsfw: boolean;
	includeDeprecated: boolean;
	maxCount: number;
	sources: SourceId[];
	gameSlug?: string;
	offset?: number;
};

export type SearchResult = {
	mods: UnifiedMod[];
	source: SourceId;
	totalCount: number | null;
};

export type SourceGame = {
	name: string;
	slug: string;
	iconUrl: string | null;
	modCount: number;
};

export const getSources = () => invoke<SourceInfo[]>('get_sources');

export const searchSources = (filters: SearchFilters) =>
	invoke<SearchResult[]>('search_sources', { filters });

export const getSourceModDescription = (source: SourceId, externalId: string) =>
	invoke<string | null>('get_source_mod_description', { source, externalId });

export const getSourceModChangelog = (source: SourceId, externalId: string, fileId: string) =>
	invoke<string | null>('get_source_mod_changelog', { source, externalId, fileId });

// export const getNexusmodsGames = () => invoke<SourceGame[]>('get_nexusmods_games');

export const getCurseforgeGames = () => invoke<SourceGame[]>('get_curseforge_games');
