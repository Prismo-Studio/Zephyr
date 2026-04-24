<script lang="ts">
	import type { QueryModsArgsWithoutMax, SortBy, SortOrder } from '$lib/types';
	import Icon from '@iconify/svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import games from '$lib/state/game.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		queryArgs: QueryModsArgsWithoutMax;
		sortOptions?: SortBy[];
		showCategories?: boolean;
		expanded?: boolean;
		externalPanel?: boolean;
		viewMode?: 'list' | 'grid';
		pageSize?: number;
		pageSizeChoices?: number[];
		onChangePageSize?: (size: number) => void;
	};

	let {
		queryArgs,
		sortOptions = ['rating', 'downloads', 'lastUpdated', 'newest', 'name'],
		showCategories = false,
		expanded = $bindable(false),
		externalPanel = false,
		viewMode = $bindable('list' as 'list' | 'grid'),
		pageSize,
		pageSizeChoices,
		onChangePageSize
	}: Props = $props();

	let sortOpen = $state(false);
	let pageSizeOpen = $state(false);

	let sortLabels = $derived({
		newest: i18nState.locale && m.modListFilters_options_newest(),
		name: i18nState.locale && m.modListFilters_options_name(),
		author: i18nState.locale && m.modListFilters_options_author(),
		lastUpdated: i18nState.locale && m.modListFilters_options_lastUpdated(),
		downloads: i18nState.locale && m.modListFilters_options_downloads(),
		rating: i18nState.locale && m.modListFilters_options_rating(),
		installDate: i18nState.locale && m.modListFilters_options_installDate(),
		custom: i18nState.locale && m.modListFilters_options_custom(),
		diskSpace: i18nState.locale && m.modListFilters_options_diskSpace()
	} as Record<SortBy, string>);

	function selectSort(option: SortBy) {
		queryArgs.sortBy = option;
		sortOpen = false;
	}
</script>

