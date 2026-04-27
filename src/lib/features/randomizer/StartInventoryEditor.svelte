<script lang="ts">
	import Icon from '@iconify/svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import NumberInput from '$lib/components/ui/NumberInput.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { randomizerStore } from './randomizer.store.svelte';
	import type { Value } from './types';

	type Row = { id: number; name: string; count: number };
	let counter = 0;

	function rowsFromStore(): Row[] {
		const val = randomizerStore.values['start_inventory'];
		if (!val || typeof val !== 'object' || Array.isArray(val)) return [];
		return Object.entries(val as Record<string, unknown>)
			.filter(([name, cnt]) => name.length > 0 && typeof cnt === 'number' && cnt > 0)
			.map(([name, cnt]) => ({ id: counter++, name, count: cnt as number }));
	}

	let rows = $state<Row[]>(rowsFromStore());
	let isOpen = $state(false);
	let focusedRowId = $state<number | null>(null);
	let activeSuggestionIdx = $state(0);
	let browserOpen = $state(false);
	let browserQuery = $state('');

	// Reset when store's start_inventory is cleared externally (reset to defaults, game change).
	$effect(() => {
		const storeVal = randomizerStore.values['start_inventory'];
		if (!storeVal) rows = [];
	});

	const allItems = $derived(randomizerStore.currentSchema?.items ?? []);

	const isMetaArchipelago = $derived.by(() => {
		const s = randomizerStore.currentSchema;
		if (!s) return false;
		const id = s.id?.toLowerCase();
		const name = s.name?.toLowerCase();
		return id === 'generic' || id === 'archipelago' || name === 'archipelago';
	});

	const showEditor = $derived(allItems.length > 0 && !isMetaArchipelago);

	const suggestions = $derived.by(() => {
		if (focusedRowId === null) return [] as string[];
		const row = rows.find((r) => r.id === focusedRowId);
		if (!row || !row.name.trim()) return [] as string[];
		const q = row.name.toLowerCase();
		return allItems.filter((item) => item.toLowerCase().includes(q)).slice(0, 12);
	});

	function sync() {
		const map: { [key: string]: Value } = {};
		for (const row of rows) {
			const name = row.name.trim();
			if (name && row.count > 0) {
				map[name] = ((map[name] as number) ?? 0) + row.count;
			}
		}
		randomizerStore.setValue('start_inventory', map);
	}

	function addRow() {
		rows.push({ id: counter++, name: '', count: 1 });
	}

	function addRowFromBrowser(name: string) {
		const existing = rows.find((r) => r.name === name);
		if (existing) {
			existing.count = Math.min(9999, existing.count + 1);
		} else {
			rows.push({ id: counter++, name, count: 1 });
		}
		sync();
	}

	function decrementFromBrowser(name: string) {
		const existing = rows.find((r) => r.name === name);
		if (!existing) return;
		if (existing.count <= 1) {
			rows = rows.filter((r) => r.id !== existing.id);
		} else {
			existing.count -= 1;
		}
		sync();
	}

	const filteredItems = $derived.by(() => {
		const q = browserQuery.trim().toLowerCase();
		if (!q) return allItems;
		return allItems.filter((it) => it.toLowerCase().includes(q));
	});

	const countMap = $derived.by(() => {
		const map = new Map<string, number>();
		for (const r of rows) {
			if (r.name.trim()) map.set(r.name, (map.get(r.name) ?? 0) + r.count);
		}
		return map;
	});

	function removeRow(id: number) {
		rows = rows.filter((r) => r.id !== id);
		sync();
	}

	function selectSuggestion(row: Row, suggestion: string) {
		row.name = suggestion;
		focusedRowId = null;
		sync();
	}

	function handleKeydown(e: KeyboardEvent, row: Row) {
		if (suggestions.length === 0) return;
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			activeSuggestionIdx = (activeSuggestionIdx + 1) % suggestions.length;
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			activeSuggestionIdx = (activeSuggestionIdx - 1 + suggestions.length) % suggestions.length;
		} else if (e.key === 'Enter') {
			e.preventDefault();
			selectSuggestion(row, suggestions[activeSuggestionIdx]);
		} else if (e.key === 'Escape') {
			focusedRowId = null;
		}
	}

	function highlightMatch(item: string, query: string): string {
		const esc = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
		const idx = item.toLowerCase().indexOf(query.toLowerCase());
		if (idx === -1) return esc(item);
		return (
			esc(item.slice(0, idx)) +
			'<strong>' +
			esc(item.slice(idx, idx + query.length)) +
			'</strong>' +
			esc(item.slice(idx + query.length))
		);
	}

	const itemCount = $derived(rows.filter((r) => r.name.trim() && r.count > 0).length);
