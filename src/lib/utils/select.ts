export function selectItems(
	items: string[],
	getLabel: (item: string) => string = (value) => value as string
) {
	return items.map((item) => ({ value: item, label: getLabel(item) }));
}
