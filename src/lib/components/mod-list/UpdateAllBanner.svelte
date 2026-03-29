<script lang="ts">
	import Checklist from '$lib/components/ui/Checklist.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import type { AvailableUpdate } from '$lib/types';
	import Icon from '@iconify/svelte';
	import ModCard from '../ui/ModCard.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import * as api from '$lib/api';
	import Button from '$lib/components/ui/Button.svelte';
	import { SvelteMap } from 'svelte/reactivity';
	import { updateBanner } from '$lib/state/misc.svelte';
	import { m } from '$lib/paraglide/messages';
	import { pluralizeOption } from '$lib/i18n';

	type Props = {
		updates: AvailableUpdate[];
	};

	let { updates }: Props = $props();

	let dialogOpen = $state(false);
	let include: SvelteMap<AvailableUpdate, boolean> = $state(new SvelteMap());

	let shownUpdates = $derived(updates.filter((update) => !update.ignore));

	$effect(() => {
		if (dialogOpen && shownUpdates.length === 0) {
			dialogOpen = false;
		}
	});

	async function updateAll() {
		let uuids = shownUpdates
			.filter((update) => include.get(update) ?? true)
			.map((update) => update.packageUuid);

		dialogOpen = false;

		await api.profile.update.mods(uuids, true);
	}
</script>

{#if shownUpdates.length > updateBanner.threshold}
	<div class="zephyr-update-banner mr-3 mb-1 flex items-center rounded-xl py-1.5 pr-1.5 pl-3.5 text-sm text-white">
		<Icon icon="mdi:arrow-up-circle" class="mr-2 text-lg" />
		{pluralizeOption(shownUpdates.length, m.updateAllBanner_content_is(), 'is', 'are')}
		<b class="mx-1">{shownUpdates.length}</b>
		{pluralizeOption(shownUpdates.length, m.updateAllBanner_content_update(), 'update', 'updates')}
		<button
			class="hover:text-white ml-1 font-semibold text-white/90 hover:underline"
			onclick={() => (dialogOpen = true)}
		>
			{m.updateAllBanner_button()}
		</button>

		<button
			class="hover:bg-white/10 ml-auto rounded-lg p-1 text-lg text-white/70 hover:text-white transition-colors"
			onclick={() => (updateBanner.threshold = shownUpdates.length)}
		>
			<Icon icon="mdi:close" />
		</button>
	</div>
{/if}

<ConfirmDialog title={m.updateAllBanner_dialog_title()} bind:open={dialogOpen}>
	{m.updateAllBanner_dialog_content()}

	<Checklist
		title={m.updateAllBanner_dialog_list_title()}
		items={shownUpdates}
		class="mt-1"
		maxHeight="sm"
		get={(update, _) => include.get(update) ?? true}
		set={(update, _, value) => include.set(update, value)}
	>
		{#snippet item({ item: update })}
			<ModCard fullName={update.fullName} showVersion={false} />

			<span class="text-light text-[#556677] ml-auto pl-1">{update.old}</span>
			<Icon icon="mdi:arrow-right" class="text-[#556677] mx-1.5 text-lg" />
			<span class="text-[#2D8CF0] text-lg font-semibold">{update.new}</span>

			<Tooltip text={m.updateAllBanner_dialog_list_content()} side="left" sideOffset={-2}>
				<button
					class="text-[#556677] hover:bg-[#142240] hover:text-[#E8ECF1] ml-2 rounded-sm p-1.5"
					onclick={() => {
						update.ignore = true;
						include.delete(update);

						api.profile.update.ignore(update.versionUuid);
					}}><Icon icon="mdi:notifications-off" /></button
				>
			</Tooltip>
		{/snippet}
	</Checklist>

	{#snippet buttons()}
		<Button color="accent" icon="mdi:download" onclick={updateAll}
			>{m.updateAllBanner_dialog_button()}</Button
		>
	{/snippet}
</ConfirmDialog>

<style>
	.zephyr-update-banner {
		background: linear-gradient(135deg, #2D8CF0, #1A6DD0);
		box-shadow: 0 2px 12px rgba(45, 140, 240, 0.2);
	}
</style>
