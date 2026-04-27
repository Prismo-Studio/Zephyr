<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import Icon from '@iconify/svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import RandomizerCatalog from '$lib/features/randomizer/RandomizerCatalog.svelte';
	import RandomizerConfigurator from '$lib/features/randomizer/RandomizerConfigurator.svelte';
	import RandomizerServerPanel from '$lib/features/randomizer/RandomizerServerPanel.svelte';
	import YamlPreview from '$lib/features/randomizer/YamlPreview.svelte';
	import { randomizerStore } from '$lib/features/randomizer/randomizer.store.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { PersistedState } from 'runed';

	let rightTab: 'yaml' | 'server' = $state('server');
	let serverPanelRef: RandomizerServerPanel | undefined = $state();
	const rightPaneCollapsed = new PersistedState<boolean>('zephyr-rdz-right-collapsed', false);

	// Resizable right pane width, persisted to localStorage.
	const MIN_RIGHT = 280;
	const MAX_RIGHT = 900;
	const STORAGE_KEY = 'zephyr-rdz-right-width';
	let rightWidth = $state(420);
	let pageEl: HTMLDivElement | undefined = $state();
	let resizing = $state(false);

	async function selectGame(gameId: string) {
		await randomizerStore.selectGame(gameId);
		rightTab = 'yaml';
	}

	function back() {
		randomizerStore.clearGame();
		rightTab = 'server';
	}

	function onPlayerSaved() {
		rightTab = 'server';
		serverPanelRef?.refresh();
	}

	function clampWidth(px: number): number {
		const containerWidth = pageEl?.getBoundingClientRect().width ?? window.innerWidth;
		// Keep at least 320px for the main column.
		const hardMax = Math.min(MAX_RIGHT, Math.max(MIN_RIGHT, containerWidth - 320));
		return Math.min(hardMax, Math.max(MIN_RIGHT, px));
	}

	function onResizeMove(e: PointerEvent) {
		if (!pageEl) return;
		const rect = pageEl.getBoundingClientRect();
		// Distance from the right edge of the container = desired pane width.
		rightWidth = clampWidth(rect.right - e.clientX);
	}

	function stopResize() {
		if (!resizing) return;
		resizing = false;
		window.removeEventListener('pointermove', onResizeMove);
		window.removeEventListener('pointerup', stopResize);
		try {
			localStorage.setItem(STORAGE_KEY, String(Math.round(rightWidth)));
		} catch {
			// ignore quota errors
		}
	}

	function startResize(e: PointerEvent) {
		e.preventDefault();
		resizing = true;
		window.addEventListener('pointermove', onResizeMove);
		window.addEventListener('pointerup', stopResize);
	}

	function resetWidth(e: MouseEvent) {
		if (e.detail !== 2) return; // only on double click
		rightWidth = 420;
		try {
			localStorage.setItem(STORAGE_KEY, String(rightWidth));
		} catch {
			// ignore
		}
	}

	function onResizeKey(e: KeyboardEvent) {
		const step = e.shiftKey ? 40 : 10;
		if (e.key === 'ArrowLeft') {
			e.preventDefault();
			rightWidth = clampWidth(rightWidth + step);
		} else if (e.key === 'ArrowRight') {
			e.preventDefault();
			rightWidth = clampWidth(rightWidth - step);
		} else if (e.key === 'Home') {
			e.preventDefault();
			rightWidth = clampWidth(MAX_RIGHT);
		} else if (e.key === 'End') {
			e.preventDefault();
			rightWidth = clampWidth(MIN_RIGHT);
		} else {
			return;
		}
		try {
			localStorage.setItem(STORAGE_KEY, String(Math.round(rightWidth)));
		} catch {
			// ignore
		}
	}

	function onGlobalKeydown(e: KeyboardEvent) {
		// Ctrl+B (or Cmd+B on macOS): toggle the multiplayer/right pane.
		// Ignore when typing in editable fields so it doesn't conflict with text editing.
		if (!(e.ctrlKey || e.metaKey) || e.shiftKey || e.altKey) return;
		if (e.key.toLowerCase() !== 'b') return;
		const target = e.target as HTMLElement | null;
		const tag = target?.tagName;
		const editable = tag === 'INPUT' || tag === 'TEXTAREA' || !!target?.isContentEditable;
		if (editable) return;
		e.preventDefault();
		rightPaneCollapsed.current = !rightPaneCollapsed.current;
	}

	onMount(() => {
		window.addEventListener('rdz-player-saved', onPlayerSaved);
		window.addEventListener('keydown', onGlobalKeydown);
		try {
			const saved = parseInt(localStorage.getItem(STORAGE_KEY) ?? '', 10);
			if (!isNaN(saved)) rightWidth = clampWidth(saved);
		} catch {
			// ignore
		}
	});

	onDestroy(() => {
		window.removeEventListener('rdz-player-saved', onPlayerSaved);
		window.removeEventListener('keydown', onGlobalKeydown);
		window.removeEventListener('pointermove', onResizeMove);
		window.removeEventListener('pointerup', stopResize);
	});

	$effect(() => {
		// If the schema is cleared from elsewhere, fall back to the server tab.
		if (!randomizerStore.currentSchema && rightTab === 'yaml') {
			rightTab = 'server';
		}
	});
