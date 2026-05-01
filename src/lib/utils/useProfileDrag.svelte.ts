/**
 * Pointer-driven drag-and-drop reorder for the profile cards grid.
 *
 * Mirrors the shape of `useModDrag` but operates on profile IDs and uses
 * nearest-center hit-testing rather than insert-line math, since cards
 * are arranged in a 2D grid.
 */
import * as api from '$lib/api';

const DRAG_THRESHOLD = 5;

type Opts = {
	refresh: () => Promise<void>;
};

export function createProfileDrag(opts: Opts) {
	const state = $state<{ draggingId: number | null; dragOverIdx: number }>({
		draggingId: null,
		dragOverIdx: -1
	});

	let ghostEl: HTMLDivElement | null = null;
	let dragOffsetX = 0;
	let dragOffsetY = 0;
	let dragFromIdx = -1;
	let dragStarted = false;
	let pointerStartX = 0;
	let pointerStartY = 0;
	let pendingCard: HTMLElement | null = null;
	let pendingProfileId: number | null = null;

	function onPointerDown(e: PointerEvent, profileId: number, idx: number) {
		const target = e.target as HTMLElement;
		if (target.closest('.z-profile-action, .z-profile-icon, input')) return;

		const card = target.closest('.z-profile-card') as HTMLElement;
		if (!card) return;

		dragFromIdx = idx;
		pendingProfileId = profileId;
		pendingCard = card;
		pointerStartX = e.clientX;
		pointerStartY = e.clientY;
		dragStarted = false;

		const rect = card.getBoundingClientRect();
		dragOffsetX = e.clientX - rect.left;
		dragOffsetY = e.clientY - rect.top;

		window.addEventListener('pointermove', onPointerMove);
		window.addEventListener('pointerup', onPointerUp);
	}

	function startDrag(e: PointerEvent) {
		if (!pendingCard) return;
		dragStarted = true;
		state.draggingId = pendingProfileId;
		const rect = pendingCard.getBoundingClientRect();
		ghostEl = pendingCard.cloneNode(true) as HTMLDivElement;
		ghostEl.style.cssText = `
			position: fixed;
			left: ${rect.left}px;
			top: ${rect.top}px;
			width: ${rect.width}px;
			height: ${rect.height}px;
			pointer-events: none;
			z-index: 9999;
			opacity: 0.95;
			border: 2px solid var(--accent-400);
			box-shadow: var(--shadow-glow-strong);
			transform: scale(1.03);
			cursor: grabbing;
		`;
		document.body.appendChild(ghostEl);
		document.body.classList.add('dragging-active');
		onPointerMove(e);
	}

	function onPointerMove(e: PointerEvent) {
		if (!dragStarted) {
			const dx = e.clientX - pointerStartX;
			const dy = e.clientY - pointerStartY;
			if (dx * dx + dy * dy < DRAG_THRESHOLD * DRAG_THRESHOLD) return;
			startDrag(e);
			return;
		}
		if (!ghostEl) return;
		ghostEl.style.left = e.clientX - dragOffsetX + 'px';
		ghostEl.style.top = e.clientY - dragOffsetY + 'px';

		const cards = document.querySelectorAll<HTMLElement>('.z-profile-card[data-profile-idx]');
		let bestIdx = -1;
		let bestDist = Infinity;
		cards.forEach((el) => {
			const idx = parseInt(el.dataset.profileIdx!);
			if (idx === dragFromIdx) return;
			const r = el.getBoundingClientRect();
			const cx = r.left + r.width / 2;
			const cy = r.top + r.height / 2;
			const d = (e.clientX - cx) ** 2 + (e.clientY - cy) ** 2;
			if (d < bestDist) {
				bestDist = d;
				bestIdx = idx;
			}
		});
		state.dragOverIdx = bestIdx;
	}

	async function onPointerUp() {
		window.removeEventListener('pointermove', onPointerMove);
		window.removeEventListener('pointerup', onPointerUp);

		if (!dragStarted) {
			pendingCard = null;
			pendingProfileId = null;
			dragFromIdx = -1;
			return;
		}

		document.body.classList.remove('dragging-active');

		if (ghostEl) {
			ghostEl.remove();
			ghostEl = null;
		}

		const from = dragFromIdx;
		const to = state.dragOverIdx;
		state.draggingId = null;
		state.dragOverIdx = -1;
		dragFromIdx = -1;
		dragStarted = false;
		pendingCard = null;
		pendingProfileId = null;

		if (from === -1 || to === -1 || from === to) return;
		try {
			await api.profile.reorderProfile(from, to);
			await opts.refresh();
		} catch (err) {
			console.error('Failed to reorder profile:', err);
		}
	}

	return {
		get draggingId() {
			return state.draggingId;
		},
		get dragOverIdx() {
			return state.dragOverIdx;
		},
		onPointerDown
	};
}
