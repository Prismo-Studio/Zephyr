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

export const getSources = () => invoke<SourceInfo[]>('get_sources');
