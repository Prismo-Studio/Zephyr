<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Dropdown from '$lib/components/ui/Dropdown.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import { open as openExternal } from '@tauri-apps/plugin-shell';
	import { pushToast } from '$lib/toast';
	import * as api from './api';
	import { randomizerStore, dependenciesSatisfied } from './randomizer.store.svelte';
	import { CATEGORY_ICONS, CATEGORY_LABELS, CATEGORY_ORDER, type OptionDef } from './types';
	import RandomizerOptionField from './RandomizerOptionField.svelte';
	import StartInventoryEditor from './StartInventoryEditor.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	// Hardcoded tutorial path overrides for older schemas that don't yet carry
	// `meta.tutorial_path` (pre-refresh bundled schemas). Once the user runs
	// "Refresh Schemas", `meta.tutorial_path` from the extractor supersedes
	// this map.
	const LEGACY_TUTORIAL_PATH: Record<string, string> = {
		alttp: 'multiworld/en',
		celeste64: 'guide/en',
		celeste_open_world: 'guide/en',
		ff1: 'multiworld/en',
		kh1: 'kh1/en',
		landstalker: 'landstalker_setup/en',
		paint: 'guide/en',
		sm: 'multiworld/en',
		smz3: 'multiworld/en',
		soe: 'multiworld/en',
		tloz: 'multiworld/en',
		wargroove: 'wargroove/en'
	};

	function setupGuideUrl(id: string, name: string, path: string): string {
		return `https://archipelago.gg/tutorial/${encodeURIComponent(name)}/${path}`;
	}

	function openSetupGuide(e: MouseEvent, id: string, name: string, path: string) {
		e.preventDefault();
		e.stopPropagation();
		openExternal(setupGuideUrl(id, name, path)).catch((err) => {
			console.error('open setup guide failed:', err);
		});
	}

	async function saveSlot() {
		if (!randomizerStore.generatedYaml) {
			await randomizerStore.refreshGenerated();
		}
		const slot = randomizerStore.playerName.trim() || 'Player1';
		try {
			const path = await api.savePlayerYaml(slot, randomizerStore.generatedYaml);
			pushToast({
				type: 'info',
				name: m.randomizer_slotSaved(),
				message: `${slot} -> ${path.split(/[/\\]/).pop()}`
			});
			// Tell the page to switch to the server tab and refresh the player list.
			window.dispatchEvent(new CustomEvent('rdz-player-saved'));
		} catch {
			// invoke() already toasted
		}
	}

	const { onBack }: { onBack: () => void } = $props();

	let openCategories = $state<Record<string, boolean>>({});
	let lastSchemaId: string | null = null;

	const schema = $derived(randomizerStore.currentSchema);

	const groupedOptions = $derived.by(() => {
		if (!schema) return [] as { category: string; options: OptionDef[] }[];
		const groups: Record<string, OptionDef[]> = {};
		for (const opt of schema.options) {
			if (opt.advanced && !randomizerStore.showAdvanced) continue;
			(groups[opt.category] ||= []).push(opt);
		}
		const ordered: { category: string; options: OptionDef[] }[] = [];
		for (const cat of CATEGORY_ORDER) {
			if (groups[cat]) ordered.push({ category: cat, options: groups[cat] });
		}
		// any unknown categories at the end
		for (const cat of Object.keys(groups)) {
			if (!CATEGORY_ORDER.includes(cat as (typeof CATEGORY_ORDER)[number])) {
				ordered.push({ category: cat, options: groups[cat] });
			}
		}
		return ordered;
	});

	$effect(() => {
		// open all categories by default the first time we see a given schema
		if (!schema) return;
		if (lastSchemaId === schema.id) return;
		lastSchemaId = schema.id;
		const next: Record<string, boolean> = {};
		for (const g of groupedOptions) next[g.category] = true;
		openCategories = next;
	});

	$effect(() => {
		// regenerate yaml whenever values / preset / seed / player change
		void randomizerStore.values;
		void randomizerStore.seed;
		void randomizerStore.playerName;
		if (schema) randomizerStore.refreshGenerated();
	});

	function visibleOptionCount(opts: OptionDef[]): number {
		return opts.filter((o) => dependenciesSatisfied(o.dependencies, randomizerStore.values)).length;
	}

	function toggleCat(cat: string) {
		openCategories[cat] = !openCategories[cat];
	}

	const presetOptions = $derived.by(() => {
		if (!schema) return [];
		return [
			{ value: '', label: m.randomizer_custom() },
			...schema.presets.map((p) => ({ value: p.id, label: p.name }))
		];
	});

	const hasPresets = $derived(!!schema && schema.presets.length > 0);
	const hasAdvanced = $derived(!!schema && schema.options.some((o) => o.advanced));

	// Decide whether to link to the archipelago.gg setup guide.
	// - Custom (non-bundled) APWorlds have no presence on archipelago.gg,
	//   so any tutorial URL we build would 404. Hide the button entirely.
	// - For bundled worlds, prefer the author-defined tutorial_path from the
	//   schema; fall back to the legacy override map, then to "setup/en".
	// A `false` is_official is authoritative; an undefined one means the
	// schema predates this field and is assumed official (all pre-refresh
	// bundled schemas are official).
	const setupGuide = $derived.by(() => {
		if (!schema) return null;
		if (schema.meta.is_official === false) return null;
		const path =
			schema.meta.tutorial_path ?? LEGACY_TUTORIAL_PATH[schema.id] ?? 'setup/en';
		return { url: setupGuideUrl(schema.id, schema.name, path), path };
	});

	const versionLabel = $derived.by(() => {
		if (!schema) return '';
		const v = schema.version?.trim() ?? '';
		if (!v || v === '0.0.0') return '';
		return `v${v}`;
	});

	const lastChangeBanner = $derived.by(() => {
		const id = randomizerStore.lastChangedId;
		if (!id || !schema) return null;
		const opt = schema.options.find((o) => o.id === id);
		if (!opt) return null;
		const impact = randomizerStore.lastImpact;
		if (impact.newlyVisible.length === 0 && impact.newlyHidden.length === 0) return null;
		const named = (ids: string[]) =>
			ids.map((i) => schema.options.find((o) => o.id === i)?.label ?? i).join(', ');
		return { option: opt, impact, named };
	});
