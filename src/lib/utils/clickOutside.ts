import type { Action } from 'svelte/action';

const LEAVE_DELAY_MS = 200;

export const clickOutside: Action<HTMLElement, () => void> = (node, handler) => {
	let currentHandler = handler;
	let leaveTimer: number | null = null;

	const onClick = (e: MouseEvent) => {
		if (!node.contains(e.target as Node)) currentHandler();
	};

	const onLeave = () => {
		if (leaveTimer != null) clearTimeout(leaveTimer);
		leaveTimer = window.setTimeout(() => {
			currentHandler();
			leaveTimer = null;
		}, LEAVE_DELAY_MS);
	};

	const onEnter = () => {
		if (leaveTimer != null) {
			clearTimeout(leaveTimer);
			leaveTimer = null;
		}
	};

	document.addEventListener('click', onClick);
	node.addEventListener('mouseleave', onLeave);
	node.addEventListener('mouseenter', onEnter);

	return {
		update(newHandler: () => void) {
			currentHandler = newHandler;
		},
		destroy() {
			document.removeEventListener('click', onClick);
			node.removeEventListener('mouseleave', onLeave);
			node.removeEventListener('mouseenter', onEnter);
			if (leaveTimer != null) clearTimeout(leaveTimer);
		}
	};
};
