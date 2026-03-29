<script lang="ts">
	import type { Mod, ModContextItem } from '../../types';
	import Icon from '@iconify/svelte';
	import { modIconSrc, isOutdated, formatModName } from '$lib/util';
	import { Switch, ContextMenu } from 'bits-ui';
	import type { DragEventHandler, MouseEventHandler } from 'svelte/elements';
	import ModContextMenuContent from './ModContextMenuContent.svelte';
	import { activeContextMenu } from '$lib/context';

	type Props = {
		mod: Mod;
		index: number;
		isSelected: boolean;
		contextItems: ModContextItem[];
		reorderable: boolean;
		locked: boolean;
		ontoggle?: (newState: boolean) => void;
		onclick?: MouseEventHandler<HTMLButtonElement>;
		ondragstart?: DragEventHandler<HTMLButtonElement>;
		ondragover?: DragEventHandler<HTMLButtonElement>;
		ondragend?: DragEventHandler<HTMLButtonElement>;
	};

	let {
		mod,
		index,
		isSelected,
		contextItems,
		reorderable,
		locked,
		ontoggle,
		onclick,
		ondragstart,
		ondragover,
		ondragend
	}: Props = $props();

	let contextMenuOpen: boolean = $state(false);

	let descriptionClasses = $derived(
		mod.enabled === false
			? 'text-[#3A4A5C] line-through'
			: isSelected
				? 'text-[#8899AA]'
				: 'text-[#556677] group-hover:text-[#8899AA]'
	);

	$effect(() => {
		if ($activeContextMenu !== null && $activeContextMenu !== mod.uuid) {
			contextMenuOpen = false;
		}
	});
</script>

<ContextMenu.Root
	bind:open={contextMenuOpen}
	onOpenChange={(open) => {
		if (open) {
			$activeContextMenu = mod.uuid;
		} else {
			$activeContextMenu = null;
		}
	}}
>
	<ContextMenu.Trigger class="contents">
		<button
			class="group flex w-full items-center rounded-xl border p-2.5 transition-colors {isSelected
				? 'border-[#2D8CF0]/40 bg-[#142240]'
				: 'hover:bg-[#142240]/60 border-transparent'}"
			data-uuid={mod.uuid}
			data-index={index}
			draggable={reorderable}
			{onclick}
			{ondragstart}
			{ondragover}
			{ondragend}
		>
			<img src={modIconSrc(mod)} alt={mod.name} class="size-12 rounded-sm" />
			<div class="shrink grow overflow-hidden pr-2 pl-3 text-left">
				<div class="flex items-center gap-1 overflow-hidden">
					<div
						class={[
							mod.enabled === false ? 'text-[#556677] line-through' : 'text-[#E8ECF1]',
							'shrink truncate font-medium'
						]}
					>
						{formatModName(mod.name)}
					</div>
					<div class={[descriptionClasses, 'px-1']}>
						{mod.version ?? '?.?.?'}
					</div>
					{#if mod.isPinned}
						<Icon class="text-[#556677] shrink-0" icon="mdi:pin" />
					{/if}
					{#if mod.isDeprecated}
						<Icon class="shrink-0 text-red-500" icon="mdi:error" />
					{/if}
					{#if isOutdated(mod)}
						<Icon class="text-[#00D4AA] shrink-0" icon="mdi:arrow-up-circle" />
					{/if}
				</div>

				{#if mod.description}
					<div class="{descriptionClasses} truncate">
						{mod.description}
					</div>
				{/if}
			</div>

			{#if reorderable && !locked}
				<Icon
					icon="material-symbols:drag-indicator"
					class="text-[#556677] mr-4 shrink-0 cursor-move text-2xl"
				/>
			{/if}

			<!-- make sure click events don't propagate and cause the mod to be selected -->
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="contents" onclick={(evt) => evt.stopPropagation()}>
				<Switch.Root
					disabled={locked}
					checked={mod.enabled ?? true}
					onCheckedChange={ontoggle}
					class="group data-[state=checked]:bg-[#2D8CF0] data-[state=checked]:hover:bg-[#2D8CF0]/80 bg-[#1A2A42] hover:bg-[#1A2A42]/80 mr-1 flex h-6 w-12 shrink-0 rounded-full px-1 py-1 transition-colors"
				>
					<Switch.Thumb
						class="data-[state=checked]:bg-white bg-[#556677] pointer-events-none h-full w-4 rounded-full transition-transform duration-75 ease-out data-[state=checked]:translate-x-6"
					/>
				</Switch.Root>
			</div>
		</button>
	</ContextMenu.Trigger>
	<ModContextMenuContent type="context" style="dark" {locked} {mod} items={contextItems} />
</ContextMenu.Root>
