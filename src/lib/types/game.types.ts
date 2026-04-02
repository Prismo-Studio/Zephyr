export type GameInfo = {
	active: Game;
	lastUpdated: string;
	all: Game[];
	favorites: string[];
};

export type Game = {
	name: string;
	slug: string;
	platforms: Platform[];
	favorite: boolean;
	modLoader: ModLoader;
	popular: boolean;
};

export enum ModLoader {
	BepInEx = 'BepInEx',
	MelonLoader = 'MelonLoader',
	Northstar = 'Northstar',
	GDWeave = 'GDWeave',
	ReturnOfModding = 'ReturnOfModding',
	BepisLoader = 'BepisLoader'
}

export type PackageCategory = {
	name: string;
	slug: string;
};

export type FiltersResponse = {
	results: PackageCategory[];
};

export type Platform = 'steam' | 'epicGames' | 'oculus' | 'origin' | 'xboxStore';

export type LaunchMode =
	| { type: 'launcher'; content?: undefined }
	| { type: 'direct'; content: { instances: number; intervalSecs: number } };
