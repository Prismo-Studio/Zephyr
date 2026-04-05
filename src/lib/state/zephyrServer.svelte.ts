import { PersistedState } from 'runed';
import { configure } from '$lib/api/zephyrServer';

interface ZephyrServerConfig {
	baseUrl: string;
	apiKey: string;
	enabled: boolean;
}

export const zephyrServerState = new PersistedState<ZephyrServerConfig>('zephyrServer', {
	baseUrl: 'http://localhost:3000',
	apiKey: '',
	enabled: false
});

// Keep the API client in sync with persisted config
$effect.root(() => {
	$effect(() => {
		configure(zephyrServerState.current.baseUrl, zephyrServerState.current.apiKey);
	});
});
