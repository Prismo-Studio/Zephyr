<script lang="ts">
	import type { Mod } from '$lib/types';
	import Icon from '@iconify/svelte';
	import {
		formatModName,
		modIconSrc,
		shortenNum,
		shortenFileSize,
		timeSince
	} from '$lib/util';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Tabs from '$lib/components/ui/Tabs.svelte';
	import CachedImage from '$lib/components/ui/CachedImage.svelte';
	import { togglePin, isModPinned } from '$lib/state/misc.svelte';
	import { loadModMarkdown } from '$lib/utils/loadModMarkdown';
	import ModDependencyList from './ModDependencyList.svelte';
	import ModVersionSelector from './ModVersionSelector.svelte';
	import ModDetailsActions from './ModDetailsActions.svelte';
	import ModExternalLinks from './ModExternalLinks.svelte';
	import ModMarkdownView from './ModMarkdownView.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import type { Snippet } from 'svelte';

	type Props = {
		mod: Mod;
		locked?: boolean;
		showVersionSelector?: boolean;
		onclose?: () => void;
		ontoggle?: () => void;
		onremove?: () => void;
		oncategoryclick?: (category: string, multi?: boolean) => void;
		ondepclick?: (author: string, name: string) => Promise<boolean> | boolean;
		activeCategories?: string[];
		children?: Snippet;
	};

	let {
		mod,
		locked = false,
		showVersionSelector = true,
		onclose,
		ontoggle,
		onremove,
		oncategoryclick,
		ondepclick,
		activeCategories = [],
		children
	}: Props = $props();

	let activeTab = $state('readme');
	let markdown = $state('');
	let loadingMarkdown = $state(false);
	let bodyEl: HTMLDivElement;

	function isExternal(): boolean {
		return mod.uuid.includes(':');
	}

	function scrollBodyToTop() {
		if (bodyEl) bodyEl.scrollTop = 0;
	}

	let tabs = $derived([
		{ id: 'readme', label: i18nState.locale && m.modpack_readme_title() },
		{ id: 'changelog', label: i18nState.locale && m.modpack_changeLog_title() },
		{
			id: 'dependencies',
			label: i18nState.locale && m.modDetails_dependencies(),
			title: `${mod.dependencies?.length ?? 0} ${i18nState.locale && m.modDetails_dependencies()}`
		}
	]);

	async function loadFor(type: 'readme' | 'changelog') {
		loadingMarkdown = true;
		markdown = await loadModMarkdown(mod, type);
		loadingMarkdown = false;
	}

	$effect(() => {
		if (mod) {
			scrollBodyToTop();
			if (activeTab !== 'dependencies') {
				loadFor(activeTab as 'readme' | 'changelog');
			}
		}
	});
</script>

