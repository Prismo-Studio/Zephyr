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

	init = async () => {
		await Promise.all([this.refresh(), this.loadInstalledThemes()]);
		if (!this.#unlisten) {
			this.#unlisten = await listen<Plugin[]>('plugins_changed', (evt) => {
				this.list = evt.payload;
			});
		}
	};

	dispose = () => {
		this.#unlisten?.();
		this.#unlisten = null;
	};
}

const plugins = new PluginsState();

export default plugins;
