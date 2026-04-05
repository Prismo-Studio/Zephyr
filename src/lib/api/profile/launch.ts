import { invoke } from '$lib/invoke';

export const launchGame = () => invoke('launch_game');
export const launchVanilla = () => invoke('launch_vanilla');
export const getArgs = () => invoke<string>('get_launch_args');
export const openGameDir = () => invoke('open_game_dir');
export const getGameDir = () => invoke<string>('get_game_dir');
