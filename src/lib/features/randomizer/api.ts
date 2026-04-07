import { invoke } from '$lib/invoke';
import type {
	GameSchema,
	GameSummary,
	GenerateOutcome,
	LintIssue,
	PlayerFile,
	PythonStatus,
	RandomizerConfig,
	SeedFile,
	ServerStatus,
	ValidationError
} from './types';

export const listSupportedGames = () => invoke<GameSummary[]>('list_supported_games');

export const getGameSchema = (gameId: string) =>
	invoke<GameSchema>('get_game_schema', { gameId });

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

export const generateSeed = () => invoke<GenerateOutcome>('generate_seed');

export const startServer = (multidata: string, port: number, password: string | null) =>
	invoke<ServerStatus>('start_server', { multidata, port, password });

export const stopServer = () => invoke('stop_server');

export const serverStatus = () => invoke<ServerStatus>('server_status');

export const openWorkspaceDir = () => invoke('open_workspace_dir');

export const listSeeds = () => invoke<SeedFile[]>('list_seeds');

export const deleteSeed = (path: string) => invoke('delete_seed', { path });

export const clearSeeds = () => invoke<number>('clear_seeds');
