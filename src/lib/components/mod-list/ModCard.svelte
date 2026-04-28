<script lang="ts">
	import type { Mod, ModContextItem } from '$lib/types';
	import Icon from '@iconify/svelte';
	import { formatModName, modIconSrc, shortenNum, timeSince } from '$lib/util';
	import Badge from '$lib/components/ui/Badge.svelte';
	import { m } from '$lib/paraglide/messages';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import type { MouseEventHandler } from 'svelte/elements';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { isModPinned, installState } from '$lib/state/misc.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import CachedImage from '$lib/components/ui/CachedImage.svelte';
	import Toggle from '$lib/components/ui/Toggle.svelte';

	type Props = {
		mod: Mod;
		isSelected?: boolean;
		locked?: boolean;
		showInstallBtn?: boolean;
		showDragHandle?: boolean;
		viewMode?: 'list' | 'grid';
		dropIndicator?: 'above' | 'below' | null;
		isDragging?: boolean;
		onclick?: MouseEventHandler<HTMLDivElement>;
		oninstall?: () => void;
		ontoggle?: (mod: Mod) => void;
		oncontextmenu?: (e: MouseEvent, mod: Mod) => void;
		onpointerdownHandle?: (e: PointerEvent, mod: Mod) => void;
		oncategoryclick?: (category: string, multi?: boolean) => void;
		activeCategories?: string[];
	};

	let {
		mod,
		isSelected = false,
		locked = false,
		showInstallBtn = true,
		showDragHandle = false,
		viewMode = 'list',
		dropIndicator = null,
		isDragging = false,
		onclick,
		oninstall,
		ontoggle,
		oncontextmenu,
		onpointerdownHandle,
		oncategoryclick,
		activeCategories = []
	}: Props = $props();

	let installing = $state(false);

	// Reset installing state when install cycle ends or mod becomes installed
	$effect(() => {
		if (!installState.active || mod.isInstalled) {
			installing = false;
		}
	});

	function handleContextMenu(e: MouseEvent) {
		e.preventDefault();
		e.stopPropagation();
		oncontextmenu?.(e, mod);
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
{#if viewMode === 'grid'}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div
		class="z-mod-grid-card"
		class:selected={isSelected}
		class:disabled-mod={mod.enabled === false}
		class:dragging={isDragging}
		class:draggable={showDragHandle && !isModPinned(mod.uuid)}
		data-mod-uuid={mod.uuid}
		onpointerdown={(e) => {
			if (showDragHandle && !isModPinned(mod.uuid)) {
				onpointerdownHandle?.(e, mod);
			}
		}}
		{onclick}
		oncontextmenu={handleContextMenu}
		role="button"
		tabindex="0"
	>
		<div class="z-grid-icon">
			<CachedImage src={modIconSrc(mod)} alt={mod.name} />
			{#if mod.isInstalled}
				<span class="z-grid-installed">
					<Icon icon="mdi:check-circle" class="text-[12px]" />
				</span>
			{/if}
			{#if mod.uuid.startsWith('curseforge:') || mod.uuid.startsWith('zephyr-server:')}
				<img src="/logos/curseforge.png" alt="CF" class="z-grid-source" />
			{:else if mod.uuid.startsWith('nexusmods:')}
				<img src="/logos/nexusmods.png" alt="NX" class="z-grid-source" />
			{:else if mod.uuid.startsWith('zephyr:')}
				<img src="/logo.png" alt="Z" class="z-grid-source" />
			{/if}
		</div>

		<div class="z-grid-body">
			<span class="z-grid-name">{formatModName(mod.name)}</span>
			{#if mod.author}
				<span class="z-grid-author">{mod.author}</span>
			{/if}
			<div class="z-grid-stats">
				{#if mod.version}
					<Tooltip text={mod.version} position="top" delay={150}>
						<span class="z-grid-version">{mod.version}</span>
					</Tooltip>
				{/if}
				{#if mod.downloads != null}
					<span class="z-grid-stat">
						<Icon icon="mdi:download" class="text-[10px]" />
						{shortenNum(mod.downloads)}
					</span>
				{/if}
				{#if mod.rating != null}
					<span class="z-grid-stat">
						<Icon icon="mdi:thumb-up" class="text-[10px]" />
						{shortenNum(mod.rating)}
					</span>
				{/if}
			</div>
		</div>

		{#if showInstallBtn && !mod.isInstalled && !locked}
			<button
				class="z-grid-install"
				disabled={installing}
				onclick={(evt) => {
					evt.stopPropagation();
					installing = true;
					oninstall?.();
				}}
			>
				{#if installing}
					<Spinner size={12} />
				{:else}
					<Icon icon="mdi:download" class="text-[12px]" />
				{/if}
			</button>
		{/if}
	</div>
{:else}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div
		class="z-mod-card"
		class:selected={isSelected}
		class:disabled-mod={mod.enabled === false}
		class:dragging={isDragging}
		class:drop-above={dropIndicator === 'above'}
		class:drop-below={dropIndicator === 'below'}
		data-mod-uuid={mod.uuid}
		{onclick}
		oncontextmenu={handleContextMenu}
		role="button"
		tabindex="0"
	>
		<!-- Drag handle -->
		{#if showDragHandle}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			{#if isModPinned(mod.uuid)}
				<div class="z-mod-drag-handle pinned-lock">
					<Icon icon="mdi:cancel" />
				</div>
			{:else}
				<div class="z-mod-drag-handle" onpointerdown={(e) => onpointerdownHandle?.(e, mod)}>
					<Icon icon="mdi:drag-vertical" />
				</div>
			{/if}
		{/if}

		<!-- Checkbox for multi-select -->
		<div class="z-mod-checkbox-wrapper">
			<Checkbox
				checked={isSelected}
				onchange={() => {
					if (!onclick) return;
					const synthEvent = new MouseEvent('click', { ctrlKey: true });
					onclick(synthEvent as any);
				}}
			/>
		</div>

		<!-- Icon -->
		<div class="z-mod-icon">
			<CachedImage src={modIconSrc(mod)} alt={mod.name} />
			{#if mod.isInstalled}
				<span class="z-mod-installed-badge">
					<Icon icon="mdi:check" class="text-[9px]" />
				</span>
			{/if}
		</div>

		<!-- Info -->
		<div class="z-mod-info">
			<div class="z-mod-name-row">
				<span class="z-mod-name">{formatModName(mod.name)}</span>
				{#if isModPinned(mod.uuid)}
					<Icon icon="mdi:pin" class="z-mod-badge-icon pinned" />
				{/if}
				{#if mod.uuid.startsWith('curseforge:') || mod.uuid.startsWith('zephyr-server:')}
					<img src="/logos/curseforge.png" alt="CF" class="z-mod-source-icon" />
				{:else if mod.uuid.startsWith('nexusmods:')}
					<img src="/logos/nexusmods.png" alt="NX" class="z-mod-source-icon" />
				{:else if mod.uuid.startsWith('zephyr:')}
					<img src="/logo.png" alt="Z" class="z-mod-source-icon" />
				{/if}
				{#if mod.isDeprecated}
					<Icon icon="mdi:alert" class="z-mod-badge-icon deprecated" />
				{/if}
				{#if mod.enabled === false}
					<Badge variant="warning">{i18nState.locale && m.modCard_disabled()}</Badge>
				{/if}
			</div>

			{#if mod.description}
				<p class="z-mod-desc">{mod.description}</p>
			{/if}

			<div class="z-mod-meta">
				{#if mod.author}
					<span class="z-mod-author">{mod.author}</span>
				{/if}
				{#if mod.version}
					<Tooltip text={mod.version} position="top" delay={150}>
						<span class="z-mod-version">{mod.version}</span>
					</Tooltip>
				{/if}
				{#if mod.downloads != null}
					<span class="z-mod-stat">
						<Icon icon="mdi:download" class="text-[10px]" />
						{shortenNum(mod.downloads)}
					</span>
				{/if}
				{#if mod.rating != null}
					<span class="z-mod-stat">
						<Icon icon="mdi:thumb-up" class="text-[10px]" />
						{shortenNum(mod.rating)}
					</span>
				{/if}
			</div>
			{#if mod.categories && mod.categories.length > 0}
				<div class="z-mod-categories">
					{#each mod.categories.slice(0, 3) as category}
						<button
							class="z-mod-category-tag"
							class:active={activeCategories.includes(category)}
							onclick={(e) => {
								e.stopPropagation();
								oncategoryclick?.(category, e.ctrlKey || e.metaKey);
							}}
						>
							{category}
						</button>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Enable/disable toggle -->
		{#if mod.isInstalled && ontoggle}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="z-mod-toggle-wrapper" onclick={(e) => e.stopPropagation()}>
				<Toggle
					checked={mod.enabled !== false}
					disabled={locked}
					onchange={() => ontoggle?.(mod)}
				/>
			</div>
		{/if}

		<!-- Install button -->
		{#if showInstallBtn && !mod.isInstalled && !locked}
			<button
				class="z-mod-install-btn"
				disabled={installing}
				onclick={(evt) => {
					evt.stopPropagation();
					installing = true;
					oninstall?.();
				}}
			>
				{#if installing}
					<Spinner size={14} />
				{:else}
					<Icon icon="mdi:download" class="text-base" />
				{/if}
			</button>
		{/if}
	</div>
{/if}

<style>
	.z-mod-card {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		width: 100%;
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-lg);
		border: 1px solid transparent;
		background: transparent;
		cursor: pointer;
		transition: all 120ms ease;
		text-align: left;
		position: relative;
		font-family: var(--font-body);
	}

	.z-mod-card:hover {
		background: var(--bg-hover);
		border-color: var(--border-subtle);
	}

	.z-mod-card.selected {
		background: var(--bg-active);
		border-color: var(--border-accent);
		box-shadow: var(--shadow-glow);
	}

	.z-mod-card.disabled-mod {
		opacity: 0.5;
	}

	/* Drag states */
	.z-mod-card.dragging {
		opacity: 0.25;
		transform: scale(0.98);
	}

	/* Drag handle */
	.z-mod-drag-handle {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 0;
		overflow: hidden;
		flex-shrink: 0;
		color: var(--text-muted);
		cursor: grab;
		opacity: 0;
		transition:
			width 150ms ease,
			opacity 150ms ease,
			color 120ms ease;
		font-size: 24px;
		touch-action: none;
		user-select: none;
	}

	.z-mod-drag-handle:active {
		cursor: grabbing;
	}

	.z-mod-card:hover .z-mod-drag-handle {
		width: 24px;
		opacity: 0.5;
	}

	.z-mod-drag-handle:hover {
		opacity: 1 !important;
		color: var(--text-accent);
	}

	.z-mod-drag-handle.pinned-lock {
		cursor: not-allowed;
		opacity: 0;
		color: var(--error);
	}

	.z-mod-card:hover .z-mod-drag-handle.pinned-lock {
		width: 24px;
		opacity: 0.4;
	}

	.z-mod-drag-handle.pinned-lock:hover {
		opacity: 0.7 !important;
		color: var(--error);
	}

	/* Checkbox */
	.z-mod-checkbox-wrapper {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		flex-shrink: 0;
		opacity: 0;
		transition: opacity 120ms ease;
	}

	.z-mod-card:hover .z-mod-checkbox-wrapper,
	.z-mod-card.selected .z-mod-checkbox-wrapper {
		opacity: 1;
	}

	/* Icon */
	.z-mod-icon {
		position: relative;
		width: 44px;
		height: 44px;
		flex-shrink: 0;
		border-radius: var(--radius-md);
		overflow: hidden;
		background: var(--bg-overlay);
	}

	.z-mod-icon img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.z-mod-installed-badge {
		position: absolute;
		bottom: -2px;
		right: -2px;
		width: 16px;
		height: 16px;
		border-radius: 50%;
		background: var(--success);
		display: flex;
		align-items: center;
		justify-content: center;
		color: white;
		border: 2px solid var(--bg-base);
	}

	/* Info */
	.z-mod-info {
		flex: 1;
		min-width: 0;
		overflow: hidden;
	}

	.z-mod-name-row {
		display: flex;
		align-items: center;
		gap: 6px;
		overflow: hidden;
	}

	.z-mod-name {
		font-weight: 600;
		font-size: 13px;
		color: var(--text-primary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	:global(.z-mod-badge-icon) {
		flex-shrink: 0;
		font-size: 12px;
	}
	:global(.z-mod-badge-icon.pinned) {
		color: var(--text-muted);
	}
	:global(.z-mod-badge-icon.deprecated) {
		color: var(--error);
	}

	.z-mod-desc {
		font-size: 12px;
		color: var(--text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		margin-top: 2px;
		line-height: 1.4;
	}

	.z-mod-card:hover .z-mod-desc,
	.z-mod-card.selected .z-mod-desc {
		color: var(--text-secondary);
	}

	.z-mod-meta {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		margin-top: 3px;
		font-size: 11px;
		color: var(--text-muted);
	}

	.z-mod-author {
		font-weight: 500;
	}

	.z-mod-version {
		font-family: var(--font-mono);
		font-size: 10px;
		max-width: 140px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		display: inline-block;
		vertical-align: middle;
	}

	.z-mod-stat {
		display: flex;
		align-items: center;
		gap: 2px;
	}

	/* Enable/disable toggle */
	.z-mod-toggle-wrapper {
		display: none;
		align-items: center;
		flex-shrink: 0;
	}

	.z-mod-card:hover .z-mod-toggle-wrapper {
		display: flex;
	}

	/* Install button */
	.z-mod-install-btn {
		display: none;
		align-items: center;
		justify-content: center;
		width: 34px;
		height: 34px;
		flex-shrink: 0;
		border-radius: var(--radius-md);
		background: var(--accent-400);
		color: var(--text-inverse);
		border: none;
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.z-mod-install-btn:hover {
		background: var(--accent-300);
		box-shadow: var(--shadow-glow);
		transform: scale(1.05);
	}

	.z-mod-install-btn:disabled {
		background: var(--bg-overlay);
		color: var(--text-muted);
		transform: none;
		box-shadow: none;
	}

	.z-mod-card:hover .z-mod-install-btn {
		display: flex;
	}

	/* Category tags */
	.z-mod-categories {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		margin-top: 3px;
	}

	.z-mod-source-icon {
		width: 14px;
		height: 14px;
		border-radius: 3px;
		object-fit: contain;
		flex-shrink: 0;
	}

	.z-mod-category-tag {
		display: inline-flex;
		align-items: center;
		padding: 1px 6px;
		border-radius: var(--radius-full);
		font-size: 10px;
		font-weight: 500;
		background: var(--bg-overlay);
		color: var(--text-muted);
		border: 1px solid var(--border-subtle);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.z-mod-category-tag:hover {
		color: var(--text-accent);
		border-color: var(--accent-400, var(--text-accent));
	}

	.z-mod-category-tag.active {
		color: var(--text-accent);
		border-color: var(--accent-400, var(--text-accent));
	}

	/* ── Grid Card ── */
	.z-mod-grid-card {
		display: flex;
		flex-direction: column;
		border-radius: var(--radius-lg);
		border: 1px solid var(--border-subtle);
		background: var(--bg-elevated);
		cursor: pointer;
		transition: all 150ms ease;
		overflow: hidden;
		position: relative;
	}

	.z-mod-grid-card:hover {
		border-color: var(--border-default);
		transform: translateY(-2px);
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
	}

	.z-mod-grid-card.selected {
		border-color: var(--accent-400);
		border-width: 2px;
		box-shadow: var(--shadow-glow-strong);
	}

	.z-mod-grid-card.disabled-mod {
		opacity: 0.5;
	}

	.z-grid-icon {
		position: relative;
		width: 100%;
		aspect-ratio: 1;
		background: var(--bg-overlay);
		overflow: hidden;
	}

	.z-grid-icon > img:not(.z-grid-source) {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.z-grid-installed {
		position: absolute;
		top: 8px;
		right: 8px;
		width: 24px;
		height: 24px;
		border-radius: 50%;
		background: var(--success);
		display: flex;
		align-items: center;
		justify-content: center;
		color: white;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
	}

	.z-grid-source {
		position: absolute;
		top: 8px;
		left: 8px;
		width: 20px;
		height: 20px;
		border-radius: 4px;
		object-fit: contain;
		background: rgba(0, 0, 0, 0.5);
		padding: 2px;
	}

	.z-grid-body {
		padding: 10px 12px;
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-height: 0;
	}

	.z-grid-name {
		font-weight: 700;
		font-size: 13px;
		color: var(--text-primary);
		line-height: 1.3;
		word-break: break-word;
	}

	.z-grid-author {
		font-size: 11px;
		color: var(--text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.z-grid-stats {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 8px;
		margin-top: 4px;
		font-size: 11px;
		color: var(--text-muted);
		min-width: 0;
		max-width: 100%;
	}

	.z-grid-stats :global(.z-tooltip-trigger) {
		min-width: 0;
		max-width: 100%;
	}

	.z-grid-version {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-accent);
		background: var(--bg-active);
		padding: 1px 6px;
		border-radius: var(--radius-full);
		max-width: 100px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		display: inline-block;
		min-width: 0;
	}

	.z-grid-stat {
		display: flex;
		align-items: center;
		gap: 2px;
	}

	.z-grid-install {
		position: absolute;
		bottom: 8px;
		right: 8px;
		width: 28px;
		height: 28px;
		border-radius: 50%;
		background: var(--accent-400);
		color: var(--text-inverse);
		border: none;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		opacity: 0;
		transition: all 150ms ease;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
	}

	.z-mod-grid-card:hover .z-grid-install {
		opacity: 1;
	}

	.z-grid-install:hover {
		background: var(--accent-300);
		transform: scale(1.1);
	}

	.z-grid-install:disabled {
		background: var(--bg-overlay);
		color: var(--text-muted);
	}

	/* Grid card draggable state */
	.z-mod-grid-card.draggable {
		cursor: grab;
		touch-action: none;
		user-select: none;
	}

	.z-mod-grid-card.draggable:active {
		cursor: grabbing;
	}

	.z-mod-grid-card.dragging {
		opacity: 0.25;
		transform: scale(0.95);
	}
</style>
