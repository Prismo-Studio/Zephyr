import { PersistedState } from 'runed';
import { configure } from '$lib/api/zephyrServer';
import { ZEPHYR_SERVER_DEFAULT_URL } from '$lib/constants/api.constants';

interface ZephyrServerConfig {
	baseUrl: string;
	apiKey: string;
	enabled: boolean;
}

export const zephyrServerState = new PersistedState<ZephyrServerConfig>('zephyrServer', {
	baseUrl: ZEPHYR_SERVER_DEFAULT_URL,
	apiKey: '',
	enabled: false
});

// Keep the API client in sync with persisted config
$effect.root(() => {
	$effect(() => {
		configure(zephyrServerState.current.baseUrl, zephyrServerState.current.apiKey);
	});
});
