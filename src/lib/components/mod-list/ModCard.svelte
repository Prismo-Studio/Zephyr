<script lang="ts">
	import type { Mod, ModContextItem } from '$lib/types';
	import Icon from '@iconify/svelte';
	import { formatModName, modIconSrc, shortenNum, timeSince } from '$lib/util';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import type { MouseEventHandler } from 'svelte/elements';

	type Props = {
		mod: Mod;
		isSelected?: boolean;
		locked?: boolean;
		showInstallBtn?: boolean;
		onclick?: MouseEventHandler<HTMLButtonElement>;
		oninstall?: () => void;
		oncontextmenu?: (e: MouseEvent, mod: Mod) => void;
	};

	let {
		mod,
		isSelected = false,
		locked = false,
		showInstallBtn = true,
		onclick,
		oninstall,
		oncontextmenu
	}: Props = $props();

	let installing = $state(false);

	function handleContextMenu(e: MouseEvent) {
		e.preventDefault();
		e.stopPropagation();
		oncontextmenu?.(e, mod);
	}
</script>

<button
	class="z-mod-card"
	class:selected={isSelected}
	class:disabled-mod={mod.enabled === false}
	{onclick}
	oncontextmenu={handleContextMenu}
>
	<!-- Icon -->
	<div class="z-mod-icon">
		<img src={modIconSrc(mod)} alt={mod.name} />
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
			{#if mod.isPinned}
				<Icon icon="mdi:pin" class="z-mod-badge-icon pinned" />
			{/if}
			{#if mod.isDeprecated}
				<Icon icon="mdi:alert" class="z-mod-badge-icon deprecated" />
			{/if}
			{#if mod.enabled === false}
				<Badge variant="warning">Disabled</Badge>
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
				<span class="z-mod-version">{mod.version}</span>
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
	</div>

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
</button>

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
		transition: all var(--transition-fast);
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
		box-shadow: 0 0 16px rgba(26, 255, 250, 0.06);
	}

	.z-mod-card.disabled-mod {
		opacity: 0.5;
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
	}

	.z-mod-stat {
		display: flex;
		align-items: center;
		gap: 2px;
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
</style>
