<script lang="ts">
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import ModListFilters from '$lib/components/mod-list/ModListFilters.svelte';
	import games from '$lib/state/game.svelte';
	import { modQuery } from '$lib/state/misc.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import type { SortBy } from '$lib/types';

	type Props = {
		expanded: boolean;
		viewMode: 'list' | 'grid';
		sortOptions: SortBy[];
		isAllSelected: boolean;
		visibleCount: number;
		showCurseForgeOnly: boolean;
		showCurseForgeToggle: boolean;
		ontoggleSelectAll: () => void;
	};

	let {
		expanded = $bindable(),
		viewMode = $bindable(),
		sortOptions,
		isAllSelected,
		visibleCount,
		showCurseForgeOnly = $bindable(),
		showCurseForgeToggle,
		ontoggleSelectAll
	}: Props = $props();

	function toggleCategory(name: string) {
		const cats = modQuery.current.includeCategories;
		modQuery.current.includeCategories = cats.includes(name)
			? cats.filter((c) => c !== name)
			: [...cats, name];
	}
</script>

<div class="z-browse-filters">
	<div class="z-browse-filters-row">
		{#if !expanded}
			<label class="z-master-checkbox-wrapper">
				<Checkbox checked={isAllSelected} onchange={ontoggleSelectAll} />
				<span class="z-master-checkbox-label">{i18nState.locale && m.batch_selectAll()}</span>
			</label>
		{/if}
		<div class="flex-1"></div>
		<ModListFilters
			queryArgs={modQuery.current}
			{sortOptions}
			externalPanel
			bind:expanded
			bind:viewMode
		/>
	</div>

	{#if expanded}
		<div class="z-browse-filter-panel">
			<label class="z-filter-toggle">
				<Checkbox bind:checked={modQuery.current.includeNsfw} />
				<span>{i18nState.locale && m.modListFilters_options_NSFW()}</span>
			</label>
			<label class="z-filter-toggle">
				<Checkbox bind:checked={modQuery.current.includeDeprecated} />
				<span>{i18nState.locale && m.modListFilters_options_deprecated()}</span>
			</label>
			{#if showCurseForgeToggle}
				<label class="z-filter-toggle">
					<Checkbox bind:checked={showCurseForgeOnly} />
					<span>CurseForge</span>
				</label>
			{/if}

			{#if games.categories.length > 0}
				<div class="z-filter-categories">
					{#each games.categories.slice(0, 20) as cat}
						<button
							class="z-category-chip"
							class:active={modQuery.current.includeCategories.includes(cat.name)}
							onclick={() => toggleCategory(cat.name)}
						>
							{cat.name}
						</button>
					{/each}
				</div>
			{/if}
		</div>

		<div class="z-browse-select-row">
			<label class="z-master-checkbox-wrapper">
				<Checkbox checked={isAllSelected} onchange={ontoggleSelectAll} />
				<span class="z-master-checkbox-label">{i18nState.locale && m.batch_selectAll()}</span>
			</label>
			<span class="z-browse-count">{visibleCount} mods</span>
		</div>
	{/if}
</div>

<style>
	.z-browse-filters {
		position: sticky;
		top: 0;
		z-index: 10;
		padding-top: var(--space-sm);
		padding-bottom: var(--space-xs);
		background: var(--bg-base);
		border-bottom: 1px solid var(--border-subtle);
		margin-bottom: var(--space-sm);
	}

	.z-browse-filters-row {
		display: flex;
		align-items: center;
		gap: var(--space-md);
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

	.z-browse-filter-panel {
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

	.z-browse-select-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding-top: var(--space-xs);
	}

	.z-browse-count {
		font-size: 11px;
		color: var(--text-muted);
	}
</style>
