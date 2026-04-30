import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { pushToast } from './toast.svelte';
import { toSentenceCase } from 'js-convert-case';
import { m } from './paraglide/messages';
import { i18nState } from './i18nCore.svelte';

const ERROR_TRANSLATIONS: Record<string, () => string> = {
	'mod is already installed.': () =>
		(i18nState.locale && m.error_modAlreadyInstalled?.()) || 'Mod is already installed.',
	'game directory not found': () =>
		(i18nState.locale && m.error_gameDirNotFound?.()) || 'Game directory not found.'
};

type Error = {
	name: string;
	message: string;
};

let errorListenerInit = false;
export function initErrorListener() {
	if (errorListenerInit) return;
	errorListenerInit = true;
	listen<Error>('error', (evt) =>
		pushToast({
			type: 'error',
			...evt.payload
		})
	).catch(() => {});
}

export async function invoke<T = void>(cmd: string, args?: any): Promise<T> {
	try {
		return await tauriInvoke<T>(cmd, args);
	} catch (error: any) {
		let errStr = error as string;
		let name = `Failed to ${toSentenceCase(cmd).toLowerCase()}`;
		let message = errStr[0].toUpperCase() + errStr.slice(1);

		if (!['.', '?', '!'].includes(message[message.length - 1])) {
			message += '.';
		}

		// Translate known backend error messages
		const lowerMsg = message.toLowerCase();
		for (const [key, translate] of Object.entries(ERROR_TRANSLATIONS)) {
			if (lowerMsg.includes(key)) {
				message = translate();
				break;
			}
		}

		pushError({ name, message });
		throw error;
	}
}

function pushError(error: Error) {
	let msg = `${error.name}: ${error.message}`;
	tauriInvoke('log_err', { msg });

	pushToast({
		type: 'error',
		...error
	});
}