<div class="z-filters">
	<div class="z-filters-row">
		<Input
			bind:value={queryArgs.searchTerm}
			placeholder={i18nState.locale && m.modListFilters_searchBar_placeholder()}
			class="z-search-input"
		>
			{#snippet iconLeft()}
				<Icon icon="mdi:magnify" />
			{/snippet}
		</Input>

		<Tooltip
			text={i18nState.locale && m.modListFilters_filterTooltip()}
			position="bottom"
			delay={200}
		>
			<button class="z-filter-btn" class:active={expanded} onclick={() => (expanded = !expanded)}>
				<Icon icon="mdi:filter-variant" />
			</button>
		</Tooltip>

		<!-- Custom sort dropdown -->
		<div class="z-sort-group">
			<div class="z-sort-wrapper">
				<button class="z-sort-trigger" onclick={() => (sortOpen = !sortOpen)}>
					<span>{sortLabels[queryArgs.sortBy]}</span>
					<Icon icon="mdi:chevron-down" class="z-sort-chevron {sortOpen ? 'open' : ''}" />
				</button>

				{#if sortOpen}
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div class="z-sort-dropdown" onmouseleave={() => (sortOpen = false)}>
						{#each sortOptions as option}
							<button
								class="z-sort-option"
								class:active={queryArgs.sortBy === option}
								onclick={() => selectSort(option)}
							>
								{#if queryArgs.sortBy === option}
									<Icon icon="mdi:check" class="z-sort-check" />
								{/if}
								<span>{sortLabels[option]}</span>
							</button>
						{/each}
					</div>
				{/if}
			</div>

			<Tooltip
				text={i18nState.locale &&
					(queryArgs.sortOrder === 'ascending'
						? m.modListFilters_options_ascending()
						: m.modListFilters_options_descending())}
				position="bottom"
				delay={200}
			>
				<button
					class="z-sort-order"
					onclick={() => {
						queryArgs.sortOrder = queryArgs.sortOrder === 'ascending' ? 'descending' : 'ascending';
					}}
					aria-label={i18nState.locale &&
						(queryArgs.sortOrder === 'ascending'
							? m.modListFilters_options_ascending()
							: m.modListFilters_options_descending())}
				>
					<Icon
						icon={queryArgs.sortOrder === 'ascending'
							? 'mdi:sort-ascending'
							: 'mdi:sort-descending'}
					/>
				</button>
			</Tooltip>
		</div>

		{#if pageSize !== undefined && pageSizeChoices && onChangePageSize}
			<div class="z-sort-wrapper">
				<Tooltip text={i18nState.locale && m.mods_pageSize_label()} position="bottom" delay={200}>
					<button
						class="z-sort-trigger z-sort-trigger-solo"
						onclick={() => (pageSizeOpen = !pageSizeOpen)}
					>
						<Icon icon="mdi:format-list-numbered" />
						<span>{pageSize}</span>
						<Icon icon="mdi:chevron-down" class="z-sort-chevron {pageSizeOpen ? 'open' : ''}" />
					</button>
				</Tooltip>

				{#if pageSizeOpen}
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div class="z-sort-dropdown" onmouseleave={() => (pageSizeOpen = false)}>
						{#each pageSizeChoices as choice}
							<button
								class="z-sort-option"
								class:active={pageSize === choice}
								onclick={() => {
									onChangePageSize?.(choice);
									pageSizeOpen = false;
								}}
							>
								{#if pageSize === choice}
									<Icon icon="mdi:check" class="z-sort-check" />
								{/if}
								<span>{choice}</span>
							</button>
						{/each}
					</div>
				{/if}
			</div>
		{/if}

		<div class="z-view-toggle">
			<Tooltip text={i18nState.locale && m.modListFilters_viewList()} position="bottom" delay={200}>
				<button
					class="z-view-btn"
					class:active={viewMode === 'list'}
					onclick={() => (viewMode = 'list')}
					aria-label={i18nState.locale && m.modListFilters_viewList()}
				>
					<Icon icon="mdi:view-list" />
				</button>
			</Tooltip>
			<Tooltip text={i18nState.locale && m.modListFilters_viewGrid()} position="bottom" delay={200}>
				<button
					class="z-view-btn"
					class:active={viewMode === 'grid'}
					onclick={() => (viewMode = 'grid')}
					aria-label={i18nState.locale && m.modListFilters_viewGrid()}
				>
					<Icon icon="mdi:view-module" />
				</button>
			</Tooltip>
		</div>
	</div>

	{#if expanded && !externalPanel}
		<div class="z-filters-expanded">
			<label class="z-filter-toggle">
				<Checkbox bind:checked={queryArgs.includeNsfw} />
				<span>{i18nState.locale && m.modListFilters_options_NSFW()}</span>
			</label>
			<label class="z-filter-toggle">
				<Checkbox bind:checked={queryArgs.includeDeprecated} />
				<span>{i18nState.locale && m.modListFilters_options_deprecated()}</span>
			</label>

			{#if showCategories && games.categories.length > 0}
				<div class="z-filter-categories">
					{#each games.categories.slice(0, 20) as cat}
						<button
							class="z-category-chip"
							class:active={queryArgs.includeCategories.includes(cat.name)}
							onclick={() => {
								const idx = queryArgs.includeCategories.indexOf(cat.name);
								if (idx >= 0) {
									queryArgs.includeCategories = queryArgs.includeCategories.filter(
										(c) => c !== cat.name
									);
								} else {
									queryArgs.includeCategories = [...queryArgs.includeCategories, cat.name];
								}
							}}
						>
							{cat.name}
						</button>
					{/each}
				</div>
			{/if}
		</div>
	{/if}
</div>

<style>
	.z-filters {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		padding-bottom: var(--space-md);
	}

	.z-filters-row {
		display: flex;
		gap: var(--space-sm);
		align-items: center;
	}

	:global(.z-search-input) {
		flex: 1;
	}

	.z-filter-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		flex-shrink: 0;
		border-radius: var(--radius-md);
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
		font-size: 18px;
	}

	.z-filter-btn:hover,
	.z-filter-btn.active {
		color: var(--text-accent);
		border-color: var(--border-accent);
		background: var(--bg-active);
	}

	.z-filter-btn.disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.z-filter-btn.disabled:hover {
		color: var(--text-muted);
		border-color: var(--border-default);
		background: var(--bg-elevated);
	}

	/* Sort group */
	.z-sort-group {
		display: flex;
		align-items: center;
		gap: 2px;
	}

	.z-sort-wrapper {
		position: relative;
	}

	.z-sort-trigger {
		display: flex;
		align-items: center;
		gap: 6px;
		height: 36px;
		padding: 0 12px;
		border-radius: var(--radius-md) 0 0 var(--radius-md);
		border: 1px solid var(--border-default);
		border-right: none;
		background: var(--bg-elevated);
		color: var(--text-primary);
		font-family: var(--font-body);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all var(--transition-fast);
		white-space: nowrap;
	}

	.z-sort-trigger.z-sort-trigger-solo {
		border-radius: var(--radius-md);
		border-right: 1px solid var(--border-default);
	}

	.z-sort-trigger:hover {
		border-color: var(--border-strong);
		background: var(--bg-overlay);
	}

	:global(.z-sort-chevron) {
		font-size: 16px;
		color: var(--text-muted);
		transition: transform var(--transition-fast);
	}

	:global(.z-sort-chevron.open) {
		transform: rotate(180deg);
	}

	.z-sort-dropdown {
		position: absolute;
		top: calc(100% + 4px);
		right: 0;
		min-width: 160px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		padding: 4px;
		z-index: var(--z-dropdown);
		box-shadow: var(--shadow-lg);
		animation: scaleIn 100ms ease;
	}

	.z-sort-option {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 8px 12px;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		font-family: var(--font-body);
		font-size: 13px;
		cursor: pointer;
		transition: all var(--transition-fast);
		text-align: left;
	}

	.z-sort-option:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.z-sort-option.active {
		color: var(--text-accent);
		background: var(--bg-active);
	}

	:global(.z-sort-check) {
		font-size: 14px;
		color: var(--text-accent);
	}

	.z-sort-order {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		border-radius: 0 var(--radius-md) var(--radius-md) 0;
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
		font-size: 16px;
	}

	.z-sort-order:hover {
		color: var(--text-primary);
		background: var(--bg-overlay);
	}

	/* Filters expanded */
	.z-filters-expanded {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-sm);
		padding: var(--space-md);
		border-radius: var(--radius-md);
		background: var(--bg-elevated);
		border: 1px solid var(--border-subtle);
		animation: slideDown var(--transition-fast) ease;
	}

	.z-filter-toggle {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
		color: var(--text-secondary);
		cursor: pointer;
		padding: 4px 10px;
		border-radius: var(--radius-sm);
		transition: background var(--transition-fast);
	}

	.z-filter-toggle:hover {
		background: var(--bg-hover);
	}

	.z-filter-categories {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		width: 100%;
		padding-top: var(--space-sm);
		border-top: 1px solid var(--border-subtle);
	}

	.z-category-chip {
		padding: 3px 10px;
		border-radius: var(--radius-full);
		font-size: 11px;
		border: 1px solid var(--border-subtle);
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.z-category-chip:hover {
		border-color: var(--border-default);
		color: var(--text-secondary);
	}

	.z-category-chip.active {
		background: var(--bg-active);
		border-color: var(--border-accent);
		color: var(--text-accent);
	}

	/* View toggle */
	.z-view-toggle {
		display: flex;
		gap: 1px;
		background: var(--bg-elevated);
		border-radius: var(--radius-md);
		border: 1px solid var(--border-subtle);
		overflow: hidden;
	}

	.z-view-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 34px;
		height: 34px;
		background: transparent;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
		font-size: 18px;
	}

	.z-view-btn:hover {
		color: var(--text-secondary);
		background: var(--bg-hover);
	}

	.z-view-btn.active {
		color: var(--text-accent);
		background: var(--bg-active);
	}
</style>
