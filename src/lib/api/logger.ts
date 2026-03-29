import { invoke } from '$lib/invoke';

export const openZephyrLog = () => invoke('open_zephyr_log');
export const logErr = (msg: string) => invoke('log_err', { msg });
