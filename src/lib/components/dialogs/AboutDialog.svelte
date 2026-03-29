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
	<div class="h-3"></div>
	<img src="logo.png" alt="Zephyr Logo" class="float-right size-20 rounded-lg" />
	<div>
		<h3 class="text-xl font-semibold text-white">Zephyr</h3>
		<p class="text-sm text-[#556677]">by Prismo Studio</p>
		<p class="text-[#8899AA]">
			{m.aboutDialog_version({ version: version })}
			<br />
			GNU General Public License v3.0
		</p>
		<div class="mt-3 flex items-center gap-2">
			<Icon icon="mdi:file-document" class="text-xl text-white" />
			<Link href="https://github.com/prismo-studio/zephyr/blob/main/CHANGELOG.md"
				>{m.aboutDialog_changelog()}</Link
			>
		</div>
		<div class="mt-1 flex items-center gap-2">
			<Icon icon="mdi:github" class="text-xl text-white" />
			<Link href="https://github.com/prismo-studio/zephyr">GitHub</Link>
		</div>
		<div class="mt-1 flex items-center gap-2">
			<Icon icon="mdi:web" class="text-xl text-white" />
			<Link href="https://prismo-studio.com">prismo-studio.com</Link>
		</div>
		<div class="mt-3 flex items-center gap-2">
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
					<Icon icon="mdi:arrow-up-circle" class="text-[#2D8CF0] inline text-xl" />
					<span class="text-[#2D8CF0]"
						>{m.aboutDialog_newVersion({ version: updates.next.version })}</span
					>
				{:else}
					<Icon icon="mdi:check" class="text-[#8899AA] text-xl" />
					<span class="text-[#8899AA]">{m.aboutDialog_latestVersion()}</span>
				{/if}
			{/if}
		</div>
	</div>
</Dialog>
