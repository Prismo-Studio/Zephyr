<script lang="ts">
	import Icon from '@iconify/svelte';
	import Toggle from '$lib/components/ui/Toggle.svelte';
	import Dropdown from '$lib/components/ui/Dropdown.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import Slider from '$lib/components/ui/Slider.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import WeightedValueEditor from './WeightedValueEditor.svelte';
	import type { OptionDef, RandomVariant, Value, ValueMode } from './types';
	import {
		RANDOM_VARIANTS,
		isRandomString,
		isWeightedMap,
		supportsRandomization,
		valueMode
	} from './types';
	import { randomizerStore, dependenciesSatisfied } from './randomizer.store.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	const { option }: { option: OptionDef } = $props();

	const visible = $derived(dependenciesSatisfied(option.dependencies, randomizerStore.values));
	const value = $derived(randomizerStore.values[option.id]);
	const dependents = $derived(randomizerStore.dependentsOf(option.id));

	const justChanged = $derived(randomizerStore.lastChangedId === option.id);
	const justImpacted = $derived(
		randomizerStore.lastImpact.newlyVisible.includes(option.id) ||
			randomizerStore.lastImpact.newlyHidden.includes(option.id)
	);

	const supportsRand = $derived(supportsRandomization(option));
	const mode = $derived<ValueMode>(valueMode(value));
	// random-low/middle/high are only meaningful for ordered (numeric) options.
	const supportsRandomVariants = $derived(option.type.kind === 'range');

	// What this value currently means (selected choice description only. Toggles speak for themselves)
	const currentExplain = $derived.by(() => {
		if (option.type.kind === 'select' && typeof value === 'string' && !isRandomString(value)) {
			const choice = option.type.choices.find((c) => c.value === value);
			return choice?.description ?? null;
		}
		return null;
	});

	function set(v: Value) {
		randomizerStore.setValue(option.id, v);
	}

	function toggleMulti(choiceValue: string, on: boolean) {
		const current = (Array.isArray(value) ? value : []) as string[];
		if (on && !current.includes(choiceValue)) {
			set([...current, choiceValue]);
		} else if (!on) {
			set(current.filter((v) => v !== choiceValue));
		}
	}

	function defaultFixedValue(): Value {
		switch (option.type.kind) {
			case 'toggle':
				return option.type.default;
			case 'range':
				return option.type.default;
			case 'select':
				return option.type.default;
			case 'multi_select':
				return [...option.type.defaults];
			case 'text':
				return option.type.default;
		}
	}

	/** Seed the weighted dict from the current fixed value when possible,
	 *  otherwise from the option's default. Always at weight 50 so a follow-up
	 *  entry naturally lands at "even split". */
	function seedWeightedFromCurrent(): Record<string, Value> {
		if (typeof value === 'boolean') return { [value ? 'true' : 'false']: 50 };
		if (typeof value === 'number') return { [String(value)]: 50 };
		if (typeof value === 'string' && !isRandomString(value)) return { [value]: 50 };
		// Fall back to the default
		switch (option.type.kind) {
			case 'toggle':
				return { [option.type.default ? 'true' : 'false']: 50 };
			case 'select':
				return { [option.type.default]: 50 };
			case 'range':
				return { [String(option.type.default)]: 50 };
			default:
				return {};
		}
	}

	function setMode(next: ValueMode) {
		if (next === mode) return;
		if (next === 'fixed') {
			set(defaultFixedValue());
		} else if (next === 'random') {
			set('random');
		} else {
			set(seedWeightedFromCurrent());
		}
	}

	function setRandomVariant(v: RandomVariant) {
		set(v);
	}

	const currentRandomVariant = $derived<RandomVariant>(
		isRandomString(value) ? value : 'random'
	);

	type DescSegment = { kind: 'text' | 'link'; value: string };

	/** Split a description string into plain-text and URL segments so URLs can
	 *  be rendered as clickable links. The global click delegate in
	 *  +layout.svelte intercepts any <a href="http..."> and opens it
	 *  externally via Tauri, so we don't need a local onclick handler. */
	function linkify(text: string): DescSegment[] {
		const re = /(https?:\/\/[^\s)]+)/g;
		const parts: DescSegment[] = [];
		let last = 0;
		for (const match of text.matchAll(re)) {
			const url = match[0];
			const idx = match.index ?? 0;
			if (idx > last) parts.push({ kind: 'text', value: text.slice(last, idx) });
			parts.push({ kind: 'link', value: url });
			last = idx + url.length;
		}
		if (last < text.length) parts.push({ kind: 'text', value: text.slice(last) });
		return parts;
	}

	const descSegments = $derived(option.description ? linkify(option.description) : []);
</script>

