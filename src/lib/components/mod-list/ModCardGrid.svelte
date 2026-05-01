<script lang="ts">
	import type { Mod } from '$lib/types';
	import Icon from '@iconify/svelte';
	import { formatModName, modIconSrc, shortenNum } from '$lib/util';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import CachedImage from '$lib/components/ui/CachedImage.svelte';
	import ModSourceIcon from './ModSourceIcon.svelte';
	import { isModPinned } from '$lib/state/misc.svelte';
	import type { MouseEventHandler } from 'svelte/elements';

	type Props = {
		mod: Mod;
		isSelected: boolean;
		locked: boolean;
		showInstallBtn: boolean;
		showDragHandle: boolean;
		isDragging: boolean;
		installing: boolean;
		onclick?: MouseEventHandler<HTMLDivElement>;
		oninstall?: () => void;
		oncontextmenu: (e: MouseEvent) => void;
		onpointerdownHandle?: (e: PointerEvent, mod: Mod) => void;
		oninstallStart: () => void;
	};

	let {
		mod,
		isSelected,
		locked,
		showInstallBtn,
		showDragHandle,
		isDragging,
		installing,
		onclick,
		oninstall,
		oncontextmenu,
		onpointerdownHandle,
		oninstallStart
	}: Props = $props();

	let canDrag = $derived(showDragHandle && !isModPinned(mod.uuid));
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
	class="z-mod-grid-card"
	class:selected={isSelected}
	class:disabled-mod={mod.enabled === false}
	class:dragging={isDragging}
	class:draggable={canDrag}
	data-mod-uuid={mod.uuid}
	onpointerdown={(e) => {
		if (canDrag) onpointerdownHandle?.(e, mod);
	}}
	{onclick}
	oncontextmenu={oncontextmenu}
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
		<ModSourceIcon uuid={mod.uuid} extraClass="z-grid-source" />
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
				oninstallStart();
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

<style>
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

	:global(.z-grid-source) {
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
