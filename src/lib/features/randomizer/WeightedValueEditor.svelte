<script lang="ts">
	import Icon from '@iconify/svelte';
	import Dropdown from '$lib/components/ui/Dropdown.svelte';
	import NumberInput from '$lib/components/ui/NumberInput.svelte';
	import type { OptionDef, Value } from './types';
	import { RANDOM_VARIANTS, formatRandomRange, isRandomString, parseRandomRange } from './types';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	/** Rows above this get a scrollbar instead of pushing the rest of the form
	 *  off screen. Not a cap: Archipelago happily takes as many branches as you
	 *  can name, so the list itself is unbounded. */
	const SCROLL_AFTER_ROWS = 8;

	type Props = {
		option: OptionDef;
		value: Record<string, Value>;
		onchange: (next: Record<string, Value>) => void;
	};

	const { option, value, onchange }: Props = $props();

	type Row = { id: number; key: string; weight: number };
	let counter = 0;

	function rowsFromValue(v: Record<string, Value>): Row[] {
		return Object.entries(v).map(([k, w]) => ({
			id: counter++,
			key: k,
			weight: typeof w === 'number' ? w : 0
		}));
	}

	let rows = $state<Row[]>(rowsFromValue(value));

	// Re-seed if the parent replaces the value (mode toggle, reset, preset).
	let lastSerialized = JSON.stringify(value);
	$effect(() => {
		const serialized = JSON.stringify(value);
		if (serialized !== lastSerialized) {
			lastSerialized = serialized;
			rows = rowsFromValue(value);
		}
	});

	// All possible keys for this option (used by the dropdown).
	const baseKeyChoices = $derived.by((): { value: string; label: string }[] => {
		switch (option.type.kind) {
			case 'toggle':
				return [
					{ value: 'true', label: i18nState.locale && m.randomizer_on() },
					{ value: 'false', label: i18nState.locale && m.randomizer_off() }
				];
			case 'select':
				return option.type.choices.map((c) => ({ value: c.value, label: c.label }));
			case 'range': {
				const t = option.type;
				const out: { value: string; label: string }[] = [];
				// For ranges, list the bounds plus midpoint as suggestions; the
				// numeric input below lets users pick any value in [min, max].
				const mid = Math.round((t.min + t.max) / 2);
				const candidates = [t.min, mid, t.max].filter((v, i, a) => a.indexOf(v) === i);
				for (const c of candidates) out.push({ value: String(c), label: String(c) });
				return out;
			}
			default:
				return [];
		}
	});

	const randomKeyChoices = $derived(RANDOM_VARIANTS.map((v) => ({ value: v, label: v })));

	const allKeyChoices = $derived([...baseKeyChoices, ...randomKeyChoices]);

	const isRange = $derived(option.type.kind === 'range');

	const rangeType = $derived(option.type.kind === 'range' ? option.type : null);

	/** Suggestions offered on the editable key field of a range option: the
	 *  bounds and midpoint, then every random keyword Archipelago accepts. Any
	 *  other number in range can still be typed. */
	const rangeSuggestions = $derived.by((): { value: string; label: string }[] => {
		const t = rangeType;
		if (!t) return [];
		const mid = Math.round((t.min + t.max) / 2);
		const numbers = [t.min, mid, t.max].filter((v, i, a) => a.indexOf(v) === i).map(String);
		return [
			...numbers,
			...RANDOM_VARIANTS,
			formatRandomRange({ skew: 'even', min: t.min, max: t.max }),
			formatRandomRange({ skew: 'low', min: t.min, max: t.max }),
			formatRandomRange({ skew: 'middle', min: t.min, max: t.max }),
			formatRandomRange({ skew: 'high', min: t.min, max: t.max })
		].map((suggestion) => ({ value: suggestion, label: suggestion }));
	});

	/** Mirrors the backend's weighted-key validation so a bad key is obvious
	 *  before the YAML is regenerated. Only range options can go wrong here;
	 *  every other type picks its key from a dropdown. */
	function keyInvalid(key: string): boolean {
		const t = rangeType;
		if (!t) return false;
		if (isRandomString(key)) return false;
		const range = parseRandomRange(key);
		if (range) return range.min > range.max || range.min < t.min || range.max > t.max;
		if (/^\d+$/.test(key.trim())) {
			const n = parseInt(key, 10);
			return n < t.min || n > t.max;
		}
		return true;
	}

	/** Keys used by more than one row. YAML mappings can't hold duplicates, so
	 *  these silently collapse — call it out rather than losing a row. */
	const duplicateKeys = $derived.by((): Set<string> => {
		const seen = new Set<string>();
		const dupes = new Set<string>();
		for (const row of rows) {
			if (!row.key) continue;
			if (seen.has(row.key)) dupes.add(row.key);
			else seen.add(row.key);
		}
		return dupes;
	});

	function emit(next: Row[]) {
		const map: Record<string, Value> = {};
		for (const r of next) {
			if (!r.key) continue;
			// Last write wins on duplicate keys
			map[r.key] = r.weight;
		}
		onchange(map);
	}

	function nextDefaultKey(): string {
		const used = new Set(rows.map((r) => r.key));
		for (const c of baseKeyChoices) {
			if (!used.has(c.value)) return c.value;
		}
		// Base choices exhausted: fall back to the random keywords, then to any
		// remaining number in a range, so a new row never lands on a duplicate.
		for (const v of RANDOM_VARIANTS) {
			if (!used.has(v)) return v;
		}
		const t = rangeType;
		if (t) {
			for (let n = t.min; n <= t.max; n += t.step || 1) {
				if (!used.has(String(n))) return String(n);
			}
		}
		return baseKeyChoices[0]?.value ?? 'random';
	}

	function addRow() {
		const key = nextDefaultKey();
		const next = [...rows, { id: counter++, key, weight: 50 }];
		rows = next;
		emit(next);
	}

	function removeRow(id: number) {
		const next = rows.filter((r) => r.id !== id);
		rows = next;
		emit(next);
	}

	function setKey(id: number, key: string) {
		const next = rows.map((r) => (r.id === id ? { ...r, key } : r));
		rows = next;
		emit(next);
	}

	function setWeight(id: number, weight: number) {
		const safe = Math.max(0, Math.min(100, Math.floor(weight)));
		const next = rows.map((r) => (r.id === id ? { ...r, weight: safe } : r));
		rows = next;
		emit(next);
	}

	function setRangeKey(id: number, raw: string) {
		// For range options, allow either a numeric key or one of the random variants.
		const trimmed = raw.trim();
		setKey(id, trimmed);
	}
