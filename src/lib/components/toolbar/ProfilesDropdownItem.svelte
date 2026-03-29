<script lang="ts">
	import type { ProfileInfo } from '$lib/types';
	import Icon from '@iconify/svelte';
	import { DropdownMenu } from 'bits-ui';
	import clsx from 'clsx';
	import * as api from '$lib/api';
	import { pushInfoToast } from '$lib/toast';
	import IconButton from '$lib/components/ui/IconButton.svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import profiles from '$lib/state/profile.svelte';

	type Props = {
		index: number;
		profile: ProfileInfo;
	};

	let { index, profile }: Props = $props();

	let isActive = $derived(profile.id === profiles.active?.id);

	async function deleteProfile() {
		let confirmed = await confirm(`Are you sure you want to delete ${profile.name}?`);
		if (!confirmed) return;

		await api.profile.deleteProfile(profile.id);

		pushInfoToast({
			message: `Deleted profile ${profile.name}.`
		});
	}
</script>

<DropdownMenu.Item
	class={[
		isActive
			? 'text-[#8899AA] hover:text-[#E8ECF1] font-medium'
			: 'text-[#556677] hover:text-[#8899AA]',
		'group hover:bg-[#142240] flex cursor-default items-center rounded-sm py-1 pr-1 pl-3 text-left'
	]}
	onclick={() => profiles.setActive(index)}
>
	{#if profile.sync !== null}
		<Icon icon="mdi:cloud" class="mr-2" />
	{/if}

	<span class="mr-3 grow">
		{profile.name}
	</span>

	<Icon icon="mdi:check" class={clsx(!isActive && 'invisible', 'text-[#00D4AA] mx-2 text-lg')} />

	<div class="bg-[#142240] group-hover:bg-[#1A2A42] mr-1 rounded-sm px-1.5 py-0.5 text-xs">
		{profile.modCount}
	</div>

	<IconButton
		label="Delete profile"
		icon="mdi:delete"
		color="red"
		onclick={(evt) => {
			evt.preventDefault();
			evt.stopPropagation();
			deleteProfile();
		}}
	/>
</DropdownMenu.Item>