</script>

{#if schema}
	<div class="rdz-config-main">
		<header class="rdz-config-header">
			<button
				class="rdz-back"
				onclick={onBack}
				aria-label={i18nState.locale && m.randomizer_backToCatalog()}
			>
				<Icon icon="mdi:arrow-left" />
			</button>
			<div class="rdz-config-title">
				<h1>{schema.name}</h1>
				{#if versionLabel}
					<small>{versionLabel}</small>
				{/if}
			</div>
			{#if setupGuide}
				<a
					class="rdz-setup-link"
					href={setupGuide.url}
					onclick={(e) => openSetupGuide(e, schema.id, schema.name, setupGuide.path)}
				>
					<Icon icon="mdi:book-open-variant" />
					{i18nState.locale && m.randomizer_setupGuide()}
				</a>
			{/if}

			<div class="rdz-config-controls">
				{#if hasPresets}
					<div class="rdz-inline-field">
						<span>{i18nState.locale && m.randomizer_preset()}</span>
						<div class="rdz-inline-control rdz-inline-control-md">
							<Dropdown
								options={presetOptions}
								value={randomizerStore.presetId ?? ''}
								onchange={(v) => {
									if (v) randomizerStore.applyPreset(v);
									else randomizerStore.resetToDefaults();
								}}
							/>
						</div>
					</div>
				{/if}

				<div class="rdz-inline-field">
					<span>{i18nState.locale && m.randomizer_player()}</span>
					<div class="rdz-inline-control rdz-inline-control-sm">
						<Input bind:value={randomizerStore.playerName} placeholder="Player1" />
					</div>
				</div>

				<div class="rdz-inline-field">
					<span>{i18nState.locale && m.randomizer_seed()}</span>
					<div class="rdz-inline-control rdz-inline-control-sm">
						<Input
							bind:value={randomizerStore.seed}
							placeholder={i18nState.locale && m.randomizer_random()}
						/>
					</div>
				</div>

				{#if hasAdvanced}
					<Button
						variant={randomizerStore.showAdvanced ? 'accent' : 'secondary'}
						size="sm"
						onclick={() => (randomizerStore.showAdvanced = !randomizerStore.showAdvanced)}
					>
						{#snippet icon()}
							<Icon icon="mdi:tune-variant" />
						{/snippet}
						{i18nState.locale && m.randomizer_advanced()}
					</Button>
				{/if}
			</div>
		</header>

		{#if lastChangeBanner}
			<div class="rdz-change-banner">
				<Icon icon="mdi:lightning-bolt" />
				<div>
					<strong>{lastChangeBanner.option.label}</strong>
					{i18nState.locale && m.randomizer_changed()}
					{#if lastChangeBanner.impact.newlyVisible.length > 0}
						{i18nState.locale && m.randomizer_nowVisible()}
						<em>{lastChangeBanner.named(lastChangeBanner.impact.newlyVisible)}</em>.
					{/if}
					{#if lastChangeBanner.impact.newlyHidden.length > 0}
						{i18nState.locale && m.randomizer_nowHidden()}
						<em>{lastChangeBanner.named(lastChangeBanner.impact.newlyHidden)}</em>.
					{/if}
				</div>
			</div>
		{/if}

		<div class="rdz-config-body">
			{#each groupedOptions as group (group.category)}
				{@const visibleCount = visibleOptionCount(group.options)}
				{#if visibleCount > 0}
					<section class="rdz-group">
						<button class="rdz-group-header" onclick={() => toggleCat(group.category)}>
							<Icon icon={CATEGORY_ICONS[group.category] ?? 'mdi:folder'} />
							<span class="rdz-group-name">
								{CATEGORY_LABELS[group.category] ?? group.category}
							</span>
							<span class="rdz-group-count">{visibleCount}</span>
							<Icon
								icon="mdi:chevron-down"
								class={openCategories[group.category]
									? 'rdz-chevron'
									: 'rdz-chevron rdz-chevron-collapsed'}
							/>
						</button>
						{#if openCategories[group.category]}
							<div class="rdz-group-fields">
								{#each group.options as opt (opt.id)}
									<RandomizerOptionField option={opt} />
								{/each}
							</div>
						{/if}
					</section>
				{/if}
			{/each}
			<StartInventoryEditor />
		</div>

		<footer class="rdz-config-footer">
			<Button variant="ghost" onclick={() => randomizerStore.resetToDefaults()}>
				{#snippet icon()}
					<Icon icon="mdi:restore" />
				{/snippet}
				{i18nState.locale && m.randomizer_reset()}
			</Button>
			<Button variant="secondary" onclick={() => randomizerStore.refreshGenerated()}>
				{#snippet icon()}
					<Icon icon="mdi:refresh" />
				{/snippet}
				{i18nState.locale && m.randomizer_refreshYaml()}
			</Button>
			<Button variant="primary" onclick={saveSlot}>
				{#snippet icon()}
					<Icon icon="mdi:account-plus" />
				{/snippet}
				{i18nState.locale && m.randomizer_savePlayerSlot()}
			</Button>
		</footer>
	</div>
{:else if randomizerStore.loadingSchema}
	<div class="rdz-loading">
		<Icon icon="mdi:loading" class="rdz-spin" />
		<p>{i18nState.locale && m.randomizer_loadingSchema()}</p>
	</div>
{/if}

<style>
	.rdz-config-main {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-width: 0;
		min-height: 0;
	}

	.rdz-config-header {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-md) var(--space-lg);
		border-bottom: 1px solid var(--border-subtle);
		background: var(--bg-surface);
		flex-wrap: wrap;
	}

	.rdz-back {
		width: 36px;
		height: 36px;
		border-radius: var(--radius-md);
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		color: var(--text-secondary);
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 20px;
	}

	.rdz-back:hover {
		border-color: var(--accent-400);
		color: var(--accent-400);
	}

	.rdz-config-title {
		display: flex;
		flex-direction: column;
	}

	.rdz-config-title h1 {
		margin: 0;
		font-size: 18px;
		color: var(--text-primary);
	}

	.rdz-config-title small {
		font-size: 11px;
		color: var(--text-muted);
		font-family: var(--font-mono, monospace);
	}

	.rdz-setup-link {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 6px 14px;
		border-radius: var(--radius-md);
		background: var(--accent-400);
		color: var(--text-inverse);
		font-size: 12px;
		font-weight: 600;
		text-decoration: none;
		transition: all var(--transition-fast);
		flex-shrink: 0;
	}

	.rdz-setup-link:hover {
		filter: brightness(1.15);
		box-shadow: var(--shadow-glow);
	}

	.rdz-config-controls {
		display: flex;
		gap: var(--space-md);
		margin-left: auto;
		align-items: center;
		flex-wrap: wrap;
	}

	.rdz-inline-field {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.rdz-inline-field span {
		font-size: 9px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-muted);
	}

	.rdz-inline-control {
		min-width: 0;
	}

	.rdz-inline-control-sm {
		width: 140px;
	}

	.rdz-inline-control-md {
		width: 180px;
	}

	.rdz-change-banner {
		display: flex;
		align-items: flex-start;
		gap: var(--space-sm);
		margin: var(--space-md) var(--space-lg) 0;
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-md);
		background: var(--bg-active);
		border-left: 3px solid var(--accent-400);
		color: var(--text-secondary);
		font-size: 12px;
		line-height: 1.5;
		animation: rdz-banner-in 200ms ease;
	}

	.rdz-change-banner :global(svg) {
		font-size: 16px;
		color: var(--accent-400);
		flex-shrink: 0;
		margin-top: 2px;
	}

	.rdz-change-banner strong {
		color: var(--text-primary);
	}

	.rdz-change-banner em {
		color: var(--accent-400);
		font-style: normal;
		font-weight: 600;
	}

	@keyframes rdz-banner-in {
		from {
			opacity: 0;
			transform: translateY(-4px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.rdz-config-body {
		flex: 1 1 auto;
		min-height: 0;
		overflow-y: auto;
		overflow-x: hidden;
		padding: var(--space-lg);
		padding-bottom: var(--space-3xl);
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.rdz-group {
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-lg);
		background: var(--bg-surface);
		overflow: visible;
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

	:global(.rdz-chevron) {
		font-size: 18px;
		transition: transform var(--transition-fast);
		color: var(--text-muted);
	}

	:global(.rdz-chevron-collapsed) {
		transform: rotate(-90deg);
	}

	.rdz-group-fields {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
		grid-auto-rows: auto;
		align-items: start;
		gap: var(--space-md);
		padding: var(--space-md);
		border-top: 1px solid var(--border-subtle);
		background: var(--bg-elevated);
		overflow: visible;
	}

	.rdz-config-footer {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-sm);
		padding: var(--space-md) var(--space-lg);
		border-top: 1px solid var(--border-subtle);
		background: var(--bg-surface);
	}

	.rdz-loading {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-md);
		padding: var(--space-3xl);
		color: var(--text-muted);
	}

	.rdz-loading :global(svg) {
		font-size: 40px;
	}

	.rdz-loading :global(.rdz-spin) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
