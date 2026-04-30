import {
	TOAST_ERROR_DURATION_MS,
	TOAST_INFO_DURATION_MS,
	TOAST_MAX_COUNT
} from './constants/ui.constants';

export type Toast = {
	type: 'error' | 'info';
	name?: string;
	message: string;
};

type ToastWithId = Toast & { id: number };

const state = $state<{ list: ToastWithId[] }>({ list: [] });

/** Reactive accessor for the current toast list. */
export function toasts(): readonly ToastWithId[] {
	return state.list;
}

export function pushInfoToast(toast: { name?: undefined; message: string }) {
	pushToast({
		type: 'info',
		...toast
	});
}

let nextId = 0;

export function pushToast(toast: Toast) {
	const id = nextId;
	nextId++;

	state.list.push({ ...toast, id });
	if (state.list.length > TOAST_MAX_COUNT) {
		state.list.shift();
	}

	setTimeout(
		() => {
			const index = state.list.findIndex((t) => t.id == id);
			if (index !== -1) {
				state.list.splice(index, 1);
			}
		},
		toast.type === 'error' ? TOAST_ERROR_DURATION_MS : TOAST_INFO_DURATION_MS
	);
}

export function clearToast(index: number) {
	state.list.splice(index, 1);
}
