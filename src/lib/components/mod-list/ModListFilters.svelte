<script lang="ts">
	import type { QueryModsArgsWithoutMax, SortBy, SortOrder } from '$lib/types';
	import Icon from '@iconify/svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import games from '$lib/state/game.svelte';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		queryArgs: QueryModsArgsWithoutMax;
		sortOptions?: SortBy[];
		showCategories?: boolean;
	};

	let {
		queryArgs,
		sortOptions = ['rating', 'downloads', 'lastUpdated', 'newest', 'name'],
		showCategories = false
	}: Props = $props();

	let showFilters = $state(false);
	let sortOpen = $state(false);

	const sortLabels: Record<SortBy, string> = {
		newest: m.modListFilters_options_newest(),
		name: m.modListFilters_options_name(),
		author: m.modListFilters_options_author(),
		lastUpdated: m.modListFilters_options_lastUpdated(),
		downloads: m.modListFilters_options_downloads(),
		rating: m.modListFilters_options_rating(),
		installDate: m.modListFilters_options_installDate(),
		custom: m.modListFilters_options_custom(),
		diskSpace: m.modListFilters_options_diskSpace()
	};

	function selectSort(option: SortBy) {
		queryArgs.sortBy = option;
		sortOpen = false;
	}
</script>

<div class="z-filters">
	<div class="z-filters-row">
		<Input
			bind:value={queryArgs.searchTerm}
			placeholder={m.modListFilters_searchBar_placeholder()}
			class="z-search-input"
		>
			{#snippet iconLeft()}
				<Icon icon="mdi:magnify" />
			{/snippet}
		</Input>

		<button
			class="z-filter-btn"
			class:active={showFilters}
			onclick={() => (showFilters = !showFilters)}
		>
			<Icon icon="mdi:filter-variant" />
		</button>

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

			<button
				class="z-sort-order"
				onclick={() => {
					queryArgs.sortOrder = queryArgs.sortOrder === 'ascending' ? 'descending' : 'ascending';
				}}
				title={queryArgs.sortOrder === 'ascending'
					? m.modListFilters_options_ascending()
					: m.modListFilters_options_descending()}
			>
				<Icon
					icon={queryArgs.sortOrder === 'ascending' ? 'mdi:sort-ascending' : 'mdi:sort-descending'}
				/>
			</button>
		</div>
	</div>

	{#if showFilters}
		<div class="z-filters-expanded">
			<label class="z-filter-toggle">
				<input type="checkbox" bind:checked={queryArgs.includeNsfw} />
				<span>{m.modListFilters_options_NSFW()}</span>
			</label>
			<label class="z-filter-toggle">
				<input type="checkbox" bind:checked={queryArgs.includeDeprecated} />
				<span>{m.modListFilters_options_deprecated()}</span>
			</label>

			{#if showCategories && games.categories.length > 0}
				<div class="z-filter-categories">
					{#each games.categories.slice(0, 20) as cat}
						<button
							class="z-category-chip"
							class:active={queryArgs.includeCategories.includes(cat.slug)}
							onclick={() => {
								const idx = queryArgs.includeCategories.indexOf(cat.slug);
								if (idx >= 0) {
									queryArgs.includeCategories = queryArgs.includeCategories.filter(
										(c) => c !== cat.slug
									);
								} else {
									queryArgs.includeCategories = [...queryArgs.includeCategories, cat.slug];
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

	.z-filter-toggle input {
		accent-color: var(--accent-400);
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
</style>
