import { check, type Update } from '@tauri-apps/plugin-updater';

const CHECK_INTERVAL_MS = 60 * 60 * 1000; // 1 hour

class UpdateState {
	next: Update | null = $state(null);
	isChecking = $state(false);
	#lastChecked: number = 0;

	refresh = async (force = false) => {
		if (this.isChecking) return;

		const now = Date.now();
		if (!force && this.#lastChecked > 0 && now - this.#lastChecked < CHECK_INTERVAL_MS) {
			return;
		}

		this.isChecking = true;
		try {
			this.next = await check();
			this.#lastChecked = Date.now();
		} finally {
			this.isChecking = false;
		}
	};
}

const updates = new UpdateState();
export default updates;
