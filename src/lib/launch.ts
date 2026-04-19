import * as api from '$lib/api';
import { installState } from '$lib/state/misc.svelte';

function waitForInstallEnd(): Promise<void> {
	return new Promise((resolve) => {
		const check = () => {
			if (!installState.active) return resolve();
			setTimeout(check, 300);
		};
		setTimeout(check, 500);
	});
}

async function installBepInEx(): Promise<boolean> {
	try {
		const results = await api.thunderstore.query({
			searchTerm: 'BepInExPack',
			includeCategories: [],
			excludeCategories: [],
			includeNsfw: false,
			includeDeprecated: false,
			includeDisabled: true,
			includeEnabled: true,
			sortBy: 'downloads',
			sortOrder: 'descending',
			maxCount: 5
		});
		const bepinex = results.find(
			(m) => m.name === 'BepInExPack' || m.name.startsWith('BepInExPack')
		);
		if (bepinex && !bepinex.isInstalled) {
			await api.profile.install.mod({
				packageUuid: bepinex.uuid,
				versionUuid: bepinex.versions[0].uuid
			});
			await waitForInstallEnd();
			return true;
		}
	} catch {
		// ignore
	}
	return false;
}

export async function launchGameWithBepInExFallback(): Promise<void> {
	try {
		await api.profile.launch.launchGameSilent();
	} catch (err: unknown) {
		const msg = String(err).toLowerCase();
		if (msg.includes('bepinex') || msg.includes('preloader not found')) {
			const installed = await installBepInEx();
			if (installed) {
				try {
					await api.profile.launch.launchGame();
					return;
				} catch {
					// ignore
				}
			}
			return;
		}
		try {
			await api.profile.launch.launchGame();
		} catch {
			// ignore
		}
	}
}