<div class="z-mod-details">
	<div class="z-details-header">
		<button class="z-details-close" onclick={onclose}>
			<Icon icon="mdi:close" />
		</button>

		<div class="z-details-hero">
			<CachedImage src={modIconSrc(mod)} alt={mod.name} class="z-details-icon" />
			<div class="z-details-title">
				<div class="z-details-name-row">
					<h2>{formatModName(mod.name)}</h2>
					{#if mod.isInstalled && !isExternal()}
						<button
							class="z-pin-toggle"
							class:pinned={isModPinned(mod.uuid)}
							onclick={() => togglePin(mod.uuid)}
							title={isModPinned(mod.uuid) ? 'Désépingler' : 'Épingler'}
						>
							<Icon icon={isModPinned(mod.uuid) ? 'mdi:pin' : 'mdi:pin-outline'} />
						</button>
					{/if}
				</div>
				{#if mod.author}
					<span class="z-details-author">
						{i18nState.locale && m.modDetails_by()} {mod.author}
					</span>
				{/if}
			</div>
		</div>

		<div class="z-details-badges">
			{#if mod.version && mod.versions.length > 1 && mod.isInstalled && showVersionSelector && !isExternal()}
				<ModVersionSelector {mod} />
			{:else if mod.version}
				<Badge variant="accent">{mod.version}</Badge>
			{/if}
			{#if mod.isInstalled}
				<Badge variant="success">{i18nState.locale && m.modDetails_installed()}</Badge>
			{/if}
			{#if mod.isDeprecated}
				<Badge variant="error">{i18nState.locale && m.modDetails_deprecated()}</Badge>
			{/if}
		</div>

		<div class="z-details-stats">
			{#if mod.downloads != null}
				<div class="z-stat">
					<Icon icon="mdi:download" />
					<span>{shortenNum(mod.downloads)}</span>
				</div>
			{/if}
			{#if mod.rating != null}
				<div class="z-stat">
					<Icon icon="mdi:thumb-up" />
					<span>{shortenNum(mod.rating)}</span>
				</div>
			{/if}
			<div class="z-stat">
				<Icon icon="mdi:file" />
				<span>{shortenFileSize(mod.fileSize)}</span>
			</div>
			{#if mod.lastUpdated}
				<div class="z-stat">
					<Icon icon="mdi:clock-outline" />
					<span>{timeSince(mod.lastUpdated)}</span>
				</div>
			{/if}
		</div>

		{#if mod.categories && mod.categories.length > 0}
			<div class="z-details-categories">
				{#each mod.categories as category}
					<button
						class="z-category-tag"
						class:active={activeCategories.includes(category)}
						onclick={(e) => oncategoryclick?.(category, e.ctrlKey || e.metaKey)}
					>
						<Icon icon="mdi:tag" class="text-[10px]" />
						{category}
					</button>
				{/each}
			</div>
		{/if}

		{#if mod.isInstalled && !isExternal()}
			<ModDetailsActions {mod} {locked} {ontoggle} {onremove} />
		{/if}

		{#if children}{@render children()}{/if}

		<ModExternalLinks {mod} />
	</div>

	<div class="z-details-content">
		<div class="z-details-tabs-row">
			<Tabs
				{tabs}
				bind:active={activeTab}
				onchange={(id) => {
					scrollBodyToTop();
					if (id !== 'dependencies') loadFor(id as 'readme' | 'changelog');
				}}
			/>
		</div>

		<div class="z-details-body" bind:this={bodyEl}>
			{#if activeTab === 'dependencies'}
				<ModDependencyList
					dependencies={mod.dependencies}
					ondepclick={async (author, name) => {
						const found = await ondepclick?.(author, name);
						if (found) activeTab = 'readme';
						return found ?? false;
					}}
				/>
			{:else}
				<ModMarkdownView {markdown} loading={loadingMarkdown} />
			{/if}
		</div>
	</div>
</div>

<style>
	.z-mod-details {
		width: 40%;
		min-width: 320px;
		max-width: 480px;
		height: 100%;
		background: var(--bg-surface);
		border-left: 1px solid var(--border-subtle);
		display: flex;
		flex-direction: column;
		overflow: visible;
		animation: slideIn var(--transition-normal) ease;
		container-type: inline-size;
	}

	@keyframes slideIn {
		from {
			transform: translateX(20px);
			opacity: 0;
		}
		to {
			transform: translateX(0);
			opacity: 1;
		}
	}

	.z-details-header {
		padding: var(--space-xl);
		border-bottom: 1px solid var(--border-subtle);
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		position: relative;
		overflow: visible;
		z-index: 1;
	}

	.z-details-close {
		position: absolute;
		top: var(--space-md);
		right: var(--space-md);
		width: 28px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.z-details-close:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.z-details-hero {
		display: flex;
		align-items: center;
		gap: var(--space-lg);
	}

	:global(.z-details-icon) {
		width: 64px;
		height: 64px;
		border-radius: var(--radius-lg);
		object-fit: cover;
		background: var(--bg-overlay);
		border: 1px solid var(--border-subtle);
		flex-shrink: 0;
	}

	.z-details-name-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.z-details-title h2 {
		font-family: var(--font-display);
		font-size: 18px;
		font-weight: 800;
		color: var(--text-primary);
		letter-spacing: -0.02em;
	}

	.z-pin-toggle {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
		flex-shrink: 0;
		font-size: 16px;
	}

	.z-pin-toggle:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.z-pin-toggle.pinned {
		color: var(--text-accent);
	}

	.z-pin-toggle.pinned:hover {
		color: var(--error);
		background: rgba(255, 92, 92, 0.1);
	}

	.z-details-author {
		font-size: 13px;
		color: var(--text-muted);
	}

	.z-details-badges {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.z-details-stats {
		display: flex;
		gap: var(--space-lg);
		font-size: 12px;
		color: var(--text-muted);
	}

	.z-stat {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.z-details-categories {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.z-category-tag {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 10px;
		border-radius: var(--radius-full);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.02em;
		background: var(--bg-overlay);
		color: var(--text-secondary);
		border: 1px solid var(--border-subtle);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.z-category-tag:hover {
		color: var(--text-accent);
		border-color: var(--accent-400, var(--text-accent));
	}

	.z-category-tag.active {
		color: var(--text-accent);
		border-color: var(--accent-400, var(--text-accent));
	}

	.z-details-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		padding: var(--space-lg);
		gap: var(--space-md);
	}

	.z-details-tabs-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-md);
	}

	.z-details-body {
		flex: 1;
		overflow-y: auto;
		padding-top: var(--space-sm);
	}
</style>
