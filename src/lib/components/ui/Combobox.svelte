<script lang="ts">
	import { dropIn, dropOut } from '$lib/transitions';
	import Icon from '@iconify/svelte';
	import { Combobox, type WithoutChildrenOrChild, mergeProps } from 'bits-ui';
	import type { ClassValue } from 'clsx';
	import { fade, fly } from 'svelte/transition';
	import DropdownArrow from './DropdownArrow.svelte';

	type Props = Combobox.RootProps & {
		triggerClass?: ClassValue;
		placeholder?: string;
	};

	let {
		items = [],
		value = $bindable(),
		open = $bindable(false),
		triggerClass,
		placeholder,
		type,
		...restProps
	}: Props = $props();

	let searchValue = $state('');

	const filteredItems = $derived.by(() => {
		if (searchValue === '') return items;
		return items.filter((item) => item.label.toLowerCase().includes(searchValue.toLowerCase()));
	});

	function handleInput(evt: Event & { currentTarget: HTMLInputElement }) {
		searchValue = evt.currentTarget.value;
	}

	function handleOpenChange(newOpen: boolean) {
		if (!newOpen) searchValue = '';
	}

	const mergedRootProps = $derived(mergeProps(restProps, { onOpenChange: handleOpenChange }));
</script>

<Combobox.Root {type} {items} bind:value={value as never} bind:open {...mergedRootProps}>
	<div
		class={[
			triggerClass,
			'group bg-[#0B1628] hover:border-[#1A2A42] focus-within:border-[#1A2A42] group flex items-center gap-2 overflow-hidden rounded-lg border border-transparent pr-2 disabled:cursor-not-allowed'
		]}
	>
		<Combobox.Input
			{placeholder}
			oninput={handleInput}
			clearOnDeselect
			class="placeholder-[#556677] text-[#8899AA] h-full w-full py-1.5 pl-3 focus:outline-0"
		/>
		<Combobox.Trigger><DropdownArrow class="text-[#556677]" {open} /></Combobox.Trigger>
	</div>
	<Combobox.Portal>
		<Combobox.Content forceMount>
			{#snippet child({ wrapperProps, props, open })}
				<div {...wrapperProps}>
					{#if open}
						<div
							{...props}
							class="border-[#1A2A42] bg-[#0F1D32] flex max-h-96 w-[var(--bits-combobox-anchor-width)] gap-0.5 overflow-y-auto rounded-lg border p-1 shadow-xl"
							in:fly={dropIn}
							out:fade={dropOut}
						>
							{#each filteredItems as item, i (i + item.value)}
								<Combobox.Item
									{...item}
									class="hover:bg-[#142240] hover:text-[#E8ECF1] group flex w-full cursor-default items-center rounded-md px-3 py-1"
								>
									{#snippet children({ selected, highlighted })}
										<span
											class={[
												selected || highlighted
													? 'text-[#8899AA]'
													: 'text-[#556677] group-hover:text-[#8899AA]'
											]}>{item.label}</span
										>

										{#if selected}
											<Icon icon="mdi:check" class="text-[#2D8CF0] ml-auto text-lg" />
										{/if}
									{/snippet}
								</Combobox.Item>
							{:else}
								<span class="w-full text-center text-[#556677] py-1"> No results found </span>
							{/each}
						</div>
					{/if}
				</div>
			{/snippet}
		</Combobox.Content>
	</Combobox.Portal>
</Combobox.Root>
