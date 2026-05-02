<script lang="ts">
	import type { Mod } from '$lib/types';
	import type { MouseEventHandler } from 'svelte/elements';
	import { installState } from '$lib/state/misc.svelte';
	import ModCardGrid from './ModCardGrid.svelte';
	import ModCardList from './ModCardList.svelte';

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

	// Reset the local installing flag when the global install cycle ends OR
	// the mod becomes installed (covers both success and external completion).
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

	function startInstalling() {
		installing = true;
	}
</script>

{#if viewMode === 'grid'}
	<ModCardGrid
		{mod}
		{isSelected}
		{locked}
		{showInstallBtn}
		{showDragHandle}
		{isDragging}
		{installing}
		{onclick}
		{oninstall}
		oncontextmenu={handleContextMenu}
		{onpointerdownHandle}
		oninstallStart={startInstalling}
	/>
{:else}
	<ModCardList
		{mod}
		{isSelected}
		{locked}
		{showInstallBtn}
		{showDragHandle}
		{dropIndicator}
		{isDragging}
		{installing}
		{activeCategories}
		{onclick}
		{oninstall}
		{ontoggle}
		oncontextmenu={handleContextMenu}
		{onpointerdownHandle}
		{oncategoryclick}
		oninstallStart={startInstalling}
	/>
{/if}
