import { invoke } from '$lib/invoke';
import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import type { ModId } from '$lib/types';

// Uses raw tauriInvoke to avoid the global error toast. Version changes
// can trigger "already installed" for deps which is expected and noisy.
export const changeModVersion = (id: ModId) => tauriInvoke('change_mod_version', { id });
export const mods = (uuids: string[], respectIgnored: boolean) =>
	invoke('update_mods', { uuids, respectIgnored });
export const ignore = (versionUuid: string) => invoke('ignore_update', { versionUuid });