</script>

<div class="rdz-page" bind:this={pageEl} class:rdz-resizing={resizing}>
	<div class="rdz-main">
		{#if randomizerStore.currentSchema}
			<RandomizerConfigurator onBack={back} />
		{:else}
			<RandomizerCatalog onSelect={selectGame} />
		{/if}
	</div>

	{#if !rightPaneCollapsed.current}
		<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
		<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
		<div
			class="rdz-resize-handle"
			class:active={resizing}
			role="separator"
			aria-orientation="vertical"
			aria-label="Resize panel"
			aria-valuenow={Math.round(rightWidth)}
			aria-valuemin={MIN_RIGHT}
			aria-valuemax={MAX_RIGHT}
			tabindex="0"
			onpointerdown={startResize}
			onclick={resetWidth}
			onkeydown={onResizeKey}
		></div>
	{/if}

	<aside
		class="rdz-right-pane"
		class:collapsed={rightPaneCollapsed.current}
		style={rightPaneCollapsed.current ? 'flex-basis: 38px;' : `flex-basis: ${rightWidth}px;`}
		data-tour="rdz-right-pane"
	>
		{#if !rightPaneCollapsed.current}
			<div class="rdz-right-tabs" data-tour="rdz-right-tabs">
				{#if randomizerStore.currentSchema}
					<button
						class="rdz-tab"
						class:active={rightTab === 'yaml'}
						onclick={() => (rightTab = 'yaml')}
					>
						<Icon icon="mdi:code-braces" />
						{i18nState.locale && m.randomizer_yaml()}
					</button>
				{/if}
				<button
					class="rdz-tab"
					class:active={rightTab === 'server'}
					onclick={() => (rightTab = 'server')}
				>
					<Icon icon="mdi:server-network" />
					{i18nState.locale && m.randomizer_multiplayer()}
				</button>
			</div>
			<div class="rdz-right-body">
				{#if rightTab === 'yaml' && randomizerStore.currentSchema}
					<YamlPreview />
				{:else}
					<div class="rdz-right-scroll">
						<RandomizerServerPanel bind:this={serverPanelRef} />
					</div>
				{/if}
			</div>
		{/if}
		<div class="rdz-right-toggle-anchor" class:collapsed={rightPaneCollapsed.current}>
			<Tooltip
				text={i18nState.locale &&
					(rightPaneCollapsed.current ? m.sidebar_expand() : m.sidebar_collapse())}
				position="left"
				delay={200}
			>
				<button
					class="rdz-right-toggle"
					onclick={() => (rightPaneCollapsed.current = !rightPaneCollapsed.current)}
					aria-label={i18nState.locale &&
						(rightPaneCollapsed.current ? m.sidebar_expand() : m.sidebar_collapse())}
				>
					<Icon icon={rightPaneCollapsed.current ? 'mdi:chevron-left' : 'mdi:chevron-right'} />
				</button>
			</Tooltip>
		</div>
	</aside>
</div>

<style>
	.rdz-page {
		display: flex;
		flex-direction: row;
		flex: 1;
		min-height: 0;
		height: 100%;
		max-height: 100%;
		overflow: hidden;
		background: var(--bg-base);
		color: var(--text-primary);
	}

	.rdz-main {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-width: 0;
		min-height: 0;
		overflow: hidden;
	}

	.rdz-right-pane {
		display: flex;
		flex-direction: column;
		flex-grow: 0;
		flex-shrink: 0;
		background: var(--bg-surface);
		min-height: 0;
		max-height: 100%;
		overflow: hidden;
		position: relative;
	}

	.rdz-right-pane.collapsed {
		border-left: 1px solid var(--border-subtle);
		background: var(--bg-base);
	}

	.rdz-right-toggle-anchor {
		position: absolute;
		bottom: 10px;
		left: 6px;
		z-index: 20;
	}

	.rdz-right-toggle-anchor.collapsed {
		left: 50%;
		transform: translateX(-50%);
	}

	.rdz-right-toggle {
		width: 26px;
		height: 26px;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0;
		border-radius: var(--radius-md);
		border: 1px solid var(--accent-400);
		background: var(--accent-400);
		color: var(--text-inverse, #0a0a0a);
		cursor: pointer;
		transition:
			color var(--transition-fast),
			border-color var(--transition-fast),
			background var(--transition-fast),
			box-shadow var(--transition-fast);
		box-shadow: var(--shadow-sm), var(--shadow-glow);
	}

	.rdz-right-toggle:hover {
		background: var(--accent-500, var(--accent-400));
		border-color: var(--accent-500, var(--accent-400));
		box-shadow: var(--shadow-md), var(--shadow-glow);
	}

	.rdz-right-toggle :global(svg) {
		font-size: 14px;
	}

	.rdz-resize-handle {
		flex: 0 0 5px;
		cursor: col-resize;
		background: var(--border-subtle);
		position: relative;
		transition: background var(--transition-fast);
		touch-action: none;
	}

	.rdz-resize-handle::before {
		content: '';
		position: absolute;
		inset: 0 -3px;
	}

	.rdz-resize-handle:hover,
	.rdz-resize-handle.active {
		background: var(--accent-400);
	}

	.rdz-resize-handle:focus-visible {
		outline: 2px solid var(--accent-400);
		outline-offset: 1px;
	}

	.rdz-page.rdz-resizing {
		cursor: col-resize;
		user-select: none;
	}

	.rdz-page.rdz-resizing :global(*) {
		user-select: none !important;
	}

	.rdz-right-tabs {
		display: flex;
		gap: 0;
		background: var(--bg-elevated);
		border-bottom: 1px solid var(--border-subtle);
		flex-shrink: 0;
	}

	.rdz-tab {
		flex: 1;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 10px;
		border: none;
		background: transparent;
		color: var(--text-muted);
		font-size: 12px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		cursor: pointer;
		border-bottom: 2px solid transparent;
		transition: all var(--transition-fast);
	}

	.rdz-tab :global(svg) {
		font-size: 14px;
	}

	.rdz-tab:hover {
		color: var(--text-secondary);
	}

	.rdz-tab.active {
		color: var(--accent-400);
		border-bottom-color: var(--accent-400);
		background: var(--bg-surface);
	}

	.rdz-right-body {
		flex: 1;
		min-height: 0;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.rdz-right-scroll {
		flex: 1 1 0;
		min-height: 0;
		overflow-y: auto;
		overflow-x: hidden;
		padding: var(--space-md);
		padding-bottom: var(--space-3xl);
	}
</style>
