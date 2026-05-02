<script lang="ts">
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import ModListFilters from './ModListFilters.svelte';
	import games from '$lib/state/game.svelte';
	import { profileQuery } from '$lib/state/misc.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import type { SortBy } from '$lib/types';

	type Props = {
		expanded: boolean;
		viewMode: 'list' | 'grid';
		sortOptions: SortBy[];
		pageSize: number;
		pageSizeChoices: readonly number[];
		isAllSelected: boolean;
		visibleCount: number;
		ontoggleSelectAll: () => void;
		onchangePageSize: (v: number) => void;
	};

	let {
		expanded = $bindable(),
		viewMode = $bindable(),
		sortOptions,
		pageSize,
		pageSizeChoices,
		isAllSelected,
		visibleCount,
		ontoggleSelectAll,
		onchangePageSize
	}: Props = $props();

	function toggleCategory(name: string) {
		const cats = profileQuery.current.includeCategories;
		profileQuery.current.includeCategories = cats.includes(name)
			? cats.filter((c) => c !== name)
			: [...cats, name];
	}
</script>

<div class="z-mods-filters">
	<div class="z-mods-filters-row">
		{#if !expanded}
			<label class="z-master-checkbox-wrapper">
				<Checkbox checked={isAllSelected} onchange={ontoggleSelectAll} />
				<span class="z-master-checkbox-label">{i18nState.locale && m.batch_selectAll()}</span>
			</label>
		{/if}
		<div class="flex-1"></div>
		<ModListFilters
			queryArgs={profileQuery.current}
			{sortOptions}
			externalPanel
			bind:expanded
			bind:viewMode
			{pageSize}
			pageSizeChoices={[...pageSizeChoices]}
			onChangePageSize={onchangePageSize}
		/>
	</div>

	{#if expanded}
		<div class="z-mods-filter-panel">
			<label class="z-filter-toggle">
				<Checkbox bind:checked={profileQuery.current.includeNsfw} />
				<span>{i18nState.locale && m.modListFilters_options_NSFW()}</span>
			</label>
			<label class="z-filter-toggle">
				<Checkbox bind:checked={profileQuery.current.includeDeprecated} />
				<span>{i18nState.locale && m.modListFilters_options_deprecated()}</span>
			</label>

			{#if games.categories.length > 0}
				<div class="z-filter-categories">
					{#each games.categories.slice(0, 20) as cat}
						<button
							class="z-category-chip"
							class:active={profileQuery.current.includeCategories.includes(cat.name)}
							onclick={() => toggleCategory(cat.name)}
						>
							{cat.name}
						</button>
					{/each}
				</div>
			{/if}
		</div>

		<div class="z-mods-select-row">
			<label class="z-master-checkbox-wrapper">
				<Checkbox checked={isAllSelected} onchange={ontoggleSelectAll} />
				<span class="z-master-checkbox-label">{i18nState.locale && m.batch_selectAll()}</span>
			</label>
			<span class="z-mods-count">{visibleCount} mods</span>
		</div>
	{/if}
</div>

<style>
	.z-mods-filters {
		position: sticky;
		top: 0;
		z-index: 10;
		padding-top: var(--space-sm);
		padding-bottom: var(--space-xs);
		background: var(--bg-base);
		border-bottom: 1px solid var(--border-subtle);
		margin-bottom: var(--space-sm);
	}

	.z-mods-filters-row {
		display: flex;
		align-items: center;
		gap: var(--space-md);
	}

	.z-mods-filter-panel {
		display: flex;
		flex-wrap: wrap;
		justify-content: flex-start;
		gap: var(--space-sm);
		padding: var(--space-md);
		border-radius: var(--radius-md);
		background: var(--bg-elevated);
		border: 1px solid var(--border-subtle);
		margin-top: var(--space-sm);
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
		justify-content: flex-start;
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

	.z-mods-select-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding-top: var(--space-xs);
	}

	.z-mods-count {
		font-size: 11px;
		color: var(--text-muted);
	}

	.z-master-checkbox-wrapper {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-xs) var(--space-sm);
		cursor: pointer;
		color: var(--text-muted);
		font-size: 13px;
		font-weight: 500;
		transition: color var(--transition-fast);
	}

	.z-master-checkbox-wrapper:hover {
		color: var(--text-primary);
	}

	.z-master-checkbox-label {
		user-select: none;
	}
</style>
