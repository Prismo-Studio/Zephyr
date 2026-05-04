<script lang="ts">
	import Icon from '@iconify/svelte';
	import Dropdown from '$lib/components/ui/Dropdown.svelte';
	import NumberInput from '$lib/components/ui/NumberInput.svelte';
	import type { OptionDef, Value } from './types';
	import { RANDOM_VARIANTS } from './types';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	const MAX_ENTRIES = 3;

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

	const randomKeyChoices = $derived(
		RANDOM_VARIANTS.map((v) => ({ value: v, label: v }))
	);

	const allKeyChoices = $derived([...baseKeyChoices, ...randomKeyChoices]);

	const isRange = $derived(option.type.kind === 'range');

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
		// Pick the first key not already used
		const used = new Set(rows.map((r) => r.key));
		for (const c of baseKeyChoices) {
			if (!used.has(c.value)) return c.value;
		}
		return baseKeyChoices[0]?.value ?? 'random';
	}

	function addRow() {
		if (rows.length >= MAX_ENTRIES) return;
		const key = nextDefaultKey();
		const next = [...rows, { id: counter++, key, weight: 50 }];
		rows = next;
		emit(next);
	}

	const canAdd = $derived(rows.length < MAX_ENTRIES);

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
		<div class="rdz-weighted-list">
			{#each rows as row (row.id)}
				<div class="rdz-weighted-row">
					{#if isRange}
						{#if (RANDOM_VARIANTS as readonly string[]).includes(row.key)}
							<Dropdown
								options={allKeyChoices}
								value={row.key}
								onchange={(v) => setRangeKey(row.id, v)}
							/>
						{:else}
							<input
								class="rdz-weighted-range-input"
								type="text"
								value={row.key}
								oninput={(e) =>
									setRangeKey(row.id, (e.currentTarget as HTMLInputElement).value)}
								placeholder={i18nState.locale && m.randomizer_weighted_value()}
							/>
						{/if}
					{:else}
						<Dropdown
							options={allKeyChoices}
							value={row.key}
							onchange={(v) => setKey(row.id, v)}
						/>
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
	{/if}

	<button class="rdz-weighted-add" onclick={addRow} disabled={!canAdd}>
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

	.rdz-weighted-row {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto auto;
		align-items: center;
		gap: var(--space-sm);
	}

	.rdz-weighted-range-input {
		width: 100%;
		padding: 6px 10px;
		border: 1px solid var(--border-default);
		border-radius: var(--radius-sm);
		background: var(--bg-elevated);
		color: var(--text-primary);
		font-size: 12px;
		font-family: var(--font-mono, monospace);
		outline: none;
		box-sizing: border-box;
		transition: border-color var(--transition-fast);
	}

	.rdz-weighted-range-input:focus {
		border-color: var(--accent-400);
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
