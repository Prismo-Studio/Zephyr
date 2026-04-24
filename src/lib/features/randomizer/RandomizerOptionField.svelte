<script lang="ts">
	import Icon from '@iconify/svelte';
	import Toggle from '$lib/components/ui/Toggle.svelte';
	import Dropdown from '$lib/components/ui/Dropdown.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import Slider from '$lib/components/ui/Slider.svelte';
	import type { OptionDef, Value } from './types';
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

	// What this value currently means (selected choice description only — toggles speak for themselves)
	const currentExplain = $derived.by(() => {
		if (option.type.kind === 'select' && typeof value === 'string') {
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
				<p class="rdz-field-desc">{option.description}</p>
			{/if}
		</div>

		<div class="rdz-field-control">
			{#if option.type.kind === 'toggle'}
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
	}

	.rdz-field-control {
		display: block;
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
