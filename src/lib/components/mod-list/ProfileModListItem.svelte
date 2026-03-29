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

	let disabled = $derived(mod.enabled === false);

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
			class="zephyr-profile-mod group"
			class:selected={isSelected}
			class:disabled
			data-uuid={mod.uuid}
			data-index={index}
			draggable={reorderable}
			{onclick}
			{ondragstart}
			{ondragover}
			{ondragend}
		>
			<div class="zephyr-mod-icon" class:disabled>
				<img src={modIconSrc(mod)} alt={mod.name} class="size-full rounded-lg object-cover" />
				{#if isOutdated(mod)}
					<div class="zephyr-mod-update">
						<Icon icon="mdi:arrow-up" class="text-[10px]" />
					</div>
				{/if}
			</div>

			<div class="shrink grow overflow-hidden pr-2 pl-3 text-left">
				<div class="flex items-center gap-1.5 overflow-hidden">
					<span
						class="shrink truncate font-semibold text-[13px] leading-tight"
						class:text-[#556677]={disabled}
						class:line-through={disabled}
						class:text-[#E8ECF1]={!disabled}
					>
						{formatModName(mod.name)}
					</span>

					<span class="text-[11px] tabular-nums {disabled ? 'text-[#3A4A5C]' : 'text-[#556677]'}">
						{mod.version ?? '?.?.?'}
					</span>

					{#if mod.isPinned}
						<Icon class="text-[#556677] shrink-0 text-xs" icon="mdi:pin" />
					{/if}
					{#if mod.isDeprecated}
						<Icon class="shrink-0 text-xs text-red-400" icon="mdi:error" />
					{/if}
				</div>

				{#if mod.description}
					<div class="mt-0.5 truncate text-xs leading-relaxed {disabled ? 'text-[#3A4A5C] line-through' : isSelected ? 'text-[#8899AA]' : 'text-[#556677] group-hover:text-[#8899AA]'}">
						{mod.description}
					</div>
				{/if}
			</div>

			{#if reorderable && !locked}
				<Icon
					icon="material-symbols:drag-indicator"
					class="text-[#556677] mr-3 shrink-0 cursor-move text-xl opacity-0 transition-opacity group-hover:opacity-100"
				/>
			{/if}

			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="contents" onclick={(evt) => evt.stopPropagation()}>
				<Switch.Root
					disabled={locked}
					checked={mod.enabled ?? true}
					onCheckedChange={ontoggle}
					class="zephyr-toggle"
				>
					<Switch.Thumb class="zephyr-toggle-thumb" />
				</Switch.Root>
			</div>
		</button>
	</ContextMenu.Trigger>
	<ModContextMenuContent type="context" style="dark" {locked} {mod} items={contextItems} />
</ContextMenu.Root>

<style>
	.zephyr-profile-mod {
		display: flex;
		width: 100%;
		align-items: center;
		border-radius: 12px;
		padding: 8px 10px;
		border: 1px solid transparent;
		transition: all 0.15s ease;
	}

	.zephyr-profile-mod:hover {
		background: rgba(20, 34, 64, 0.6);
		border-color: rgba(26, 42, 66, 0.5);
	}

	.zephyr-profile-mod.selected {
		background: #142240;
		border-color: rgba(45, 140, 240, 0.35);
		box-shadow: 0 0 20px rgba(45, 140, 240, 0.06);
	}

	.zephyr-mod-icon {
		position: relative;
		width: 44px;
		height: 44px;
		flex-shrink: 0;
		transition: opacity 0.15s ease;
	}

	.zephyr-mod-icon.disabled {
		opacity: 0.4;
	}

	.zephyr-mod-update {
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
		animation: pulse-update 2s ease-in-out infinite;
	}

	@keyframes pulse-update {
		0%, 100% { box-shadow: 0 0 0 0 rgba(0, 212, 170, 0.3); }
		50% { box-shadow: 0 0 0 4px rgba(0, 212, 170, 0); }
	}

	:global(.zephyr-toggle) {
		position: relative;
		width: 40px;
		height: 22px;
		flex-shrink: 0;
		border-radius: 11px;
		background: #1A2A42;
		padding: 3px;
		transition: background 0.2s ease;
		margin-right: 4px;
	}

	:global(.zephyr-toggle[data-state="checked"]) {
		background: #2D8CF0;
	}

	:global(.zephyr-toggle:hover) {
		background: #1E3050;
	}

	:global(.zephyr-toggle[data-state="checked"]:hover) {
		background: #4DA3FF;
	}

	:global(.zephyr-toggle-thumb) {
		display: block;
		width: 16px;
		height: 16px;
		border-radius: 50%;
		background: #556677;
		transition: all 0.15s ease;
		pointer-events: none;
	}

	:global(.zephyr-toggle-thumb[data-state="checked"]) {
		background: white;
		transform: translateX(18px);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
	}
</style>
