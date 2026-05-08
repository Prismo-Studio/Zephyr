import * as api from '$lib/api';
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

	// Re-fetch the registry from GitHub. The Rust side emits `plugins_changed`
	// on success, which our existing listener picks up to refresh the list.
	refetch = async () => {
		await api.plugins.refresh();
	};

	init = async () => {
		await this.refresh();
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