{#if visible}
	<div class="rdz-field" class:just-changed={justChanged} class:just-impacted={justImpacted}>
		<div class="rdz-field-head">
			<div class="rdz-field-label-row">
				<label for={`opt-${option.id}`}>{option.label}</label>
				{#if option.advanced}
					<span class="rdz-field-tag">{i18nState.locale && m.randomizer_advanced()}</span>
				{/if}
				{#if dependents.length > 0}
					<span
						class="rdz-field-tag rdz-field-tag-soft"
						title={`Affects ${dependents.length} option(s)`}
					>
						<Icon icon="mdi:link-variant" />
						{dependents.length}
					</span>
				{/if}
			</div>
			{#if option.description}
				<p class="rdz-field-desc">
					{#each descSegments as seg}
						{#if seg.kind === 'link'}
							<a class="rdz-field-desc-link" href={seg.value}>{seg.value}</a>
						{:else}
							{seg.value}
						{/if}
					{/each}
				</p>
			{/if}
		</div>

		{#if supportsRand}
			<div class="rdz-mode-row">
				<div class="rdz-mode-tabs" role="tablist">
					<button
						type="button"
						role="tab"
						class="rdz-mode-tab"
						class:active={mode === 'fixed'}
						aria-selected={mode === 'fixed'}
						onclick={() => setMode('fixed')}
					>
						<Icon icon="mdi:target" />
						<span>{i18nState.locale && m.randomizer_mode_fixed()}</span>
					</button>
					<button
						type="button"
						role="tab"
						class="rdz-mode-tab"
						class:active={mode === 'random'}
						aria-selected={mode === 'random'}
						onclick={() => setMode('random')}
					>
						<Icon icon="mdi:dice-multiple" />
						<span>{i18nState.locale && m.randomizer_mode_random()}</span>
					</button>
					<button
						type="button"
						role="tab"
						class="rdz-mode-tab"
						class:active={mode === 'weighted'}
						aria-selected={mode === 'weighted'}
						onclick={() => setMode('weighted')}
					>
						<Icon icon="mdi:chart-pie" />
						<span>{i18nState.locale && m.randomizer_mode_weighted()}</span>
					</button>
				</div>
				{#if mode === 'weighted'}
					<Tooltip
						text={i18nState.locale ? m.randomizer_mode_weightedDesc() : ''}
						position="top"
					>
						<span class="rdz-mode-info" aria-label="Weighted mode help">
							<Icon icon="mdi:information-outline" />
						</span>
					</Tooltip>
				{/if}
			</div>
		{/if}

		<div class="rdz-field-control">
			{#if mode === 'random'}
				{#if supportsRandomVariants}
					<Dropdown
						options={RANDOM_VARIANTS.map((v) => ({ value: v, label: v }))}
						value={currentRandomVariant}
						onchange={(v) => setRandomVariant(v as RandomVariant)}
					/>
				{:else}
					<div class="rdz-random-fixed">
						<Icon icon="mdi:dice-multiple" />
						<span>{i18nState.locale && m.randomizer_mode_randomDesc()}</span>
					</div>
				{/if}
			{:else if mode === 'weighted'}
				<WeightedValueEditor
					{option}
					value={isWeightedMap(value) ? value : {}}
					onchange={(next) => set(next)}
				/>
			{:else if option.type.kind === 'toggle'}
				<div class="rdz-toggle-row">
					<Toggle checked={value === true} onchange={(checked) => set(checked)} />
					<span class="rdz-toggle-state"
						>{value === true
							? i18nState.locale && m.randomizer_on()
							: i18nState.locale && m.randomizer_off()}</span
					>
				</div>
			{:else if option.type.kind === 'range'}
				{@const t = option.type}
				{@const cur = typeof value === 'number' ? value : t.default}
				<Slider value={cur} min={t.min} max={t.max} step={t.step} onchange={(v) => set(v)} />
			{:else if option.type.kind === 'select'}
				{@const t = option.type}
				<Dropdown
					options={t.choices.map((c) => ({ value: c.value, label: c.label }))}
					value={typeof value === 'string' ? value : t.default}
					onchange={(v) => set(v)}
				/>
			{:else if option.type.kind === 'multi_select'}
				{@const t = option.type}
				{@const selected = (Array.isArray(value) ? value : []) as string[]}
				<div class="rdz-checks">
					{#each t.choices as choice}
						<button
							type="button"
							class="rdz-check-row"
							onclick={() => toggleMulti(choice.value, !selected.includes(choice.value))}
						>
							<Checkbox
								checked={selected.includes(choice.value)}
								onchange={(c) => toggleMulti(choice.value, c)}
							/>
							<span>{choice.label}</span>
						</button>
					{/each}
				</div>
			{:else if option.type.kind === 'text'}
				{@const t = option.type}
				<Input
					value={typeof value === 'string' ? value : t.default}
					placeholder={t.placeholder ?? ''}
					oninput={(e) => set((e.currentTarget as HTMLInputElement).value)}
				/>
			{/if}
		</div>

		{#if currentExplain}
			<div class="rdz-field-explain">
				<Icon icon="mdi:information-outline" />
				<span>{currentExplain}</span>
			</div>
		{/if}
	</div>
{/if}

<style>
	.rdz-field {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		padding: var(--space-md);
		border-radius: var(--radius-md);
		background: var(--bg-base);
		border: 1px solid var(--border-subtle);
		box-sizing: border-box;
		min-width: 0;
		height: auto;
		min-height: fit-content;
		overflow: visible;
		transition:
			border-color 200ms ease,
			box-shadow 200ms ease;
	}

	.rdz-field.just-changed {
		border-color: var(--accent-400);
		box-shadow:
			0 0 0 1px var(--accent-400),
			var(--shadow-glow);
	}

	.rdz-field.just-impacted {
		animation: rdz-flash 600ms ease;
	}

	@keyframes rdz-flash {
		0% {
			background: var(--bg-active);
		}
		100% {
			background: var(--bg-base);
		}
	}

	.rdz-field-head {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.rdz-field-label-row {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		flex-wrap: wrap;
	}

	.rdz-field-label-row label {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.rdz-field-tag {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		font-size: 9px;
		font-weight: 700;
		letter-spacing: 0.05em;
		text-transform: uppercase;
		padding: 2px 6px;
		border-radius: var(--radius-full);
		background: rgba(255, 165, 0, 0.15);
		color: #ffb74d;
	}

	.rdz-field-tag-soft {
		background: var(--bg-active);
		color: var(--accent-400);
	}

	.rdz-field-desc {
		margin: 0;
		font-size: 11px;
		color: var(--text-muted);
		line-height: 1.4;
		/* Preserve author-written line breaks in multi-line option descriptions
		   without forcing horizontal overflow on long single lines. */
		white-space: pre-line;
		/* Long URLs (e.g. Google Doc share links) must wrap rather than blow
		   out the field width. */
		overflow-wrap: anywhere;
	}

	.rdz-field-desc-link {
		color: var(--accent-400);
		text-decoration: underline;
		cursor: pointer;
	}

	.rdz-field-desc-link:hover {
		filter: brightness(1.15);
	}

	.rdz-field-control {
		display: block;
	}

	/* Mode tabs (Fixed / Random / Weighted) */
	.rdz-mode-row {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-wrap: wrap;
	}

	.rdz-mode-tabs {
		display: inline-flex;
		gap: 2px;
		padding: 2px;
		border-radius: var(--radius-md);
		background: var(--bg-elevated);
		border: 1px solid var(--border-subtle);
	}

	.rdz-mode-info {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 22px;
		height: 22px;
		border-radius: var(--radius-full);
		color: var(--text-muted);
		cursor: help;
		transition: color var(--transition-fast);
	}

	.rdz-mode-info:hover {
		color: var(--accent-400);
	}

	.rdz-mode-info :global(svg) {
		font-size: 14px;
	}

	.rdz-mode-tab {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 3px 8px;
		border: none;
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		transition:
			background var(--transition-fast),
			color var(--transition-fast);
	}

	.rdz-mode-tab :global(svg) {
		font-size: 11px;
	}

	.rdz-mode-tab:hover {
		color: var(--text-secondary);
	}

	.rdz-mode-tab.active {
		background: var(--accent-400);
		color: var(--text-inverse);
	}

	.rdz-random-fixed {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 6px 10px;
		border-radius: var(--radius-sm);
		background: var(--bg-active);
		color: var(--accent-400);
		font-size: 12px;
		font-weight: 600;
	}

	.rdz-random-fixed :global(svg) {
		font-size: 14px;
	}

	/* Toggle */
	.rdz-toggle-row {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
	}

	.rdz-toggle-state {
		font-size: 12px;
		color: var(--text-muted);
		font-weight: 600;
	}

	/* Multi-select as a stack of checkbox rows */
	.rdz-checks {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.rdz-check-row {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: 4px 8px;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		font-size: 12px;
		text-align: left;
		transition: background var(--transition-fast);
	}

	.rdz-check-row:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	/* Explain banner */
	.rdz-field-explain {
		display: flex;
		align-items: flex-start;
		gap: 6px;
		padding: 6px 8px;
		border-radius: var(--radius-sm);
		background: var(--bg-active);
		border-left: 2px solid var(--accent-400);
		color: var(--text-secondary);
		font-size: 11px;
		line-height: 1.4;
	}

	.rdz-field-explain :global(svg) {
		font-size: 13px;
		color: var(--accent-400);
		flex-shrink: 0;
		margin-top: 1px;
	}
</style>
