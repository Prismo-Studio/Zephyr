import * as api from '$lib/api';
import type { ProfileInfo, ManagedGameInfo } from '$lib/types';
import auth from './auth.svelte';

class ProfilesState {
	list: ProfileInfo[] = $state([]);
	activeId: number | null = $state(null);
	ready: boolean = $state(false);

	active: ProfileInfo | null = $derived(
		this.list.find((profile) => profile.id === this.activeId) ?? null
	);

	activeLocked = $derived.by(() => {
		if (this.active === null) return false;
		if (this.active.sync === null) return false;
		if (auth.user === null) return true;

		return this.active.sync.owner.discordId != auth.user.discordId;
	});

	update = async (info: ManagedGameInfo) => {
		this.list = info.profiles;
		this.activeId = info.activeId;
		this.ready = true;
	};

	updateOne = async (info: ProfileInfo) => {
		const index = this.list.findIndex((profile) => profile.id == info.id);
		if (index === -1) {
			this.list.push(info);
		} else {
			this.list[index] = info;
		}
	};

	refresh = async () => {
		try {
			const info = await api.profile.getInfo();
			profiles.update(info);
		} finally {
			this.ready = true;
		}
	};

	setActive = async (index: number) => {
		await api.profile.setActive(index);
	};

	cycle = async (delta: 1 | -1) => {
		if (this.list.length < 2) return;
		const currentIdx = this.list.findIndex((p) => p.id === this.activeId);
		if (currentIdx === -1) {
			await api.profile.setActive(0);
			return;
		}
		const next = (currentIdx + delta + this.list.length) % this.list.length;
		await api.profile.setActive(next);
	};
}

const profiles = new ProfilesState();

export default profiles;
