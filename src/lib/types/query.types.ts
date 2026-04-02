export type SortBy =
	| 'newest'
	| 'name'
	| 'author'
	| 'lastUpdated'
	| 'downloads'
	| 'rating'
	| 'installDate'
	| 'custom'
	| 'diskSpace';

export type SortOrder = 'ascending' | 'descending';

export type QueryModsArgs = {
	searchTerm: string;
	includeCategories: string[];
	excludeCategories: string[];
	includeNsfw: boolean;
	includeDeprecated: boolean;
	includeDisabled: boolean;
	includeEnabled: boolean;
	sortBy: SortBy;
	sortOrder: SortOrder;
	maxCount: number;
};

export type QueryModsArgsWithoutMax = Omit<QueryModsArgs, 'maxCount'>;
