import { invoke } from '$lib/invoke';
import type { Plugin } from '$lib/types';

export type InstalledTheme = {
	id: string;
	name: string;
	css: string;
};

export const list = () => invoke<Plugin[]>('get_plugins');

export const setEnabled = (id: string, enabled: boolean) =>
	invoke('set_plugin_enabled', { id, enabled });

export const refresh = () => invoke('refresh_plugins');

export const install = (id: string) => invoke<InstalledTheme>('install_plugin', { id });

export const uninstall = (id: string) => invoke('uninstall_plugin', { id });

export const getInstalledThemes = () => invoke<InstalledTheme[]>('get_installed_themes');

export const registerLocalPlugin = (path: string) =>
	invoke<Plugin>('register_local_plugin', { path });

export const unregisterLocalPlugin = (id: string) => invoke('unregister_local_plugin', { id });

export const reloadLocalPlugin = (id: string) => invoke<Plugin>('reload_local_plugin', { id });

export const getPluginUiUrl = (id: string) => invoke<string>('get_plugin_ui_url', { id });
