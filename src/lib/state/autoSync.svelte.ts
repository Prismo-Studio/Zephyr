import * as prefs from '$lib/api/prefs';
import * as sync from '$lib/api/profile/sync';
import * as profileApi from '$lib/api/profile';
import profiles from './profile.svelte';
import auth from './auth.svelte';
import { pushToast } from '$lib/toast.svelte';
import { m } from '$lib/paraglide/messages';

let pushTimer: ReturnType<typeof setTimeout> | null = null;
let pushing = false;
const DEBOUNCE_MS = 1500;

async function shouldAutoPush(): Promise<boolean> {
	try {
		const p = await prefs.get();
		if (!p.pullBeforeLaunch) return false;
	} catch {
		return false;
	}

	if (!auth.user) return false;
	const active = profiles.active;
	if (!active || !active.sync) return false;
	if (active.sync.owner.discordId !== auth.user.discordId) return false;
	return true;
}

async function doPush() {
	if (pushing) return;
	pushing = true;
	try {
		if (!(await shouldAutoPush())) return;
		await sync.push();
		await profiles.refresh();
	} catch (e: any) {
		pushToast({
			type: 'error',
			name: m.sync_autoSyncFailed(),
			message: e?.message ?? String(e)
		});
	} finally {
		pushing = false;
	}
}

export function maybeAutoPush() {
	if (pushTimer) clearTimeout(pushTimer);
	pushTimer = setTimeout(() => {
		pushTimer = null;
		void doPush();
	}, DEBOUNCE_MS);
}

export async function maybeSyncAfterImport(
	options: { forceFork?: boolean } = {}
): Promise<boolean> {
	try {
		const p = await prefs.get();
		if (!p.pullBeforeLaunch) return false;
	} catch {
		return false;
	}

	if (!auth.user) return false;

	let active: { id: number; sync: { id: string; owner: { discordId: string } } | null } | null;
	try {
		const info = await profileApi.getInfo();
		const found = info.profiles.find((p) => p.id === info.activeId);
		active = found
			? {
					id: found.id,
					sync: found.sync
						? { id: found.sync.id, owner: { discordId: found.sync.owner.discordId } }
						: null
				}
			: null;
		await profiles.refresh();
	} catch {
		active = null;
	}
	if (!active) return false;

	const ownsSync = !!active.sync && active.sync.owner.discordId === auth.user.discordId;

	try {
		if (!options.forceFork && ownsSync) {
			await sync.push();
			await profiles.refresh();
			return true;
		}
		if (active.sync) {
			await sync.disconnect(false);
		}
		const newId = await sync.create();
		await profiles.refresh();
		pushToast({
			type: 'info',
			name: m.sync_synced(),
			message: m.sync_copiedToCloud({ id: newId.slice(0, 8) + '…' })
		});
		return true;
	} catch (e: any) {
		pushToast({
			type: 'error',
			name: m.sync_autoSyncFailed(),
			message: e?.message ?? String(e)
		});
		return false;
	}
}
