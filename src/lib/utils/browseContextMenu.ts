import type { Mod } from '$lib/types';
import type { ContextMenuItem } from '$lib/components/ui/ContextMenu.svelte';
import { open } from '@tauri-apps/plugin-shell';
import * as api from '$lib/api';
import { m } from '$lib/paraglide/messages';

export type BrowseContextMenuHandlers = {
	installLatest: (mod: Mod) => void;
	toggleMod: (mod: Mod) => void;
	removeMod: (mod: Mod) => void;
	doBatchInstall: () => void;
	refresh: () => Promise<void>;
};

export type BrowseContextMenuOpts = {
	mod: Mod;
	locked: boolean;
	mods: Mod[];
	selectedModIds: string[];
	handlers: BrowseContextMenuHandlers;
};

export function buildBrowseContextMenu({
	mod,
	locked,
	mods,
	selectedModIds,
	handlers
}: BrowseContextMenuOpts): ContextMenuItem[] {
	const items: ContextMenuItem[] = [];
	const isMulti = selectedModIds.length > 1 && selectedModIds.includes(mod.uuid);

	if (isMulti) {
		const selectedMods = selectedModIds
			.map((id) => mods.find((mm) => mm.uuid === id))
			.filter(Boolean) as Mod[];
		const notInstalled = selectedMods.filter((mm) => !mm.isInstalled);
		const installed = selectedMods.filter((mm) => mm.isInstalled);

		if (notInstalled.length > 0 && !locked) {
			items.push({
				label: `${m.mods_contextMenu_install()} (${notInstalled.length})`,
				icon: 'mdi:download',
				onclick: handlers.doBatchInstall
			});
		}

		if (installed.length > 0) {
			items.push({
				label: `${m.mods_contextMenu_uninstall()} (${installed.length})`,
				icon: 'mdi:delete',
				danger: true,
				disabled: locked,
				onclick: async () => {
					for (const im of installed) {
						await api.profile.forceRemoveMods([im.uuid]);
					}
					await handlers.refresh();
				}
			});
		}
		return items;
	}

	if (!mod.isInstalled && !locked) {
		items.push({
			label: m.mods_contextMenu_install(),
			icon: 'mdi:download',
			onclick: () => handlers.installLatest(mod)
		});
	}

	if (mod.isInstalled) {
		items.push({
			label: mod.enabled === false ? m.mods_contextMenu_enable() : m.mods_contextMenu_disable(),
			icon: mod.enabled === false ? 'mdi:eye' : 'mdi:eye-off',
			disabled: locked,
			onclick: () => handlers.toggleMod(mod)
		});
	}

	if (mod.websiteUrl) {
		items.push({
			label: m.mods_contextMenu_openThunderstore(),
			icon: 'mdi:open-in-new',
			onclick: () => open(mod.websiteUrl!)
		});
	}

	if (mod.isInstalled) {
		items.push({ label: '', separator: true });
		items.push({
			label: m.mods_contextMenu_uninstall(),
			icon: 'mdi:delete',
			danger: true,
			disabled: locked,
			onclick: () => handlers.removeMod(mod)
		});
	}

	return items;
}
