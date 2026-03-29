<script lang="ts">
	import * as api from '$lib/api';
	import type { ConfigSection, ConfigFile } from '$lib/types';
	import { capitalize } from '$lib/util';
	import Icon from '@iconify/svelte';
	import ExpandedConfigEntryDialog from '$lib/components/dialogs/ExpandedConfigEntryDialog.svelte';

	import Button from '$lib/components/ui/Button.svelte';
	import ConfigFileEditor from '$lib/components/config/ConfigFileEditor.svelte';
	import ProfileLockedBanner from '$lib/components/mod-list/ProfileLockedBanner.svelte';
	import ConfigFileList from '$lib/components/config/ConfigFileList.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { m } from '$lib/paraglide/messages';

	let selectedFile: ConfigFile | null = $state(null);
	let selectedSection: ConfigSection | null = $state(null);
</script>

<div class="flex grow overflow-hidden">
	<ConfigFileList bind:selectedFile bind:selectedSection />

	<div class="flex max-w-4xl grow flex-col overflow-y-auto py-4">
		{#if profiles.activeLocked}
			<ProfileLockedBanner class="mx-4 mb-4" />
		{/if}

		{#if selectedFile !== null}
			<div class="shrink-0 truncate px-4 text-xl font-bold text-[#E8ECF1]">
				{selectedFile.relativePath}
				{#if selectedSection}
					<span class="mx-1 text-[#3A4A5C]">/</span>
					<span class="text-[#8899AA]">{selectedSection.name.length > 0 ? selectedSection.name : m.config_nameLess()}</span>
				{/if}
			</div>

			{#if selectedFile.type === 'ok'}
				<ConfigFileEditor
					file={selectedFile}
					section={selectedSection}
					locked={profiles.activeLocked}
				/>
			{:else if selectedFile.type === 'unsupported'}
				<div class="text-[#556677] mb-1 px-4">
					{m.config_unsupported_content()}
				</div>
				<Button
					class="mx-4 max-w-max"
					color="primary"
					onclick={() => api.config.openFile(selectedFile!)}
					icon="mdi:open-in-new"
				>
					{m.config_unsupported_button()}
				</Button>
			{:else if selectedFile.type === 'err'}
				<div class="text-[#556677] mb-1 px-4">
					{m.config_err_content()}
				</div>
				<code class="bg-[#0B1628] mx-4 mb-1 flex rounded-sm p-4 text-red-500">
					{capitalize(selectedFile.error)}
				</code>
				<Button
					class="mx-4 max-w-max"
					color="primary"
					onclick={() => api.config.openFile(selectedFile!)}
					icon="icon=mdi:open-in-new"
				>
					{m.config_err_button()}
				</Button>
			{/if}
		{:else}
			<div class="flex w-full grow flex-col items-center justify-center gap-2">
				<div class="rounded-2xl bg-[#142240]/40 p-5">
					<Icon icon="mdi:tune-variant" class="text-[#3A4A5C] text-5xl" />
				</div>
				<div class="text-base font-medium text-[#8899AA]">{m.config_content()}</div>
			</div>
		{/if}
	</div>
</div>

<ExpandedConfigEntryDialog />
