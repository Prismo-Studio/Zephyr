import * as api from '$lib/api';
import pluginThemes from '$lib/design-system/pluginThemes.svelte';
import { getTheme, setTheme } from '$lib/design-system/tokens';
import type { Plugin } from '$lib/types';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

class PluginsState {
	list: Plugin[] = $state([]);
	ready = $state(false);
	#unlisten: UnlistenFn | null = null;

	isEnabled = (id: string): boolean => {
		const plugin = this.list.find((p) => p.id === id);
		return plugin?.enabled ?? true;
	};

	refresh = async () => {
		try {
			this.list = await api.plugins.list();
		} finally {
			this.ready = true;
		}
	};

	setEnabled = async (id: string, enabled: boolean) => {
		await api.plugins.setEnabled(id, enabled);
		await this.refresh();
	};

	install = async (id: string) => {
		const theme = await api.plugins.install(id);
		pluginThemes.register({ id: theme.id, name: theme.name }, theme.css);
		await this.refresh();
	};

	uninstall = async (id: string) => {
		await api.plugins.uninstall(id);
		pluginThemes.unregister(id);
		// If the user is sitting on the theme they just removed, drop them
		// back to the default — otherwise [data-theme] points at a selector
		// nothing matches.
		if (getTheme() === id) {
			setTheme('dark');
		}
		await this.refresh();
	};

	// Re-fetch the registry from GitHub. The Rust side emits `plugins_changed`
	// on success, which our existing listener picks up to refresh the list.
	refetch = async () => {
		await api.plugins.refresh();
	};

	registerLocal = async (path: string) => {
		await api.plugins.registerLocalPlugin(path);
		await this.loadInstalledThemes();
		await this.refresh();
	};

	unregisterLocal = async (id: string) => {
		await api.plugins.unregisterLocalPlugin(id);
		pluginThemes.unregister(id);
		if (getTheme() === id) setTheme('dark');
		await this.refresh();
	};

	reloadLocal = async (id: string) => {
		await api.plugins.reloadLocalPlugin(id);
		await this.loadInstalledThemes();
		await this.refresh();
	};

	loadInstalledThemes = async () => {
		try {
			const themes = await api.plugins.getInstalledThemes();
			for (const theme of themes) {
				pluginThemes.register({ id: theme.id, name: theme.name }, theme.css);
			}
		} catch {
			// Best-effort: a missing/corrupted theme just means the user falls
			// back to the default palette until they reinstall.
		}
	};

	#unlistenDevChanged: UnlistenFn | null = null;

	init = async () => {
		await Promise.all([this.refresh(), this.loadInstalledThemes()]);
		if (!this.#unlisten) {
			this.#unlisten = await listen<Plugin[]>('plugins_changed', (evt) => {
				this.list = evt.payload;
			});
		}
		if (!this.#unlistenDevChanged) {
			this.#unlistenDevChanged = await listen<{ id: string }>('dev_plugin_changed', (evt) => {
				// Filesystem watcher detected a change in the dev plugin folder.
				// Re-read the manifest and re-fetch its theme css so the UI updates
				// without the author clicking Reload manually.
				this.reloadLocal(evt.payload.id).catch(() => {});
			});
		}
	};

	dispose = () => {
		this.#unlisten?.();
		this.#unlisten = null;
		this.#unlistenDevChanged?.();
		this.#unlistenDevChanged = null;
	};
}

const plugins = new PluginsState();

export default plugins;
