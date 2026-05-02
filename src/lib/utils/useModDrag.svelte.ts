/**
 * Custom-sort drag & drop for the mod list.
 *
 * Encapsulates pointer tracking, RAF-throttled move, ghost element lifecycle,
 * and the reorder API call. The owning component still binds the container ref
 * and reads `draggedMod` / `placeholderIndex` for visual state.
 */
import * as api from '$lib/api';
import type { Mod } from '$lib/types';
import { createDragGhost, computeInsertPosition, resetGridPositions } from '$lib/utils/dragDrop';

const DRAG_THRESHOLD = 8;

type Opts = {
	/** Current visible mod ordering (post-sort/filter). */
	getSortedMods: () => Mod[];
	/** `'ascending'` | `'descending'` — used to flip reorder delta sign. */
	getSortOrder: () => string;
	/** `'grid'` | `'list'` — affects insert-position math. */
	getViewMode: () => string;
	/** Whether drag is currently allowed (custom sort + not locked). */
	canDrag: () => boolean;
	/** Pin check — pinned mods can't be dragged. */
	isPinned: (uuid: string) => boolean;
	/** Refresh the mod list after a successful reorder. */
	refresh: () => Promise<void>;
};

export function createModDrag(opts: Opts) {
	const state = $state<{ draggedMod: Mod | null; placeholderIndex: number }>({
		draggedMod: null,
		placeholderIndex: -1
	});

	let dragFromIndex = -1;
	let insertPos = -1;
	let ghostEl: HTMLDivElement | null = null;
	let dragOffsetX = 0;
	let dragOffsetY = 0;
	let dragPendingEvent: PointerEvent | null = null;
	let dragRafId: number | null = null;
	let dragStartX = 0;
	let dragStartY = 0;
	let isDragging = false;
	let dragStartMod: Mod | null = null;

	function handleDragHandleDown(e: PointerEvent, mod: Mod) {
		if (!opts.canDrag() || opts.isPinned(mod.uuid)) return;

		dragFromIndex = opts.getSortedMods().findIndex((m) => m.uuid === mod.uuid);
		dragStartX = e.clientX;
		dragStartY = e.clientY;
		isDragging = false;
		state.draggedMod = null;
		dragStartMod = mod;

		window.addEventListener('pointermove', handleWindowPointermove);
		window.addEventListener('pointerup', handleWindowPointerup);
	}

	function handleWindowPointermove(e: PointerEvent) {
		if (isDragging || !dragStartMod) return;

		const deltaX = e.clientX - dragStartX;
		const deltaY = e.clientY - dragStartY;
		const distance = Math.sqrt(deltaX * deltaX + deltaY * deltaY);

		if (distance > DRAG_THRESHOLD) {
			isDragging = true;
			state.draggedMod = dragStartMod;
			insertPos = -1;
			state.placeholderIndex = -1;

			const card = document.querySelector(`[data-mod-uuid="${dragStartMod.uuid}"]`) as HTMLElement;
			if (!card) return;

			const rect = card.getBoundingClientRect();
			dragOffsetX = e.clientX - rect.left;
			dragOffsetY = e.clientY - rect.top;

			ghostEl = createDragGhost(card, e);

			window.removeEventListener('pointermove', handleWindowPointermove);
			window.addEventListener('pointermove', handlePointerMoveThrottled);
		}
	}

	function handleWindowPointerup() {
		window.removeEventListener('pointermove', handleWindowPointermove);
		window.removeEventListener('pointermove', handlePointerMoveThrottled);
		window.removeEventListener('pointerup', handleWindowPointerup);

		if (dragRafId !== null) {
			cancelAnimationFrame(dragRafId);
			dragRafId = null;
		}

		if (ghostEl) {
			ghostEl.remove();
			ghostEl = null;
		}

		// Only reorder if drag actually occurred
		if (isDragging && state.draggedMod && insertPos >= 0 && insertPos !== dragFromIndex) {
			const isDescending = opts.getSortOrder() === 'descending';
			const delta = isDescending ? -(insertPos - dragFromIndex) : insertPos - dragFromIndex;
			if (delta !== 0) {
				const captured = state.draggedMod;
				(async () => {
					try {
						await api.profile.reorderMod(captured.uuid, delta);
						await opts.refresh();
					} catch (err) {
						console.error('Failed to reorder mod:', err);
					}
				})();
			}
		}

		state.draggedMod = null;
		dragStartMod = null;
		insertPos = -1;
		state.placeholderIndex = -1;
		dragFromIndex = -1;
		isDragging = false;
		resetGridPositions();
	}

	function handlePointerMoveThrottled(e: PointerEvent) {
		dragPendingEvent = e;
		if (dragRafId === null) {
			dragRafId = requestAnimationFrame(() => {
				if (dragPendingEvent) {
					handlePointerMove(dragPendingEvent);
				}
				dragRafId = null;
			});
		}
	}

	function handlePointerMove(e: PointerEvent) {
		if (!state.draggedMod || !ghostEl) return;

		ghostEl.style.left = e.clientX - dragOffsetX + 'px';
		ghostEl.style.top = e.clientY - dragOffsetY + 'px';

		const result = computeInsertPosition(
			e,
			dragFromIndex,
			'[data-mod-index]',
			opts.getViewMode() === 'grid'
		);
		insertPos = result.insertPos;
		state.placeholderIndex = result.placeholderIndex;
	}

	return {
		get draggedMod() {
			return state.draggedMod;
		},
		get placeholderIndex() {
			return state.placeholderIndex;
		},
		handleDragHandleDown
	};
}
