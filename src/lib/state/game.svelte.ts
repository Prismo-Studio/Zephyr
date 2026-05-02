import type { FiltersResponse, Game, GameInfo, PackageCategory } from '$lib/types';
import * as api from '$lib/api';
import { pushToast } from '$lib/toast.svelte';
import { fetch } from '@tauri-apps/plugin-http';
import { THUNDERSTORE_CATEGORY_URL } from '$lib/constants/api.constants';

class GamesState {
	active: Game | null = $state(null);
	lastUpdated: string = $state('');
	list: Game[] = $state([]);
	categories: PackageCategory[] = $state([]);

	refresh = async () => {
		const info = await api.profile.getGameInfo();

		for (let game of info.all) {
			game.favorite = info.favorites.includes(game.slug);
		}

		this.active = info.active;
		this.lastUpdated = info.lastUpdated;
		this.list = info.all;

		this.#refreshCategories();
	};

	#refreshCategories = async () => {
		const slug = this.active?.slug;
		if (!slug) return;

		try {
			const url = THUNDERSTORE_CATEGORY_URL(slug);
			const response = await fetch(url);

			if (!response.ok) {
				throw new Error(await response.text());
			}

			const data = (await response.json()) as FiltersResponse;
			this.categories = data.results.sort((a, b) => a.name.localeCompare(b.name));
		} catch (err) {
			pushToast({
				type: 'error',
				name: 'Failed to fetch categories',
				message: JSON.stringify(err)
			});
		}
	};

	setActive = async (slug: string) => {
		await api.profile.setActiveGame(slug);
		await this.refresh();
	};
}

const games = new GamesState();

export default games;
