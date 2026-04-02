/**
 * Handles Ctrl+click (toggle), Shift+click (range), and plain click (single)
 * selection on a list of items identified by string IDs.
 */
export function handleMultiSelect(
	evt: MouseEvent,
	itemId: string,
	index: number,
	currentSelection: string[],
	allIds: string[],
	lastClickedIndex: number
): { selection: string[]; lastIndex: number } {
	if (evt.ctrlKey || evt.metaKey) {
		const selection = currentSelection.includes(itemId)
			? currentSelection.filter((id) => id !== itemId)
			: [...currentSelection, itemId];
		return { selection, lastIndex: index };
	}

	if (evt.shiftKey && lastClickedIndex !== -1) {
		const start = Math.min(lastClickedIndex, index);
		const end = Math.max(lastClickedIndex, index);
		const newSelection = new Set(currentSelection);
		for (let j = start; j <= end; j++) {
			if (allIds[j]) newSelection.add(allIds[j]);
		}
		return { selection: Array.from(newSelection), lastIndex: lastClickedIndex };
	}

	if (currentSelection.length === 1 && currentSelection[0] === itemId) {
		return { selection: [], lastIndex: index };
	}

	return { selection: [itemId], lastIndex: index };
}
