import { invoke } from '$lib/invoke';
import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import type {
	ApworldRefreshResult,
	CustomApworld,
	GameSchema,
	GameSummary,
	GenerateOutcome,
	LintIssue,
	PatchFile,
	PlayerFile,
	PythonStatus,
	RandomizerConfig,
	RuntimeStatus,
	SeedFile,
	ServerStatus,
	ValidationError
} from './types';

export const listSupportedGames = () => invoke<GameSummary[]>('list_supported_games');

export const getGameSchema = (gameId: string) => invoke<GameSchema>('get_game_schema', { gameId });

export const generateYaml = (config: RandomizerConfig) =>
	invoke<string>('generate_yaml', { config });

export const validateConfig = (config: RandomizerConfig) =>
	invoke<ValidationError[]>('validate_config', { config });

export const lintYaml = (gameId: string, yaml: string) =>
	invoke<LintIssue[]>('lint_yaml', { gameId, yaml });

// --- Archipelago process integration ---

export const checkPython = () => invoke<PythonStatus>('check_python');

export const savePlayerYaml = (slotName: string, yaml: string) =>
	invoke<string>('save_player_yaml', { slotName, yaml });

export const listPlayerYamls = () => invoke<PlayerFile[]>('list_player_yamls');

export const deletePlayerYaml = (path: string) => invoke('delete_player_yaml', { path });

export const renamePlayerYaml = (path: string, newName: string) =>
	invoke<string>('rename_player_yaml', { path, newName });

export const generateSeed = () => invoke<GenerateOutcome>('generate_seed');

export const startServer = (multidata: string, port: number, password: string | null) =>
	invoke<ServerStatus>('start_server', { multidata, port, password });

export const stopServer = () => invoke('stop_server');

export const serverStatus = () => invoke<ServerStatus>('server_status');

export const openWorkspaceDir = () => invoke('open_workspace_dir');

export const listSeeds = () => invoke<SeedFile[]>('list_seeds');

export const deleteSeed = (path: string) => invoke('delete_seed', { path });

export const renameSeed = (path: string, newName: string) =>
	invoke<string>('rename_seed', { path, newName });

export const clearSeeds = () => invoke<number>('clear_seeds');

// --- Custom apworlds ---

export const listCustomApworlds = () => invoke<CustomApworld[]>('list_custom_apworlds');

export const installApworldFromPath = (srcPath: string) =>
	invoke<CustomApworld>('install_apworld_from_path', { srcPath });

export const installApworldFromBytes = (fileName: string, bytesBase64: string) =>
	invoke<CustomApworld>('install_apworld_from_bytes', { fileName, bytesBase64 });

export const removeCustomApworld = (fileName: string) =>
	invoke('remove_custom_apworld', { fileName });

export const refreshApworldSchemas = () =>
	invoke<ApworldRefreshResult>('refresh_apworld_schemas');

export const openCustomWorldsDir = () => invoke('open_custom_worlds_dir');

// --- Patch files & custom clients ---

export const listPatches = () => invoke<PatchFile[]>('list_patches');

export const deletePatch = (path: string) => invoke('delete_patch', { path });

export const applyPatch = (path: string) => invoke('apply_patch', { path });

export const launchApComponent = (name: string) => invoke('launch_ap_component', { name });

export const getRomPaths = () => invoke<Record<string, string>>('get_rom_paths');

export const setRomPath = (extension: string, romPath: string) =>
	invoke('set_rom_path', { extension, romPath });

export const clearRomPath = (extension: string) => invoke('clear_rom_path', { extension });

// --- Archipelago runtime install ---

export const runtimeStatus = () => invoke<RuntimeStatus>('runtime_status');

export const installRuntime = (url?: string) =>
	invoke<RuntimeStatus>('install_runtime', { url });

export const provisionRuntimeVenv = () => invoke<RuntimeStatus>('provision_runtime_venv');

export const removeRuntime = () => invoke('remove_runtime');

const REMOTE_URL = 'https://randomizer-server-production.up.railway.app';

export type RemoteStatus = {
	running: boolean;
	seed: string | null;
	port: number;
	recent_log: string[];
};

export async function remoteStatus(): Promise<RemoteStatus> {
	// Use tauriInvoke directly to avoid toast spam on polling errors
	const json = await tauriInvoke<string>('remote_request', {
		remoteUrl: REMOTE_URL,
		endpoint: '/status',
		method: 'GET'
	});
	return JSON.parse(json);
}

export async function remoteUploadSeed(filePath: string): Promise<{ uploaded: string }> {
	const json = await invoke<string>('remote_upload_seed', {
		path: filePath,
		remoteUrl: REMOTE_URL
	});
	return JSON.parse(json);
}

export async function remoteStart(seed?: string): Promise<RemoteStatus> {
	const json = await invoke<string>('remote_request', {
		remoteUrl: REMOTE_URL,
		endpoint: '/start',
		method: 'POST',
		body: JSON.stringify(seed ? { seed } : {})
	});
	return JSON.parse(json);
}

export async function remoteStop(): Promise<RemoteStatus> {
	const json = await invoke<string>('remote_request', {
		remoteUrl: REMOTE_URL,
		endpoint: '/stop',
		method: 'POST'
	});
	return JSON.parse(json);
}
