import type { LaunchMode, Platform } from './game.types';

/** A directory that was unavailable on startup and got reset to its default. */
export type DirFallback = {
	field: 'dataDir' | 'cacheDir';
	configured: string;
	fallback: string;
};

export type Prefs = {
	dataDir: string;
	cacheDir: string;
	/** Read-only, only sent by the backend. Absent when nothing fell back. */
	dirFallbacks?: DirFallback[];
	fetchModsAutomatically: boolean;
	pullBeforeLaunch: boolean;
	zoomFactor: number;
	dpiScale: number;
	language: string;
	gamepadEnabled: boolean;
	gamePrefs: Map<string, GamePrefs>;
	disabledPlugins: string[];
	installedPlugins: string[];
};

export type GamePrefs = {
	dirOverride: string | null;
	customArgs: string[];
	customArgsEnabled: boolean;
	launchMode: LaunchMode;
	platform: Platform | null;
};

export type Zoom = { factor: number } | { delta: number };
