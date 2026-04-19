import { getCurrentWindow } from '@tauri-apps/api/window';

export const fullscreenState = $state<{ active: boolean }>({ active: false });

let initialised = false;

export async function initFullscreen() {
	if (initialised) return;
	initialised = true;
	try {
		fullscreenState.active = await getCurrentWindow().isFullscreen();
	} catch (err) {
		console.warn('fullscreen init failed:', err);
	}
}

export async function setFullscreen(on: boolean) {
	try {
		const win = getCurrentWindow();
		if (on && (await win.isMaximized())) {
			await win.unmaximize();
		}
		await win.setFullscreen(on);
		fullscreenState.active = on;
	} catch (err) {
		console.error('setFullscreen failed:', err);
	}
}

export async function toggleFullscreen() {
	await setFullscreen(!fullscreenState.active);
}
