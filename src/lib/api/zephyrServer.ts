/**
 * HTTP client for the Zephyr Server (local CurseForge proxy).
 * All CurseForge mod operations go through this module.
 */

let _baseUrl = 'http://localhost:3000';
let _apiKey = '';

export function configure(baseUrl: string, apiKey: string): void {
	_baseUrl = baseUrl;
	_apiKey = apiKey;
}

function getBaseUrl(): string {
	return _baseUrl.replace(/\/+$/, '');
}

function getHeaders(): Record<string, string> {
	const headers: Record<string, string> = {
		'Content-Type': 'application/json'
	};
	if (_apiKey) {
		headers['x-api-key'] = _apiKey;
	}
	return headers;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
	const url = `${getBaseUrl()}${path}`;
	const response = await fetch(url, {
		...init,
		headers: {
			...getHeaders(),
			...(init?.headers ?? {})
		}
	});

	if (!response.ok) {
		const body = await response.json().catch(() => ({ error: response.statusText }));
		throw new Error(body.error ?? `Server responded with ${response.status}`);
	}

	return response.json() as Promise<T>;
}

// ── Types matching ZephyrServer responses ──

export interface CurseForgePagination {
	index: number;
	pageSize: number;
	resultCount: number;
	totalCount: number;
}

export interface CurseForgeMod {
	id: number;
	gameId: number;
	name: string;
	slug: string;
	links: {
		websiteUrl: string | null;
		wikiUrl: string | null;
		issuesUrl: string | null;
		sourceUrl: string | null;
	};
	summary: string;
	downloadCount: number;
	categories: Array<{
		id: number;
		name: string;
		slug: string;
		iconUrl: string;
	}>;
	authors: Array<{ id: number; name: string; url: string }>;
	logo: { thumbnailUrl: string; url: string } | null;
	mainFileId: number;
	latestFiles: CurseForgeFile[];
	latestFilesIndexes: Array<{
		gameVersion: string;
		fileId: number;
		filename: string;
		releaseType: number;
	}>;
	dateCreated: string;
	dateModified: string;
	dateReleased: string;
	gamePopularityRank: number;
	isAvailable: boolean;
	thumbsUpCount: number;
	allowModDistribution: boolean | null;
}

export interface CurseForgeFile {
	id: number;
	gameId: number;
	modId: number;
	displayName: string;
	fileName: string;
	releaseType: number;
	fileDate: string;
	fileLength: number;
	downloadCount: number;
	downloadUrl: string | null;
	gameVersions: string[];
	dependencies: Array<{ modId: number; relationType: number }>;
}

export interface SearchResponse {
	data: CurseForgeMod[];
	pagination: CurseForgePagination;
}

export interface ModResponse {
	data: CurseForgeMod;
}

export interface FilesResponse {
	data: CurseForgeFile[];
	pagination: CurseForgePagination;
}

export interface InstalledMod {
	modId: number;
	fileId: number;
	name: string;
	fileName: string;
	version: string;
	gameId: number;
	installedAt: string;
	installedPath: string;
	gameDirectory: string;
}

export interface InstalledResponse {
	data: InstalledMod[];
	total: number;
}

export interface InstallResult {
	success: boolean;
	mod: {
		modId: number;
		name: string;
		fileName: string;
		installedPath: string;
	};
}

// ── API methods ──

const CURSEFORGE_GAME_IDS: Record<string, number> = {
	minecraft: 432,
	'minecraft-bedrock': 78022,
	'minecraft-dungeons': 69271,
	worldofwarcraft: 1,
	wow: 1,
	stardewvalley: 669,
	'stardew-valley': 669,
	terraria: 431,
	palworld: 85196,
	'helldivers-2': 85440,
	helldivers2: 85440,
	'among-us': 69761,
	amongus: 69761,
	'the-elder-scrolls-online': 455,
	teso: 455,
	'starcraft-ii': 65,
	sc2: 65,
	'civilization-vi': 727,
	civ6: 727,
	'kerbal-space-program': 4401,
	kerbal: 4401,
	'darkest-dungeon': 608,
	darkestdungeon: 608,
	'surviving-mars': 61489,
	sims4: 78062,
	'the-sims-4': 78062,
	valheim: 73492,
	'risk-of-rain-2': 69069,
	riskofrain2: 69069,
	'cities-skylines-ii': 78018,
	'cities-skylines': 702
};

export function getCurseForgeGameId(slug: string | null | undefined): number | null {
	if (!slug) return null;
	return CURSEFORGE_GAME_IDS[slug] ?? null;
}

export async function healthCheck(): Promise<boolean> {
	try {
		await request<{ status: string }>('/health');
		return true;
	} catch {
		return false;
	}
}

export async function searchMods(
	query: string,
	gameId: number = 432,
	page: number = 0,
	pageSize: number = 25
): Promise<SearchResponse> {
	const params = new URLSearchParams({
		gameId: String(gameId),
		page: String(page),
		pageSize: String(pageSize)
	});
	if (query) params.set('q', query);
	return request<SearchResponse>(`/mods/search?${params}`);
}

export async function getMod(modId: number): Promise<ModResponse> {
	return request<ModResponse>(`/mods/${modId}`);
}

export async function getModFiles(
	modId: number,
	page: number = 0,
	pageSize: number = 50
): Promise<FilesResponse> {
	const params = new URLSearchParams({
		page: String(page),
		pageSize: String(pageSize)
	});
	return request<FilesResponse>(`/mods/${modId}/files?${params}`);
}

export async function installMod(
	modId: number,
	fileId: number,
	gameDirectory?: string
): Promise<InstallResult> {
	return request<InstallResult>(`/mods/${modId}/install`, {
		method: 'POST',
		body: JSON.stringify({
			fileId,
			...(gameDirectory ? { gameDirectory } : {})
		})
	});
}

export async function getInstalled(): Promise<InstalledResponse> {
	return request<InstalledResponse>('/installed');
}

export async function removeMod(modId: number): Promise<{ success: boolean }> {
	return request<{ success: boolean }>(`/installed/${modId}`, {
		method: 'DELETE'
	});
}

export async function launchGame(
	gameDirectory: string,
	javaPath?: string,
	extraArgs?: string[]
): Promise<{ success: boolean; pid: number }> {
	return request<{ success: boolean; pid: number }>('/launch', {
		method: 'POST',
		body: JSON.stringify({
			gameDirectory,
			...(javaPath ? { javaPath } : {}),
			...(extraArgs ? { extraArgs } : {})
		})
	});
}
