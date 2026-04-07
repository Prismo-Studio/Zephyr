/**
 * Manages pointer-based drag-and-drop reordering in a vertical list.
 * Tracks ghost element, insertion position, and placeholder index.
 */
export type DragState<T> = {
	item: T | null;
	fromIndex: number;
	insertPos: number;
	placeholderIndex: number;
};

export function createDragState<T>(): DragState<T> {
	return {
		item: null,
		fromIndex: -1,
		insertPos: -1,
		placeholderIndex: -1
	};
}

export function resetDragState<T>(state: DragState<T>) {
	state.item = null;
	state.fromIndex = -1;
	state.insertPos = -1;
	state.placeholderIndex = -1;
}

export function createDragGhost(card: HTMLElement, e: PointerEvent): HTMLDivElement {
	const rect = card.getBoundingClientRect();
	const ghost = card.cloneNode(true) as HTMLDivElement;
	ghost.style.cssText = `
		position: fixed;
		left: ${rect.left}px;
		top: ${rect.top}px;
		width: ${rect.width}px;
		pointer-events: none;
		z-index: 9999;
		opacity: 0.9;
		border: 1px solid var(--accent-400);
		box-shadow: 0 8px 32px rgba(26, 255, 250, 0.2), 0 0 0 1px rgba(26, 255, 250, 0.3);
		background: var(--bg-elevated);
		border-radius: var(--radius-lg);
		transform: scale(1.02);
		cursor: grabbing;
	`;
	document.body.appendChild(ghost);
	return ghost;
}

/**
 * Computes the insertion position.
 * For list view: counts items with vertical center above the cursor.
 * For grid view: finds the item closest to the cursor position based on center point.
 */
export function computeInsertPosition(
	e: PointerEvent,
	dragFromIndex: number,
	itemSelector: string,
	isGridView: boolean = false
): { insertPos: number; placeholderIndex: number } {
	const wrappers = document.querySelectorAll(itemSelector);
	let aboveCount = 0;

	if (!isGridView) {
		// List view: vertical positioning
		wrappers.forEach((el) => {
			const idx = parseInt(el.getAttribute('data-mod-index')!);
			if (idx === dragFromIndex) return;

			const rect = el.getBoundingClientRect();
			const midY = rect.top + rect.height / 2;
			if (midY < e.clientY) aboveCount++;
		});

		const insertPos = aboveCount;
		let placeholderIndex: number;

		if (insertPos <= dragFromIndex) {
			placeholderIndex = insertPos;
		} else {
			placeholderIndex = insertPos + 1;
		}

		if (insertPos === dragFromIndex) {
			placeholderIndex = -1;
		}

		return { insertPos, placeholderIndex };
	} else {
		// Grid view: find closest item based on center point
		let closestIndex = -1;
		let closestDistance = Infinity;

		wrappers.forEach((el) => {
			const idx = parseInt(el.getAttribute('data-mod-index')!);
			if (idx === dragFromIndex) return;

			const rect = el.getBoundingClientRect();
			const midX = rect.left + rect.width / 2;
			const midY = rect.top + rect.height / 2;

			// Calculate distance from cursor to center of item
			const distX = e.clientX - midX;
			const distY = e.clientY - midY;
			const distance = Math.sqrt(distX * distX + distY * distY);

			if (distance < closestDistance) {
				closestDistance = distance;
				closestIndex = idx;
			}
		});

		let insertPos = closestIndex;
		if (insertPos < 0) insertPos = 0;

		let placeholderIndex: number;
		if (insertPos <= dragFromIndex) {
			placeholderIndex = insertPos;
		} else {
			placeholderIndex = insertPos + 1;
		}

		if (insertPos === dragFromIndex) {
			placeholderIndex = -1;
		}

		return { insertPos, placeholderIndex };
	}
}
