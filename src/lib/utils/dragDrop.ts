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

let _lastGridResult = -1;

export function resetGridPositions() {
	_lastGridResult = -1;
}

export function computeInsertPosition(
	e: PointerEvent,
	dragFromIndex: number,
	itemSelector: string,
	isGridView: boolean = false
): { insertPos: number; placeholderIndex: number } {
	const wrappers = document.querySelectorAll(itemSelector);

	if (!isGridView) {
		let aboveCount = 0;
		wrappers.forEach((el) => {
			const idx = parseInt(el.getAttribute('data-mod-index')!);
			if (idx === dragFromIndex) return;
			const rect = el.getBoundingClientRect();
			if (rect.top + rect.height / 2 < e.clientY) aboveCount++;
		});

		const insertPos = aboveCount;
		let placeholderIndex: number;
		if (insertPos <= dragFromIndex) placeholderIndex = insertPos;
		else placeholderIndex = insertPos + 1;
		if (insertPos === dragFromIndex) placeholderIndex = -1;
		return { insertPos, placeholderIndex };
	} else {
		let hoverIdx = -1;
		wrappers.forEach((el) => {
			const idx = parseInt(el.getAttribute('data-mod-index')!);
			if (idx === dragFromIndex) return;
			const rect = el.getBoundingClientRect();
			if (
				e.clientX >= rect.left &&
				e.clientX <= rect.right &&
				e.clientY >= rect.top &&
				e.clientY <= rect.bottom
			) {
				hoverIdx = idx;
			}
		});

		if (hoverIdx !== -1 && hoverIdx !== dragFromIndex) {
			_lastGridResult = hoverIdx;
		}

		if (_lastGridResult === -1 || _lastGridResult === dragFromIndex) {
			return { insertPos: dragFromIndex, placeholderIndex: -1 };
		}

		const placeholderIndex =
			_lastGridResult > dragFromIndex ? _lastGridResult + 1 : _lastGridResult;
		return { insertPos: _lastGridResult, placeholderIndex };
	}
}
