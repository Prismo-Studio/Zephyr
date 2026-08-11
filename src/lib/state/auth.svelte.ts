import * as api from '$lib/api';
import type { SyncUser } from '$lib/types';
import { captureEvent } from '$lib/telemetry.svelte';
import { blockAutoSync, resetAutoSyncBlock } from './autoSync.svelte';

class AuthState {
	user: SyncUser | null = $state(null);

	refresh = async () => {
		this.user = await api.profile.sync.getUser();
	};

	login = async () => {
		const user = await api.profile.sync.login();
		this.user = user;
		resetAutoSyncBlock();
		captureEvent('discord_login');
		return user;
	};

	logout = async () => {
		await api.profile.sync.logout();
		this.user = null;
		captureEvent('discord_logout');
	};

	/** The backend already dropped the credentials, so only mirror that here.
	 *  Returns false when the session was already closed, so callers can avoid
	 *  notifying the user twice. */
	sessionExpired = () => {
		blockAutoSync();

		if (this.user === null) return false;

		this.user = null;
		captureEvent('discord_session_expired');
		return true;
	};
}

const auth = new AuthState();

export default auth;
