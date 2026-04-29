import type { Mod, AvailableUpdate, Dependant } from './mod.types';

export type ProfileInfo = {
	id: number;
	name: string;
	modCount: number;
	sync: SyncProfileInfo | null;
	customArgs: string[];
	customArgsEnabled: boolean;
	missing: boolean;
	icon: string | null;
};

export type SyncProfileInfo = {
	id: string;
	owner: SyncUser;
	syncedAt: string;
	updatedAt: string;
	missing: boolean;
};

export type ListedSyncProfile = {
	id: string;
	name: string;
	community: string;
	createdAt: string;
	updatedAt: string;
};

export type SyncUser = {
	discordId: string;
	name: string;
	displayName: string;
	avatar: string | null;
};

export type ManagedGameInfo = {
	profiles: ProfileInfo[];
	activeId: number;
};

export type ProfileQuery = {
	mods: Mod[];
	totalModCount: number;
	filteredModCount: number;
	unknownMods: Dependant[];
	updates: AvailableUpdate[];
};

export type ImportData =
	| ({ type: 'legacy' } & LegacyImportData)
	| ({ type: 'sync' } & SyncImportData);

export type LegacyImportData = {
	manifest: ProfileManifest;
	path: string;
	deleteAfterImport: boolean;
	missingMods: string[];
};

export type SyncImportData = {
	manifest: ProfileManifest;
	id: string;
	created_at: string;
	updated_at: string;
	owner: SyncUser;
};

type ProfileManifest = {
	profileName: string;
	mods: ProfileManifestMod[];
	community: string | null;
	ignoredUpdates: string[];
};

type ProfileManifestMod = {
	name: string;
	enabled: string;
	version: {
		major: number;
		minor: number;
		patch: number;
	};
};

export type MissingProfileAction = { type: 'locate'; newPath: string } | { type: 'delete' };
