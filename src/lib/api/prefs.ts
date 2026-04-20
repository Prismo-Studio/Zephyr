import { invoke } from '$lib/invoke';
import type { Prefs, Zoom } from '$lib/types';

export const get = () => invoke<Prefs>('get_prefs');
export const set = (value: Prefs) => invoke('set_prefs', { value });
export const zoomWindow = (value: Zoom) => invoke('zoom_window', { value });
export const setDpiScale = (value: number) => invoke<number>('set_dpi_scale', { value });
export const getSystemFonts = () => invoke<string[]>('get_system_fonts');
export const openDir = (path: string) => invoke('open_dir', { path });
export const uploadCustomBackground = (filePath: string) =>
	invoke<{ url: string; kind: 'image' | 'video' }>('upload_custom_background', { filePath });
export const probeCustomBackground = (filePath: string) =>
	invoke<{
		size: number;
		kind: 'image' | 'video';
		max_image_bytes: number;
		max_video_bytes: number;
	}>('probe_custom_background', { filePath });
