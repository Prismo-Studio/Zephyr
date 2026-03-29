<script lang="ts">
	import type { Mod, ModContextItem } from '../../types';
	import Icon from '@iconify/svelte';
	import { formatModName, modIconSrc } from '$lib/util';
	import type { MouseEventHandler } from 'svelte/elements';
	import { ContextMenu } from 'bits-ui';
	import { activeContextMenu } from '$lib/context';
	import ModContextMenuContent from './ModContextMenuContent.svelte';
	import Spinner from '../ui/Spinner.svelte';

	type Props = {
		mod: Mod;
		isSelected: boolean;
		locked: boolean;
		contextItems: ModContextItem[];
		onclick?: MouseEventHandler<HTMLButtonElement>;
		oninstall?: () => void;
	};

	let { mod, isSelected, locked, contextItems, onclick, oninstall }: Props = $props();

	let contextMenuOpen = $state(false);
	let loading = $state(false);

	let descriptionClasses = $derived(
		isSelected ? 'text-[#8899AA]' : 'text-[#556677] group-hover:text-[#8899AA]'
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
			class="group flex w-full rounded-xl border p-2.5 transition-colors {isSelected
				? 'border-[#2D8CF0]/40 bg-[#142240]'
				: 'hover:bg-[#142240]/60 border-transparent'}"
			{onclick}
		>
			<img src={modIconSrc(mod)} alt={mod.name} class="size-12 rounded-sm" />
			<div class="shrink grow overflow-hidden pl-3 text-left">
				<div class="flex items-center gap-1 overflow-hidden">
					<div class="shrink truncate pr-1 font-medium text-white">
						{formatModName(mod.name)}
					</div>
					{#if mod.isPinned}
						<Icon class="text-[#556677] shrink-0" icon="mdi:pin" />
					{/if}
					{#if mod.isDeprecated}
						<Icon class="shrink-0 text-red-500" icon="mdi:error" />
					{/if}
					{#if mod.isInstalled}
						<Icon class="text-[#00D4AA] shrink-0" icon="mdi:check-circle" />
					{/if}
				</div>

				{#if mod.description !== null}
					<div class="truncate {descriptionClasses}">
						{mod.description}
					</div>
				{/if}
			</div>

			{#if !mod.isInstalled && !locked}
				<!-- svelte-ignore node_invalid_placement_ssr -->
				<!-- we're not using ssr -->
				<button
					class="bg-[#2D8CF0] hover:bg-[#2D8CF0]/80 disabled:bg-[#1A2A42] disabled:text-[#556677] mt-0.5 mr-0.5 ml-2 hidden rounded-lg p-2.5 align-middle text-2xl text-white group-hover:inline transition-colors"
					disabled={loading}
					onclick={(evt) => {
						evt.stopPropagation();
						oninstall?.();
						loading = true;
					}}
				>
					{#if loading}
						<Spinner />
					{:else}
						<Icon icon="mdi:download" />
					{/if}
				</button>
			{/if}
		</button>
	</ContextMenu.Trigger>
	<ModContextMenuContent type="context" style="dark" {locked} {mod} items={contextItems} />
</ContextMenu.Root>