</script>

{#if showEditor}
	<section class="rdz-group">
		<button class="rdz-group-header" onclick={() => (isOpen = !isOpen)}>
			<Icon icon="mdi:treasure-chest" />
			<span class="rdz-group-name">{i18nState.locale && m.randomizer_inventory_title()}</span>
			<span class="rdz-group-count">{itemCount}</span>
			<Icon
				icon="mdi:chevron-down"
				class={isOpen ? 'rdz-chevron' : 'rdz-chevron rdz-chevron-collapsed'}
			/>
		</button>

		{#if isOpen}
			<div class="rdz-inv-body">
				<p class="rdz-inv-desc">
					{i18nState.locale && m.randomizer_inventory_desc()}
				</p>

				{#if rows.length > 0}
					<div class="rdz-inv-list">
						{#each rows as row (row.id)}
							<div class="rdz-inv-row">
								<div class="rdz-inv-name">
									<Input
										value={row.name}
										placeholder={i18nState.locale && m.randomizer_inventory_itemPlaceholder()}
										onfocus={() => {
											focusedRowId = row.id;
											activeSuggestionIdx = 0;
										}}
										onblur={() => {
											focusedRowId = null;
										}}
										oninput={(e) => {
											row.name = (e.currentTarget as HTMLInputElement).value;
											focusedRowId = row.id;
											activeSuggestionIdx = 0;
											sync();
										}}
										onkeydown={(e) => handleKeydown(e, row)}
									/>
									{#if focusedRowId === row.id && suggestions.length > 0}
										<div class="rdz-inv-suggestions">
											{#each suggestions as suggestion, i}
												<button
													class="rdz-inv-suggestion"
													class:active={i === activeSuggestionIdx}
													onmousedown={(e) => {
														e.preventDefault();
														selectSuggestion(row, suggestion);
													}}
												>
													{@html highlightMatch(suggestion, row.name)}
												</button>
											{/each}
										</div>
									{/if}
								</div>
								<div class="rdz-inv-count">
									<NumberInput
										value={row.count}
										min={1}
										max={9999}
										onchange={(v) => {
											row.count = v;
											sync();
										}}
									/>
								</div>
								<button
									class="rdz-inv-remove"
									onclick={() => removeRow(row.id)}
									aria-label={i18nState.locale && m.randomizer_inventory_remove()}
								>
									<Icon icon="mdi:close" />
								</button>
							</div>
						{/each}
					</div>
				{/if}

				<div class="rdz-inv-actions">
					<button class="rdz-inv-add" onclick={addRow}>
						<Icon icon="mdi:plus" />
						{i18nState.locale && m.randomizer_inventory_addItem()}
					</button>
					<button class="rdz-inv-browse" onclick={() => (browserOpen = true)}>
						<Icon icon="mdi:format-list-bulleted" />
						{i18nState.locale && m.randomizer_inventory_browseItems()} ({allItems.length})
					</button>
				</div>
			</div>
		{/if}
	</section>

	<Modal bind:open={browserOpen} title={i18nState.locale && m.randomizer_inventory_browseTitle()}>
		<div class="rdz-inv-browser">
			<Input
				bind:value={browserQuery}
				placeholder={i18nState.locale && m.randomizer_inventory_browseSearch()}
			>
				{#snippet iconLeft()}<Icon icon="mdi:magnify" />{/snippet}
			</Input>
			<div class="rdz-inv-browser-meta">
				{i18nState.locale &&
					m.randomizer_inventory_browseCount({
						filtered: String(filteredItems.length),
						total: String(allItems.length)
					})}
			</div>
			<div class="rdz-inv-browser-list">
				{#each filteredItems as item (item)}
					{@const count = countMap.get(item) ?? 0}
					<div class="rdz-inv-browser-item" class:added={count > 0}>
						<button
							type="button"
							class="rdz-inv-browser-name-btn"
							onclick={() => addRowFromBrowser(item)}
							title={count > 0
								? `${i18nState.locale && m.randomizer_inventory_inInventory()} · ${count}`
								: (i18nState.locale && m.randomizer_inventory_addItem()) || ''}
						>
							<span class="rdz-inv-browser-name">{item}</span>
						</button>

						{#if count > 0}
							<div class="rdz-inv-browser-counter" role="group">
								<button
									type="button"
									class="rdz-inv-browser-step"
									onclick={() => decrementFromBrowser(item)}
									aria-label={i18nState.locale && m.randomizer_inventory_decrease()}
								>
									<Icon icon="mdi:minus" />
								</button>
								<span class="rdz-inv-browser-count" aria-live="polite">{count}</span>
								<button
									type="button"
									class="rdz-inv-browser-step"
									onclick={() => addRowFromBrowser(item)}
									aria-label={i18nState.locale && m.randomizer_inventory_increase()}
								>
									<Icon icon="mdi:plus" />
								</button>
							</div>
						{:else}
							<button
								type="button"
								class="rdz-inv-browser-add"
								onclick={() => addRowFromBrowser(item)}
								aria-label={i18nState.locale && m.randomizer_inventory_addItem()}
							>
								<Icon icon="mdi:plus" />
							</button>
						{/if}
					</div>
				{/each}
				{#if filteredItems.length === 0}
					<div class="rdz-inv-browser-empty">
						{i18nState.locale && m.randomizer_inventory_browseEmpty({ query: browserQuery })}
					</div>
				{/if}
			</div>
		</div>
	</Modal>
{/if}

<style>
	.rdz-group {
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-lg);
		background: var(--bg-surface);
	}

	.rdz-group-header {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		width: 100%;
		padding: var(--space-md);
		border: none;
		background: transparent;
		color: var(--text-primary);
		cursor: pointer;
		font-size: 13px;
		font-weight: 600;
	}

	.rdz-group-header:hover {
		background: var(--bg-hover);
	}

	.rdz-group-name {
		flex: 1;
		text-align: left;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		font-size: 12px;
	}

	.rdz-group-count {
		font-size: 10px;
		font-weight: 700;
		padding: 2px 8px;
		border-radius: var(--radius-full);
		background: var(--bg-active);
		color: var(--text-muted);
	}

	.rdz-inv-body {
		padding: var(--space-md);
		border-top: 1px solid var(--border-subtle);
		background: var(--bg-elevated);
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.rdz-inv-desc {
		margin: 0;
		font-size: 11px;
		color: var(--text-muted);
		line-height: 1.4;
	}

	.rdz-inv-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
	}

	.rdz-inv-row {
		display: flex;
		align-items: center;
		gap: var(--space-md);
	}

	.rdz-inv-name {
		flex: 1;
		min-width: 0;
		position: relative;
	}

	.rdz-inv-count {
		flex-shrink: 0;
	}

	/* Suggestions dropdown */
	.rdz-inv-suggestions {
		position: absolute;
		top: calc(100% + 2px);
		left: 0;
		right: 0;
		z-index: 100;
		background: var(--bg-overlay);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		box-shadow: var(--shadow-lg);
		overflow: hidden;
	}

	.rdz-inv-suggestion {
		display: block;
		width: 100%;
		padding: 7px 12px;
		border: none;
		background: transparent;
		color: var(--text-secondary);
		font-size: 12px;
		text-align: left;
		cursor: pointer;
		transition: background var(--transition-fast);
	}

	.rdz-inv-suggestion:hover,
	.rdz-inv-suggestion.active {
		background: var(--bg-active);
		color: var(--text-primary);
	}

	.rdz-inv-suggestion :global(strong) {
		color: var(--accent-400);
		font-weight: 700;
	}

	.rdz-inv-remove {
		width: 30px;
		height: 30px;
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 15px;
		transition: all var(--transition-fast);
	}

	.rdz-inv-remove:hover {
		border-color: var(--color-error, #e05252);
		color: var(--color-error, #e05252);
		background: rgba(224, 82, 82, 0.08);
	}

	.rdz-inv-add {
		align-self: flex-start;
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 14px;
		border: 1px dashed var(--border-default);
		border-radius: var(--radius-md);
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 12px;
		font-weight: 600;
		transition: all var(--transition-fast);
	}

	.rdz-inv-add:hover {
		border-color: var(--accent-400);
		color: var(--accent-400);
		background: var(--bg-active);
	}

	.rdz-inv-actions {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}

	.rdz-inv-browse {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 6px 14px;
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 12px;
		font-weight: 600;
		transition: all var(--transition-fast);
	}

	.rdz-inv-browse:hover {
		border-color: var(--accent-400);
		color: var(--accent-400);
	}

	.rdz-inv-browser {
		display: flex;
		flex-direction: column;
		gap: 10px;
		min-width: 420px;
		max-width: 540px;
	}

	.rdz-inv-browser-meta {
		font-size: 11px;
		color: var(--text-muted);
		letter-spacing: 0.04em;
	}

	.rdz-inv-browser-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
		max-height: 50vh;
		overflow-y: auto;
		padding-right: 4px;
	}

	.rdz-inv-browser-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 4px 4px 4px 10px;
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--text-secondary);
		font-size: 12px;
		font-family: var(--font-mono, monospace);
		transition:
			background var(--transition-fast),
			color var(--transition-fast);
	}

	.rdz-inv-browser-item:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.rdz-inv-browser-item.added {
		background: color-mix(in srgb, var(--accent-400) 10%, transparent);
		color: var(--accent-400);
	}

	.rdz-inv-browser-item.added:hover {
		background: color-mix(in srgb, var(--accent-400) 16%, transparent);
	}

	.rdz-inv-browser-item :global(svg) {
		font-size: 13px;
		flex-shrink: 0;
	}

	.rdz-inv-browser-name-btn {
		flex: 1;
		min-width: 0;
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 3px 0;
		border: none;
		background: transparent;
		color: inherit;
		font: inherit;
		text-align: left;
		cursor: pointer;
	}

	.rdz-inv-browser-name {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.rdz-inv-browser-add {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition:
			border-color var(--transition-fast),
			color var(--transition-fast),
			background var(--transition-fast);
	}

	.rdz-inv-browser-add:hover {
		border-color: var(--accent-400);
		color: var(--accent-400);
	}

	.rdz-inv-browser-counter {
		display: inline-flex;
		align-items: center;
		gap: 2px;
		padding: 2px;
		border: 1px solid color-mix(in srgb, var(--accent-400) 35%, transparent);
		border-radius: var(--radius-md);
		background: var(--bg-overlay, var(--bg-base));
	}

	.rdz-inv-browser-step {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 22px;
		height: 22px;
		border: none;
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--accent-400);
		cursor: pointer;
		transition:
			background var(--transition-fast),
			color var(--transition-fast);
	}

	.rdz-inv-browser-step:hover {
		background: color-mix(in srgb, var(--accent-400) 18%, transparent);
	}

	.rdz-inv-browser-count {
		min-width: 22px;
		text-align: center;
		font-size: 12px;
		font-weight: 600;
		color: var(--text-primary);
		font-variant-numeric: tabular-nums;
	}

	.rdz-inv-browser-empty {
		padding: 24px 12px;
		text-align: center;
		color: var(--text-muted);
		font-size: 12px;
	}
</style>
