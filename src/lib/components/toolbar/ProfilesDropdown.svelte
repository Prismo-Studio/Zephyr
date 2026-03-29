<script lang="ts">
	import Icon from '@iconify/svelte';
	import { DropdownMenu } from 'bits-ui';
	import ProfilesDropdownItem from './ProfilesDropdownItem.svelte';
	import CreateProfileDialog from '$lib/components/dialogs/CreateProfileDialog.svelte';
	import { fade, fly } from 'svelte/transition';
	import { dropIn, dropOut } from '$lib/transitions';
	import DropdownArrow from '../ui/DropdownArrow.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { m } from '$lib/paraglide/messages';

	let open = $state(false);
	let createDialogOpen = $state(false);
</script>

<DropdownMenu.Root bind:open>
	<DropdownMenu.Trigger
		class="group border-[#1A2A42] text-[#8899AA] group-hover:text-[#E8ECF1] hover:bg-[#0F1D32] flex min-w-40 shrink items-center border-r pr-4 pl-6 transition-colors duration-150"
	>
		<Icon icon="mdi:layers-outline" class="mr-2 text-lg text-[#556677] group-hover:text-[#2D8CF0] transition-colors" />
		<span class="mr-auto shrink truncate font-semibold text-sm">
			{profiles.active?.name}
		</span>

		<div
			class="bg-[#0F1D32] group-hover:bg-[#142240] mr-2 ml-4 rounded-md px-2 py-0.5 text-xs font-semibold text-[#2D8CF0] tabular-nums"
		>
			{profiles.active?.modCount}
		</div>

		<DropdownArrow {open} />
	</DropdownMenu.Trigger>
	<DropdownMenu.Content forceMount>
		{#snippet child({ wrapperProps, props, open })}
			<div {...wrapperProps}>
				{#if open}
					<div
						{...props}
						class="border-[#1A2A42] bg-[#0F1D32] z-30 flex max-h-[80lvh] min-w-44 flex-col gap-0.5 overflow-y-auto rounded-xl border p-1.5 shadow-[0_15px_40px_rgba(0,0,0,0.5)]"
						in:fly={dropIn}
						out:fade={dropOut}
					>
						{#each profiles.list as profile, index (profile.id)}
							<ProfilesDropdownItem {profile} {index} />
						{/each}

						<DropdownMenu.Item
							class="bg-[#2D8CF0] hover:bg-[#3D9CFF] flex cursor-pointer items-center justify-center rounded-lg py-1.5 text-sm font-medium text-white transition-colors"
							onclick={() => (createDialogOpen = true)}
						>
							<Icon icon="mdi:plus" class="mr-1.5 text-lg" />
							{m.profilesDropdown_button()}
						</DropdownMenu.Item>
					</div>
				{/if}
			</div>
		{/snippet}
	</DropdownMenu.Content>
</DropdownMenu.Root>

<CreateProfileDialog bind:open={createDialogOpen} />
