<script lang="ts">
	import Icon from '@iconify/svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import NumberInput from '$lib/components/ui/NumberInput.svelte';
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

	// Reset when store's start_inventory is cleared externally (reset to defaults, game change).
	$effect(() => {
		const storeVal = randomizerStore.values['start_inventory'];
		if (!storeVal) rows = [];
	});

	const allItems = $derived(randomizerStore.currentSchema?.items ?? []);

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

<section class="rdz-group">
	<button class="rdz-group-header" onclick={() => (isOpen = !isOpen)}>
		<Icon icon="mdi:treasure-chest" />
		<span class="rdz-group-name">Start Inventory</span>
		<span class="rdz-group-count">{itemCount}</span>
		<Icon
			icon="mdi:chevron-down"
			class={isOpen ? 'rdz-chevron' : 'rdz-chevron rdz-chevron-collapsed'}
		/>
	</button>

	{#if isOpen}
		<div class="rdz-inv-body">
			<p class="rdz-inv-desc">
				Items the player starts with at the beginning of the game. Item names must match exactly,
				check the setup guide if unsure.
			</p>

			{#if rows.length > 0}
				<div class="rdz-inv-list">
					{#each rows as row (row.id)}
						<div class="rdz-inv-row">
							<div class="rdz-inv-name">
								<Input
									value={row.name}
									placeholder="Item name…"
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
								aria-label="Remove item"
							>
								<Icon icon="mdi:close" />
							</button>
						</div>
					{/each}
				</div>
			{/if}

			<button class="rdz-inv-add" onclick={addRow}>
				<Icon icon="mdi:plus" />
				Add Item
			</button>
		</div>
	{/if}
</section>

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
		gap: var(--space-xs);
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
</style>
