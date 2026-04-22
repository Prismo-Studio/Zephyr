import { getCurrentWindow } from '@tauri-apps/api/window';

export const fullscreenState = $state<{ active: boolean }>({ active: false });

let initialised = false;

export async function initFullscreen() {
	if (initialised) return;
	initialised = true;
	try {
		const win = getCurrentWindow();
		fullscreenState.active = await win.isFullscreen();
		// Keep the state in sync when fullscreen is toggled outside of our
		// toggleFullscreen() wrapper (e.g. the native green button on macOS).
		await win.onResized(async () => {
			try {
				fullscreenState.active = await win.isFullscreen();
			} catch {
				// ignore
			}
		});
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
