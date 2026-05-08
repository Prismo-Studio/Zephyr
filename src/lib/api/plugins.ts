import { invoke } from '$lib/invoke';
import type { Plugin } from '$lib/types';

export const list = () => invoke<Plugin[]>('get_plugins');

export const setEnabled = (id: string, enabled: boolean) =>
	invoke('set_plugin_enabled', { id, enabled });

export const refresh = () => invoke('refresh_plugins');
