import type { Mod } from '$lib/types';
import type { ContextMenuItem } from '$lib/components/ui/ContextMenu.svelte';
import { open } from '@tauri-apps/plugin-shell';
import * as api from '$lib/api';
import { m } from '$lib/paraglide/messages';
import { isModPinned, togglePin } from '$lib/state/misc.svelte';

export type ModContextMenuHandlers = {
	toggleMod: (mod: Mod) => void;
	removeMod: (mod: Mod) => void;
	doBatchToggle: () => void;
	doBatchRemove: () => void;
};

export type ModContextMenuOpts = {
	mod: Mod;
	locked: boolean;
	selectedModIds: string[];
	handlers: ModContextMenuHandlers;
};

export function buildModContextMenu({
	mod,
	locked,
	selectedModIds,
	handlers
}: ModContextMenuOpts): ContextMenuItem[] {
	const items: ContextMenuItem[] = [];
	const isMulti = selectedModIds.length > 1 && selectedModIds.includes(mod.uuid);

	if (isMulti) {
		const count = selectedModIds.length;
		items.push({
			label: `${m.mods_contextMenu_enable()} / ${m.mods_contextMenu_disable()} (${count})`,
			icon: 'mdi:toggle-switch',
			disabled: locked,
			onclick: handlers.doBatchToggle
		});
		items.push({ label: '', separator: true });
		items.push({
			label: `${m.mods_contextMenu_uninstall()} (${count})`,
			icon: 'mdi:delete',
			danger: true,
			disabled: locked,
			onclick: handlers.doBatchRemove
		});
		return items;
	}

	if (mod.isInstalled) {
		items.push({
			label: mod.enabled === false ? m.mods_contextMenu_enable() : m.mods_contextMenu_disable(),
			icon: mod.enabled === false ? 'mdi:eye' : 'mdi:eye-off',
			disabled: locked,
			onclick: () => handlers.toggleMod(mod)
		});

		const pinned = isModPinned(mod.uuid);
		items.push({
			label: pinned ? m.mods_contextMenu_unpin() : m.mods_contextMenu_pin(),
			icon: pinned ? 'mdi:pin-off' : 'mdi:pin',
			onclick: () => togglePin(mod.uuid)
		});
	}

	if (mod.websiteUrl) {
		items.push({
			label: m.mods_contextMenu_openThunderstore(),
			icon: 'mdi:open-in-new',
			onclick: () => open(mod.websiteUrl!)
		});
	}

	if (mod.donateUrl) {
		items.push({
			label: m.mods_contextMenu_donate(),
			icon: 'mdi:heart',
			onclick: () => open(mod.donateUrl!)
		});
	}

	if (mod.isInstalled) {
		items.push({
			label: m.mods_contextMenu_openFolder(),
			icon: 'mdi:folder-open',
			onclick: () => api.profile.openModDir(mod.uuid)
		});
		items.push({
			label: m.mods_contextMenu_editConfig(),
			icon: 'mdi:file-cog',
			onclick: () => {
				window.location.href = `/config?mod=${encodeURIComponent(mod.name)}`;
			}
		});
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
