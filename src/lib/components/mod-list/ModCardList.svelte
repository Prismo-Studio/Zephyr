<script lang="ts">
	import type { Mod } from '$lib/types';
	import Icon from '@iconify/svelte';
	import { formatModName, modIconSrc, shortenNum } from '$lib/util';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import CachedImage from '$lib/components/ui/CachedImage.svelte';
	import Toggle from '$lib/components/ui/Toggle.svelte';
	import ModSourceIcon from './ModSourceIcon.svelte';
	import { isModPinned } from '$lib/state/misc.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import type { MouseEventHandler } from 'svelte/elements';

	type Props = {
		mod: Mod;
		isSelected: boolean;
		locked: boolean;
		showInstallBtn: boolean;
		showDragHandle: boolean;
		dropIndicator: 'above' | 'below' | null;
		isDragging: boolean;
		installing: boolean;
		activeCategories: string[];
		onclick?: MouseEventHandler<HTMLDivElement>;
		oninstall?: () => void;
		ontoggle?: (mod: Mod) => void;
		oncontextmenu: (e: MouseEvent) => void;
		onpointerdownHandle?: (e: PointerEvent, mod: Mod) => void;
		oncategoryclick?: (category: string, multi?: boolean) => void;
		oninstallStart: () => void;
	};

	let {
		mod,
		isSelected,
		locked,
		showInstallBtn,
		showDragHandle,
		dropIndicator,
		isDragging,
		installing,
		activeCategories,
		onclick,
		oninstall,
		ontoggle,
		oncontextmenu,
		onpointerdownHandle,
		oncategoryclick,
		oninstallStart
	}: Props = $props();
</script>

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
	oncontextmenu={oncontextmenu}
	role="button"
	tabindex="0"
>
	<!-- Drag handle: full-height left strip, always visible (discoverable),
		 never animates width so the row layout never shifts on hover. -->
	{#if showDragHandle}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		{#if isModPinned(mod.uuid)}
			<div class="z-mod-drag-rail pinned-lock" aria-label="Pinned, not draggable">
				<Icon icon="mdi:pin" />
			</div>
		{:else}
			<div
				class="z-mod-drag-rail"
				onpointerdown={(e) => onpointerdownHandle?.(e, mod)}
				aria-label="Drag to reorder"
			>
				<Icon icon="mdi:drag-vertical" />
			</div>
		{/if}
	{/if}

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

	<div class="z-mod-icon">
		<CachedImage src={modIconSrc(mod)} alt={mod.name} />
		{#if mod.isInstalled}
			<span class="z-mod-installed-badge">
				<Icon icon="mdi:check" class="text-[9px]" />
			</span>
		{/if}
	</div>

	<div class="z-mod-info">
		<div class="z-mod-name-row">
			<span class="z-mod-name">{formatModName(mod.name)}</span>
			{#if isModPinned(mod.uuid)}
				<Icon icon="mdi:pin" class="z-mod-badge-icon pinned" />
			{/if}
			<ModSourceIcon uuid={mod.uuid} extraClass="z-mod-source-icon" />
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

	{#if showInstallBtn && !mod.isInstalled && !locked}
		<button
			class="z-mod-install-btn"
			disabled={installing}
			onclick={(evt) => {
				evt.stopPropagation();
				oninstallStart();
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

	.z-mod-card.dragging {
		opacity: 0.25;
		transform: scale(0.98);
	}

	.z-mod-card:has(.z-mod-drag-rail) {
		padding-left: 24px;
	}

	.z-mod-drag-rail {
		position: absolute;
		left: 0;
		top: 0;
		bottom: 0;
		width: 14px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-top-left-radius: var(--radius-lg);
		border-bottom-left-radius: var(--radius-lg);
		background-color: color-mix(in srgb, var(--text-muted) 12%, transparent);
		color: var(--text-muted);
		font-size: 18px;
		cursor: grab;
		touch-action: none;
		user-select: none;
		transition:
			background-color 150ms ease,
			color 120ms ease;
		z-index: 2;
	}

	.z-mod-drag-rail:active {
		cursor: grabbing;
	}

	.z-mod-card:hover .z-mod-drag-rail {
		background-color: color-mix(in srgb, var(--text-muted) 18%, transparent);
		color: var(--text-secondary);
	}

	.z-mod-drag-rail:hover {
		background-color: color-mix(in srgb, var(--accent-400) 25%, transparent) !important;
		color: var(--accent-400) !important;
	}

	.z-mod-drag-rail.pinned-lock {
		color: var(--accent-400);
		font-size: 12px;
		cursor: not-allowed;
		background-color: color-mix(in srgb, var(--accent-400) 10%, transparent);
	}

	.z-mod-card:hover .z-mod-drag-rail.pinned-lock {
		background-color: color-mix(in srgb, var(--accent-400) 16%, transparent);
		color: var(--accent-400);
	}

	.z-mod-drag-rail.pinned-lock:hover {
		background-color: color-mix(in srgb, var(--accent-400) 16%, transparent) !important;
	}

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

	.z-mod-toggle-wrapper {
		display: none;
		align-items: center;
		flex-shrink: 0;
	}

	.z-mod-card:hover .z-mod-toggle-wrapper {
		display: flex;
	}

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

	.z-mod-categories {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		margin-top: 3px;
	}

	:global(.z-mod-source-icon) {
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
</style>