</script>

<div class="rdz-weighted">
	{#if rows.length === 0}
		<div class="rdz-weighted-empty">
			{i18nState.locale && m.randomizer_weighted_empty()}
		</div>
	{:else}
		<div class="rdz-weighted-list" class:scrollable={rows.length > SCROLL_AFTER_ROWS}>
			{#each rows as row (row.id)}
				<div class="rdz-weighted-row">
					{#if isRange}
						<Dropdown
							editable
							options={rangeSuggestions}
							value={row.key}
							onchange={(v) => setRangeKey(row.id, v)}
							invalid={keyInvalid(row.key)}
							placeholder={i18nState.locale && m.randomizer_weighted_value()}
						/>
					{:else}
						<Dropdown options={allKeyChoices} value={row.key} onchange={(v) => setKey(row.id, v)} />
					{/if}
					<div class="rdz-weighted-weight">
						<NumberInput
							value={row.weight}
							min={0}
							max={100}
							onchange={(v) => setWeight(row.id, v)}
						/>
					</div>
					<button
						class="rdz-weighted-remove"
						onclick={() => removeRow(row.id)}
						aria-label={i18nState.locale && m.randomizer_weighted_remove()}
					>
						<Icon icon="mdi:close" />
					</button>
				</div>
			{/each}
		</div>
		{#if duplicateKeys.size > 0}
			<p class="rdz-weighted-warn">
				<Icon icon="mdi:alert-circle-outline" />
				{i18nState.locale &&
					m.randomizer_weighted_duplicate({ keys: [...duplicateKeys].join(', ') })}
			</p>
		{/if}
	{/if}

	<button class="rdz-weighted-add" onclick={addRow}>
		<Icon icon="mdi:plus" />
		{i18nState.locale && m.randomizer_weighted_addRow()}
	</button>
</div>

<style>
	.rdz-weighted {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
	}

	.rdz-weighted-empty {
		font-size: 11px;
		color: var(--text-muted);
		font-style: italic;
		padding: 4px 0;
	}

	.rdz-weighted-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
	}

	/* Long weight tables stay inside the option card instead of stretching it. */
	.rdz-weighted-list.scrollable {
		max-height: 296px;
		overflow-y: auto;
		padding-right: 4px;
	}

	.rdz-weighted-warn {
		display: flex;
		align-items: center;
		gap: 4px;
		margin: 0;
		font-size: 11px;
		font-weight: 600;
		color: #ffb74d;
	}

	.rdz-weighted-warn :global(svg) {
		font-size: 13px;
		flex-shrink: 0;
	}

	.rdz-weighted-row {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto auto;
		align-items: center;
		gap: var(--space-sm);
	}

	.rdz-weighted-weight {
		flex-shrink: 0;
	}

	.rdz-weighted-remove {
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

	.rdz-weighted-remove:hover {
		border-color: var(--color-error, #e05252);
		color: var(--color-error, #e05252);
		background: rgba(224, 82, 82, 0.08);
	}

	.rdz-weighted-add {
		align-self: flex-start;
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 5px 12px;
		border: 1px dashed var(--border-default);
		border-radius: var(--radius-md);
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 11px;
		font-weight: 600;
		transition: all var(--transition-fast);
	}

	.rdz-weighted-add:hover:not(:disabled) {
		border-color: var(--accent-400);
		color: var(--accent-400);
		background: var(--bg-active);
	}

	.rdz-weighted-add:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
</style>
