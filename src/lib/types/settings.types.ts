import type { LaunchMode, Platform } from './game.types';

export type Prefs = {
	dataDir: string;
	cacheDir: string;
	fetchModsAutomatically: boolean;
	pullBeforeLaunch: boolean;
	zoomFactor: number;
	dpiScale: number;
	language: string;
	gamepadEnabled: boolean;
	gamePrefs: Map<string, GamePrefs>;
};

export type GamePrefs = {
	dirOverride: string | null;
	customArgs: string[];
	customArgsEnabled: boolean;
	launchMode: LaunchMode;
	platform: Platform | null;
};

export type Zoom = { factor: number } | { delta: number };
