import { invoke } from '$lib/invoke';

export const getCachedIcon = (url: string) => invoke<string>('get_cached_icon', { url });
export const clearIconCache = () => invoke<number>('clear_icon_cache');
