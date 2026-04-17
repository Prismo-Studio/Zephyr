import { invoke } from '$lib/invoke';
import type {
	Game,
	GameInfo,
	ModActionResponse,
	ProfileQuery,
	ManagedGameInfo,
	QueryModsArgs,
	ModId,
	MarkdownType
} from '$lib/types';

async function triggerAutoPush() {
	try {
		const mod = await import('$lib/state/autoSync.svelte');
		mod.maybeAutoPush();
	} catch {}
}

export * as export from './export';
export * as import from './import';
export * as install from './install';
export * as launch from './launch';
export * as sync from './sync';
export * as update from './update';

export const getGameInfo = () => invoke<GameInfo>('get_game_info');
export const favoriteGame = (slug: string) => invoke('favorite_game', { slug });
export const setActiveGame = (slug: string) => invoke('set_active_game', { slug });
export const getInfo = () => invoke<ManagedGameInfo>('get_profile_info');
export const getAllSyncIds = () => invoke<string[]>('get_all_sync_ids');
export const setActive = (index: number) => invoke('set_active_profile', { index });
export const reorderProfile = (from: number, to: number) => invoke('reorder_profile', { from, to });
export const query = (args: QueryModsArgs) => invoke<ProfileQuery>('query_profile', { args });
export const isModInstalled = (uuid: string) => invoke<boolean>('is_mod_installed', { uuid });
export const create = (name: string, overridePath: string | null) =>
	invoke('create_profile', { name, overridePath });
export const deleteProfile = (profileId: number) => invoke('delete_profile', { id: profileId });
export const rename = (name: string) => invoke('rename_profile', { name });
export const duplicate = (name: string) => invoke('duplicate_profile', { name });
export const removeMod = async (uuid: string) => {
	const res = await invoke<ModActionResponse>('remove_mod', { uuid });
	if (res?.type === 'done') void triggerAutoPush();
	return res;
};
export const toggleMod = async (uuid: string) => {
	const res = await invoke<ModActionResponse>('toggle_mod', { uuid });
	if (res?.type === 'done') void triggerAutoPush();
	return res;
};
export const forceRemoveMods = async (uuids: string[]) => {
	const res = await invoke('force_remove_mods', { uuids });
	void triggerAutoPush();
	return res;
};
export const forceToggleMods = async (uuids: string[]) => {
	const res = await invoke('force_toggle_mods', { uuids });
	void triggerAutoPush();
	return res;
};
export const reorderMod = async (uuid: string, delta: number) => {
	const res = await invoke('reorder_mod', { uuid, delta });
	void triggerAutoPush();
	return res;
};
export const setAllModsState = async (enable: boolean) => {
	const res = await invoke<number>('set_all_mods_state', { enable });
	void triggerAutoPush();
	return res;
};
export const removeDisabledMods = async () => {
	const res = await invoke<number>('remove_disabled_mods');
	void triggerAutoPush();
	return res;
};
export const getDependants = (uuid: string) => invoke<string[]>('get_dependants', { uuid });
export const openDir = () => invoke('open_profile_dir');
export const openModDir = (uuid: string) => invoke('open_mod_dir', { uuid });
export const openGameLog = () => invoke('open_game_log');
export const createDesktopShortcut = () => invoke('create_desktop_shortcut');
export const getLocalMarkdown = (uuid: string, type: MarkdownType) =>
	invoke<string | null>('get_local_markdown', { uuid, kind: type });
export const setCustomArgs = (customArgs: string[], enabled: boolean) =>
	invoke('set_custom_args', { customArgs, enabled });
export const setProfilePath = (profileId: number, newPath: string) =>
	invoke('set_profile_path', { profileId, newPath });
export const forgetProfile = (profileId: number) => invoke('forget_profile', { profileId });
export const setProfileIcon = (profileId: number, imagePath: string) =>
	invoke<string>('set_profile_icon', { profileId, imagePath });
export const setProfileIconUrl = (profileId: number, url: string) =>
	invoke('set_profile_icon_url', { profileId, url });
export const uploadProfileIcon = (profileId: number, imagePath: string) =>
	invoke<string>('upload_profile_icon', { profileId, imagePath });
export const removeProfileIcon = (profileId: number) =>
	invoke('remove_profile_icon', { profileId });
