<script lang="ts">
	import ContextMenuContent from '$lib/components/ui/ContextMenuContent.svelte';
	import * as api from '$lib/api';
	import type { ContextItem, Mod, ModId } from '$lib/types';
	import { shortenFileSize } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { DropdownMenu } from 'bits-ui';
	import clsx from 'clsx';
	import DropdownArrow from '$lib/components/ui/DropdownArrow.svelte';
	import Spinner from '../ui/Spinner.svelte';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		mod: Mod;
		locked: boolean;
		install: (mod: ModId) => void;
	};

	let { mod, locked, install }: Props = $props();

	let versionsOpen = $state(false);
	let downloadSize: number | null = $state(null);

	let loading = $state(false);

	let disabled = $derived(mod.isInstalled || locked || loading);

	let modId = $derived({
		packageUuid: mod.uuid,
		versionUuid: mod.versionUuid
	});

	let contextItems: ContextItem[] = $derived(
		mod.versions.map((version) => ({
			label: version.name,
			onclick: () =>
				install({
					packageUuid: mod.uuid,
					versionUuid: version.uuid
				})
		}))
	);

	$effect(() => {
		loading = false;
		api.profile.install.getDownloadSize(modId).then((size) => (downloadSize = size));
	});
</script>

<div class="mt-2 flex text-base text-white">
	<button
		class="zephyr-install-main enabled:hover:shadow-[0_0_20px_rgba(45,140,240,0.3)] flex grow items-center justify-center gap-2 rounded-l-xl py-2.5 font-semibold transition-all duration-200 disabled:cursor-not-allowed"
		class:installed={mod.isInstalled}
		class:locked
		onclick={() => {
			install(modId);
			loading = true;
		}}
		{disabled}
	>
		{#if locked}
			<Icon icon="mdi:lock" class="text-lg" />
			{m.installModButton_button_locked()}
		{:else if mod.isInstalled}
			<Icon icon="mdi:check-circle" class="text-lg text-[#00D4AA]" />
			{m.installModButton_button_isInstalled()}
		{:else if loading}
			<Spinner />
			{m.installModButton_button_loading()}
		{:else}
			<Icon icon="mdi:download" class="align-middle text-xl" />
			{m.installModButton_button_install()}

			{#if downloadSize}
				<span class="text-sm font-normal opacity-80">({shortenFileSize(downloadSize)})</span>
			{/if}
		{/if}
	</button>
	<DropdownMenu.Root bind:open={versionsOpen}>
		<DropdownMenu.Trigger
			class="zephyr-install-versions ml-px rounded-r-xl px-2 py-2.5 text-xl transition-all duration-200 disabled:cursor-not-allowed"
			class:installed={mod.isInstalled}
			class:locked
			{disabled}
		>
			<DropdownArrow open={versionsOpen} class="text-white" />
		</DropdownMenu.Trigger>
		<ContextMenuContent
			type="dropdown"
			style="light"
			items={contextItems}
			class="max-h-90 overflow-y-auto text-base"
		/>
	</DropdownMenu.Root>
</div>

<style>
	.zephyr-install-main {
		background: linear-gradient(135deg, #2D8CF0, #2575D0);
	}
	.zephyr-install-main:hover:not(:disabled) {
		background: linear-gradient(135deg, #3D9CFF, #2D8CF0);
	}
	.zephyr-install-main.installed,
	.zephyr-install-main.locked {
		background: #142240;
		color: #8899AA;
	}

	.zephyr-install-versions {
		background: linear-gradient(135deg, #2575D0, #2D8CF0);
		color: white;
	}
	.zephyr-install-versions:hover:not(:disabled) {
		background: #3D9CFF;
	}
	.zephyr-install-versions.installed,
	.zephyr-install-versions.locked {
		background: #142240;
		color: #8899AA;
	}
</style>
