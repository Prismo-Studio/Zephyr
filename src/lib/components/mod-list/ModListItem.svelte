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
			class="zephyr-mod-item group"
			class:selected={isSelected}
			{onclick}
		>
			<div class="zephyr-mod-icon">
				<img src={modIconSrc(mod)} alt={mod.name} class="size-full rounded-lg object-cover" />
				{#if mod.isInstalled}
					<div class="zephyr-mod-installed">
						<Icon icon="mdi:check" class="text-[10px]" />
					</div>
				{/if}
			</div>

			<div class="shrink grow overflow-hidden pl-3 text-left">
				<div class="flex items-center gap-1.5 overflow-hidden">
					<span class="shrink truncate font-semibold text-[#E8ECF1] text-[13px] leading-tight">
						{formatModName(mod.name)}
					</span>
					{#if mod.isPinned}
						<Icon class="text-[#556677] shrink-0 text-xs" icon="mdi:pin" />
					{/if}
					{#if mod.isDeprecated}
						<Icon class="shrink-0 text-xs text-red-400" icon="mdi:error" />
					{/if}
				</div>

				{#if mod.description !== null}
					<div class="truncate text-xs leading-relaxed {isSelected ? 'text-[#8899AA]' : 'text-[#556677] group-hover:text-[#8899AA]'} mt-0.5">
						{mod.description}
					</div>
				{/if}

				{#if mod.author}
					<div class="mt-0.5 truncate text-[11px] text-[#556677]">
						{mod.author}
					</div>
				{/if}
			</div>

			{#if !mod.isInstalled && !locked}
				<!-- svelte-ignore node_invalid_placement_ssr -->
				<!-- we're not using ssr -->
				<button
					class="zephyr-install-btn"
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
						<Icon icon="mdi:download" class="text-lg" />
					{/if}
				</button>
			{/if}
		</button>
	</ContextMenu.Trigger>
	<ModContextMenuContent type="context" style="dark" {locked} {mod} items={contextItems} />
</ContextMenu.Root>

<style>
	.zephyr-mod-item {
		display: flex;
		width: 100%;
		align-items: center;
		border-radius: 12px;
		padding: 8px 10px;
		border: 1px solid transparent;
		transition: all 0.15s ease;
		position: relative;
	}

	.zephyr-mod-item:hover {
		background: rgba(20, 34, 64, 0.6);
		border-color: rgba(26, 42, 66, 0.5);
	}

	.zephyr-mod-item.selected {
		background: #142240;
		border-color: rgba(45, 140, 240, 0.35);
		box-shadow: 0 0 20px rgba(45, 140, 240, 0.06);
	}

	.zephyr-mod-icon {
		position: relative;
		width: 44px;
		height: 44px;
		flex-shrink: 0;
	}

	.zephyr-mod-installed {
		position: absolute;
		bottom: -2px;
		right: -2px;
		width: 16px;
		height: 16px;
		border-radius: 50%;
		background: #00D4AA;
		display: flex;
		align-items: center;
		justify-content: center;
		color: white;
		border: 2px solid #0F1D32;
	}

	.zephyr-install-btn {
		display: none;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		flex-shrink: 0;
		border-radius: 10px;
		background: #2D8CF0;
		color: white;
		transition: all 0.15s ease;
		margin-left: 8px;
	}

	.zephyr-install-btn:hover {
		background: #4DA3FF;
		transform: scale(1.05);
	}

	.zephyr-install-btn:disabled {
		background: #1A2A42;
		color: #556677;
		transform: none;
	}

	.zephyr-mod-item:hover .zephyr-install-btn {
		display: flex;
	}
</style>
