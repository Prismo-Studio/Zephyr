import type { ConfigEntry } from '$lib/types';

const LIST_SEPARATOR_KEYWORD = 'ListSeparator=';

export interface ListSeparator {
	type: 'default' | 'custom';
	char: string;
}

export function getListSeparator({ description }: ConfigEntry): ListSeparator {
	if (description !== null) {
		const separatorIndex = description.indexOf(LIST_SEPARATOR_KEYWORD);

		if (separatorIndex !== -1) {
			return {
				type: 'custom',
				char: description[separatorIndex + LIST_SEPARATOR_KEYWORD.length]
			};
		}
	}

	return { type: 'default', char: ',' };
}
