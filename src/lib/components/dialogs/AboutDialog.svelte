<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import Link from '$lib/components/ui/Link.svelte';

	import Icon from '@iconify/svelte';
	import { getVersion } from '@tauri-apps/api/app';
	import { onMount } from 'svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import updates from '$lib/state/update.svelte';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		open?: boolean;
	};

	let { open = $bindable(false) }: Props = $props();

	let version = $state('');
	let checkedUpdate = $state(false);

	$effect(() => {
		if (open) checkedUpdate = false;
	});

	onMount(async () => {
		version = await getVersion();
	});
</script>

<Dialog bind:open title={m.aboutDialog_title()}>
	<div class="mt-4 flex items-start gap-5">
		<div class="shrink-0">
			<img src="logo.png" alt="Zephyr Logo" class="size-20 rounded-2xl shadow-lg" style="box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);" />
		</div>
		<div class="grow">
			<h3 class="text-2xl font-bold text-white">Zephyr</h3>
			<p class="mt-0.5 text-sm text-[#8899AA]">by <span class="text-[#2D8CF0]">Prismo Studio</span></p>
			<p class="mt-2 text-sm text-[#556677]">
				{m.aboutDialog_version({ version: version })} &middot; GPL-3.0
			</p>
		</div>
	</div>

	<div class="mt-5 flex flex-col gap-1.5 rounded-xl border border-[#1A2A42] bg-[#0B1628] p-3">
		<a href="https://github.com/prismo-studio/zephyr/blob/main/CHANGELOG.md" target="_blank" class="flex items-center gap-2.5 rounded-lg px-3 py-2 text-sm text-[#8899AA] hover:bg-[#142240] hover:text-[#E8ECF1] transition-colors">
			<Icon icon="mdi:file-document" class="text-lg text-[#556677]" />
			{m.aboutDialog_changelog()}
		</a>
		<a href="https://github.com/prismo-studio/zephyr" target="_blank" class="flex items-center gap-2.5 rounded-lg px-3 py-2 text-sm text-[#8899AA] hover:bg-[#142240] hover:text-[#E8ECF1] transition-colors">
			<Icon icon="mdi:github" class="text-lg text-[#556677]" />
			GitHub
		</a>
		<a href="https://prismo-studio.com" target="_blank" class="flex items-center gap-2.5 rounded-lg px-3 py-2 text-sm text-[#8899AA] hover:bg-[#142240] hover:text-[#E8ECF1] transition-colors">
			<Icon icon="mdi:web" class="text-lg text-[#556677]" />
			prismo-studio.com
		</a>
	</div>

	<div class="mt-4 flex items-center gap-2">
		<Button
			onclick={() => updates.refresh().then(() => (checkedUpdate = true))}
			loading={updates.isChecking}
			color="primary"
			class="mr-2"
			icon="mdi:refresh"
		>
			{m.aboutDialog_checkUpdate()}
		</Button>

		{#if !updates.isChecking && checkedUpdate}
			{#if updates.next}
				<Icon icon="mdi:arrow-up-circle" class="text-[#00D4AA] inline text-lg" />
				<span class="text-sm text-[#00D4AA]"
					>{m.aboutDialog_newVersion({ version: updates.next.version })}</span
				>
			{:else}
				<Icon icon="mdi:check-circle" class="text-[#8899AA] text-lg" />
				<span class="text-sm text-[#8899AA]">{m.aboutDialog_latestVersion()}</span>
			{/if}
		{/if}
	</div>
</Dialog>
