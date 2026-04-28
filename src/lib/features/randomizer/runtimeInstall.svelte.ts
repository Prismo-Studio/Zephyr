import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { installRuntime, provisionRuntimeVenv, runtimeStatus } from './api';
import type { RuntimeProgress, RuntimeStatus } from './types';

type Mode = 'install' | 'venv' | null;

class RuntimeInstallStore {
	status: RuntimeStatus | null = $state(null);
	loading = $state(true);
	installing = $state(false);
	progress: RuntimeProgress | null = $state(null);
	mode: Mode = $state(null);
	lastError: string | null = $state(null);

	private unlisten: UnlistenFn | null = null;
	private listenerStarted = false;

	async startListener(): Promise<void> {
		if (this.listenerStarted) return;
		this.listenerStarted = true;
		try {
			this.unlisten = await listen<RuntimeProgress>(
				'randomizer://runtime-progress',
				(event) => {
					this.progress = event.payload;
				}
			);
		} catch {
			this.listenerStarted = false;
		}
	}

	disposeListener(): void {
		this.unlisten?.();
		this.unlisten = null;
		this.listenerStarted = false;
	}

	async refresh(): Promise<void> {
		this.loading = true;
		try {
			this.status = await runtimeStatus();
		} finally {
			this.loading = false;
		}
	}

	async install(): Promise<void> {
		if (this.installing) return;
		this.installing = true;
		this.mode = 'install';
		this.progress = null;
		this.lastError = null;
		try {
			this.status = await installRuntime();
		} catch (err) {
			this.lastError = (err as { message?: string })?.message ?? String(err);
			throw err;
		} finally {
			this.installing = false;
			this.mode = null;
		}
	}

	async provisionDeps(): Promise<void> {
		if (this.installing) return;
		this.installing = true;
		this.mode = 'venv';
		this.progress = null;
		this.lastError = null;
		try {
			this.status = await provisionRuntimeVenv();
		} catch (err) {
			this.lastError = (err as { message?: string })?.message ?? String(err);
			throw err;
		} finally {
			this.installing = false;
			this.mode = null;
		}
	}
}

export const runtimeInstallStore = new RuntimeInstallStore();
