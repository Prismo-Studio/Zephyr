import * as api from '$lib/api';
import type { SyncUser } from '$lib/types';
import { captureEvent } from '$lib/telemetry.svelte';

class AuthState {
	user: SyncUser | null = $state(null);

	refresh = async () => {
		this.user = await api.profile.sync.getUser();
	};

	login = async () => {
		const user = await api.profile.sync.login();
		this.user = user;
		captureEvent('discord_login');
		return user;
	};

	logout = async () => {
		await api.profile.sync.logout();
		this.user = null;
		captureEvent('discord_logout');
	};
}

const auth = new AuthState();

export default auth;
