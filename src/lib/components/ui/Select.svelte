<script lang="ts">
	import { dropIn, dropOut } from '$lib/transitions';
	import { emptyOrUndefined } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { Select, type WithoutChildren } from 'bits-ui';

	import type { Snippet } from 'svelte';
	import { fade, fly } from 'svelte/transition';
	import DropdownArrow from './DropdownArrow.svelte';

	type Props = WithoutChildren<Select.RootProps> & {
		placeholder?: string;
		items: { value: string; label: string; disabled?: boolean }[];
		triggerClass?: string;
		icon?: string;
		avoidCollisions?: boolean;
		item?: Snippet<[{ label: string; value: string; selected: boolean }]>;
	} & (
			| {
					label?: never;
			  }
			| {
					placeholder?: never;
					label: Snippet<[{ defaultLabel: string | null }]>;
			  }
			| {
					placeholder?: never;
					label: string;
			  }
		);

	let {
		open = $bindable(false),
		value = $bindable(),
		triggerClass,
		items,
		placeholder,
		icon,
		avoidCollisions,
		label,
		item: itemSnippet,
		...restProps
	}: Props = $props();

	const selectedLabel = $derived(
		restProps.type === 'single'
			? items.find((item) => item.value === value)?.label
			: emptyOrUndefined(
					items
						.filter((item) => value?.includes(item.value))
						.map((item) => item.label)
						.join(', ')
				)
	);
</script>

<Select.Root bind:value={value as never} bind:open {...restProps}>
	<Select.Trigger
		class={[
			triggerClass,
			'group bg-[#0B1628] enabled:hover:border-[#2D8CF0] flex items-center gap-2 overflow-hidden rounded-lg border border-[#1A2A42] py-2 pr-2 pl-3 transition-colors'
		]}
	>
		{#if icon}
			<Icon class="text-[#556677] shrink-0 text-lg" {icon} />
		{/if}

		{#if label && typeof label !== 'string'}
			{@render label({ defaultLabel: selectedLabel ?? null })}
		{:else}
			<div
				class={[
					label || selectedLabel ? 'text-[#E8ECF1]' : 'text-[#556677]',
					'group-disabled:text-[#556677] shrink grow truncate text-left'
				]}
			>
				{label ?? selectedLabel ?? placeholder}
			</div>
		{/if}

		<DropdownArrow {open} class="text-[#556677] group-disabled:text-[#3A4A5C] ml-auto" />
	</Select.Trigger>
	<Select.Portal>
		<Select.Content forceMount {avoidCollisions}>
			{#snippet child({ wrapperProps, props, open })}
				<div {...wrapperProps}>
					{#if open}
						<div
							{...props}
							class="border-[#1A2A42] bg-[#0F1D32] flex max-h-96 w-(--bits-select-anchor-width) gap-0.5 overflow-y-auto rounded-xl border p-1.5 shadow-[0_15px_40px_rgba(0,0,0,0.5)]"
							in:fly={dropIn}
							out:fade={dropOut}
						>
							<Select.Viewport>
								{#each items as item, i (i + item.value)}
									<Select.Item
										{...item}
										class="hover:bg-[#142240] hover:text-[#E8ECF1] group flex w-full cursor-default items-center rounded-lg px-3 py-1.5 transition-colors"
									>
										{#snippet children({ selected })}
											{#if itemSnippet}
												{@render itemSnippet({ ...item, selected })}
											{/if}

											<span
												class={[
													selected
														? 'text-[#E8ECF1]'
														: 'text-[#8899AA] group-hover:text-[#E8ECF1]'
												]}>{item.label}</span
											>

											{#if selected}
												<Icon icon="mdi:check" class="text-[#2D8CF0] ml-auto text-lg" />
											{/if}
										{/snippet}
									</Select.Item>
								{/each}
							</Select.Viewport>
						</div>
					{/if}
				</div>
			{/snippet}
		</Select.Content>
	</Select.Portal>
</Select.Root>
