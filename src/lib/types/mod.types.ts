export type Mod = {
	name: string;
	description: string | null;
	categories: string[] | null;
	version: string | null;
	author: string | null;
	rating: number | null;
	downloads: number | null;
	fileSize: number;
	websiteUrl: string | null;
	donateUrl: string | null;
	dependencies: string[] | null;
	isPinned: boolean;
	isDeprecated: boolean;
	isInstalled: boolean | undefined;
	containsNsfw: boolean;
	uuid: string;
	versionUuid: string;
	lastUpdated: string | null;
	versions: ModVersion[];
	type: ModType;
	enabled?: boolean | null;
	icon: string | null;
	configFile: string | null;
};

export type ModVersion = {
	name: string;
	uuid: string;
};

export enum ModType {
	Local = 'local',
	Remote = 'remote'
}

export type ModId = {
	packageUuid: string;
	versionUuid: string;
};

export type Dependant = {
	fullName: string;
	uuid: string;
};

export type ModActionResponse = { type: 'done' } | { type: 'confirm'; dependants: Dependant[] };

export type InstallTask = 'download' | 'extract' | 'install';

export type InstallEvent =
	| { type: 'show' }
	| { type: 'hide'; reason: 'done' | 'error' | 'cancelled' }
	| { type: 'addCount'; mods: number; bytes: number }
	| { type: 'addProgress'; mods: number; bytes: number }
	| { type: 'setTask'; name: string; task: InstallTask };

export type AvailableUpdate = {
	fullName: string;
	ignore: boolean;
	packageUuid: string;
	versionUuid: string;
	old: string;
	new: string;
};

export type ModpackArgs = {
	name: string;
	description: string;
	author: string;
	categories: string[];
	nsfw: boolean;
	readme: string;
	changelog: string;
	versionNumber: string;
	iconPath: string;
	websiteUrl: string;
	includeDisabled: boolean;
	includeFileMap: Map<string, boolean>;
};

export type MarkdownType = 'readme' | 'changelog';
