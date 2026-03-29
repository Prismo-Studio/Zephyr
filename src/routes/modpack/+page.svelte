<script lang="ts">
	import InputField from '$lib/components/ui/InputField.svelte';
	import FormField from '$lib/components/ui/FormField.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import PathField from '$lib/components/ui/PathField.svelte';
	import Markdown from '$lib/components/ui/Markdown.svelte';
	import Link from '$lib/components/ui/Link.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import ApiKeyDialog from '$lib/components/dialogs/ApiKeyDialog.svelte';

	import * as api from '$lib/api';
	import type { ModpackArgs } from '$lib/types';
	import { open } from '@tauri-apps/plugin-dialog';

	import Dialog from '$lib/components/ui/Dialog.svelte';
	import Checklist from '$lib/components/ui/Checklist.svelte';
	import ResizableInputField from '$lib/components/ui/ResizableInputField.svelte';
	import { toHeaderCase } from 'js-convert-case';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import { SvelteMap } from 'svelte/reactivity';
	import profiles from '$lib/state/profile.svelte';
	import games from '$lib/state/game.svelte';
	import { apiKeyDialog } from '$lib/state/misc.svelte';
	import { m, modpack_button_export, modpack_includeFiles_title } from '$lib/paraglide/messages';

	const URL_PATTERN =
		'[Hh][Tt][Tt][Pp][Ss]?://(?:(?:[a-zA-Z\u00a1-\uffff0-9]+-?)*[a-zA-Z\u00a1-\uffff0-9]+)(?:.(?:[a-zA-Z\u00a1-\uffff0-9]+-?)*[a-zA-Z\u00a1-\uffff0-9]+)*(?:.(?:[a-zA-Z\u00a1-\uffff]{2,}))(?::d{2,5})?(?:/[^s]*)?';

	let name: string = $state('');
	let author: string = $state('');
	let selectedCategories: string[] = $state([]);
	let nsfw: boolean = $state(false);
	let description: string = $state('');
	let readme: string = $state('');
	let changelog: string = $state('');
	let versionNumber: string = $state('');
	let iconPath: string = $state('');
	let websiteUrl: string = $state('');
	let includeDisabled: boolean = $state(false);

	let doneDialogOpen = $state(false);
	let loading: string | null = $state(null);

	let includeFiles = $state(new SvelteMap<string, boolean>());
	let includedFileCount = $state(0);
	let includedFilesSearch = $state('');

	let shownFileIncludes = $derived.by(() => {
		let fileNames: string[];

		if (includedFilesSearch.length === 0) {
			fileNames = Array.from(includeFiles.keys());
		} else {
			let lowerSearch = includedFilesSearch.toLowerCase();

			fileNames = Array.from(
				includeFiles
					.keys()
					.filter((path) => path.toLowerCase().replace('\\', '/').includes(lowerSearch))
			);
		}

		return fileNames.sort();
	});

	function countIncludedFiles(includeFiles?: SvelteMap<string, boolean>) {
		if (!includeFiles) return 0;

		let count = 0;
		for (let enabled of includeFiles.values()) {
			if (enabled) count++;
		}
		return count;
	}

	async function refresh() {
		loading = m.modpack_refresh_loading();

		let args = await api.profile.export.getPackArgs();

		name = args.name;
		author = args.author;
		nsfw = args.nsfw;
		description = args.description;
		selectedCategories = args.categories;
		changelog = args.changelog;
		readme = args.readme;
		versionNumber = args.versionNumber;
		iconPath = args.iconPath;
		websiteUrl = args.websiteUrl;
		includeDisabled = args.includeDisabled;
		includeFiles = new SvelteMap(Object.entries(args.includeFileMap));

		loading = null;
	}

	async function browseIcon() {
		let path = await open({
			defaultPath: iconPath.length > 0 ? iconPath : undefined,
			title: m.modpack_browseIcon_title(),
			filters: [{ name: m.modpack_browseIcon_filter(), extensions: ['png', 'jpg', 'jpeg', 'gif'] }]
		});

		if (path === null) return;
		iconPath = path;
		saveArgs();
	}

	async function generateChangelog(all: boolean) {
		changelog = await api.profile.export.generateChangelog(args(), all);
		saveArgs();
	}

	async function exportToFile() {
		let dir = await open({
			title: m.modpack_exportToFile_title(),
			defaultPath: `${name}.zip`,
			directory: true
		});

		if (!dir) return;

		loading = m.modpack_exportToFile_loading();
		try {
			await api.profile.export.exportPack(dir, args());
		} finally {
			loading = null;
		}
	}

	async function uploadToThunderstore() {
		let hasToken = await api.thunderstore.hasToken();

		if (!hasToken) {
			apiKeyDialog.open = true;

			// wait until api key has been set
			await new Promise<void>((resolve) => {
				const interval = setInterval(() => {
					if (!apiKeyDialog.open) {
						clearInterval(interval);
						resolve();
					}
				}, 100);

				return () => clearInterval(interval);
			});

			hasToken = await api.thunderstore.hasToken();

			if (!hasToken) return;
		}

		loading = m.modpack_uploadToThunderstore_loading();
		try {
			await api.profile.export.uploadPack(args());
			doneDialogOpen = true;
		} finally {
			loading = null;
		}
	}

	function saveArgs() {
		// wait a tick to ensure the variables are updated
		setTimeout(() => {
			api.profile.export.setPackArgs(args());
		});
	}

	function args(): ModpackArgs {
		return {
			name,
			description,
			author,
			nsfw,
			readme,
			changelog,
			versionNumber,
			iconPath,
			websiteUrl,
			includeDisabled,
			includeFileMap: includeFiles,
			categories: selectedCategories
		};
	}

	$effect(() => {
		profiles.active;
		refresh();
	});

	// some communities don't have a specific modpack category
	let modpackCategoryExists = $derived(
		games.categories.some((category) => category.slug === 'modpacks')
	);

	// make sure the modpacks category is always selected if it exists
	$effect(() => {
		if (
			modpackCategoryExists &&
			selectedCategories &&
			!selectedCategories.some((category) => category === 'modpacks')
		) {
			selectedCategories = ['modpacks', ...selectedCategories];
		}
	});

	$effect(() => {
		includedFileCount = countIncludedFiles(includeFiles);
	});
</script>

<div class="relative mx-auto flex h-full w-full max-w-4xl grow flex-col gap-3 overflow-y-auto px-8 py-6">
	<!-- Page Header -->
	<div class="mb-4">
		<h1 class="text-2xl font-bold text-[#E8ECF1]">{m.modpack_header_title()}</h1>
		<p class="mt-1 text-sm text-[#556677]">{m.modpack_header_description()}</p>
	</div>

	{#if loading}
		<div class="absolute inset-0 flex items-center justify-center gap-3 text-lg text-[#8899AA]">
			<Spinner />
			{loading}
		</div>
	{:else}
		<FormField
			label={m.modpack_name_title()}
			description={m.modpack_name_description()}
			required={true}
		>
			<InputField
				onchange={saveArgs}
				bind:value={name}
				placeholder={m.modpack_name_placeholder()}
				required={true}
				pattern="^[a-zA-Z0-9_]+$"
				class="w-full"
			/>
		</FormField>

		<FormField
			label={m.modpack_author_title()}
			description={m.modpack_author_description()}
			required
		>
			<InputField
				onchange={saveArgs}
				bind:value={author}
				placeholder={m.modpack_author_placeholder()}
				class="w-full"
				required
			/>
		</FormField>

		<FormField label={m.modpack_description_title()} description={m.modpack_description_description()} required>
			<InputField
				onchange={saveArgs}
				bind:value={description}
				placeholder={m.modpack_description_placeholder()}
				maxlength={250}
				class="w-full"
				required
			/>
		</FormField>

		<FormField
			label={m.modpack_categories_title()}
			description={m.modpack_categories_description()}
		>
			<Select
				items={games.categories.map((category) => ({
					label: category.name,
					value: category.slug
				}))}
				bind:value={selectedCategories}
				onValueChange={saveArgs}
				type="multiple"
				triggerClass="w-full"
			>
				{#snippet label()}
					{#if selectedCategories.length === 0}
						<span class="text-[#556677] truncate pl-2">{m.modpack_categories_content()}</span>
					{:else}
						<div class="flex flex-wrap gap-1">
							{#each selectedCategories as category}
								<div class="bg-[#142240] text-[#E8ECF1] rounded-md py-1 pr-1 pl-3 text-sm">
									<span class="truncate overflow-hidden">{toHeaderCase(category)}</span>

									<button
										class="hover:bg-[#1A2A42] ml-1 rounded-md px-1.5"
										onclick={(evt) => {
											evt.stopPropagation();
											selectedCategories = selectedCategories.filter((cat) => cat !== category);
										}}
									>
										x
									</button>
								</div>
							{/each}
						</div>
					{/if}
				{/snippet}
			</Select>
		</FormField>

		<FormField
			label={m.modpack_version_title()}
			description={m.modpack_version_description()}
			required
		>
			<InputField
				onchange={saveArgs}
				bind:value={versionNumber}
				placeholder={m.modpack_version_placeholder()}
				required={true}
				pattern="^\d+\.\d+\.\d+$"
				class="w-full"
			/>
		</FormField>

		<FormField label={m.modpack_website_title()} description={m.modpack_website_description()}>
			<InputField
				onchange={saveArgs}
				bind:value={websiteUrl}
				placeholder={m.modpack_website_placeholder()}
				pattern={URL_PATTERN}
				class="w-full"
			/>
		</FormField>

		<FormField
			label={m.modpack_icon_title()}
			description={m.modpack_icon_description()}
			required
		>
			<PathField icon="mdi:file-image" onclick={browseIcon} value={iconPath} />
		</FormField>

		<FormField
			label={m.modpack_readme_title()}
			description={m.modpack_readme_description()}
			required
		>
			<ResizableInputField
				onchange={saveArgs}
				bind:value={readme}
				placeholder={m.modpack_readme_placeholder()}
				mono={true}
			/>

			<details class="mt-1">
				<summary class="text-[#2D8CF0] cursor-pointer text-sm hover:text-[#00D4AA] transition-colors">
					{m.modpack_readme_preview()}
				</summary>
				<Markdown class="mt-1 px-4" source={readme} />
				<div class="bg-[#1A2A42] mt-4 h-px"></div>
			</details>
		</FormField>

		<FormField
			label={m.modpack_changeLog_title()}
			description={m.modpack_changeLog_description()}
		>
			<ResizableInputField
				onchange={saveArgs}
				bind:value={changelog}
				placeholder={m.modpack_changeLog_placeholder()}
				mono={true}
			/>

			<Button color="primary" onclick={() => generateChangelog(false)}>
				{m.modpack_changeLog_button_single({versionNumber})}
			</Button>
			<Button color="primary" onclick={() => generateChangelog(true)}>
				{m.modpack_changeLog_button_all()}
			</Button>

			<details class="mt-1">
				<summary class="text-[#2D8CF0] cursor-pointer text-sm hover:text-[#00D4AA] transition-colors">{m.modpack_changeLog_preview()}</summary>
				<Markdown class="mt-1 px-4" source={changelog} />
				<div class="bg-[#1A2A42] mt-4 h-px"></div>
			</details>
		</FormField>

		<FormField
			label={m.modpack_includeFiles_title({count : includedFileCount, size: includeFiles?.size})}
			description={m.modpack_includeFiles_description()}
		>
			<details>
				<summary class="text-[#2D8CF0] cursor-pointer text-sm hover:text-[#00D4AA] transition-colors">{m.modpack_includeFiles_preview()}</summary>
				<InputField bind:value={includedFilesSearch} class="w-full" placeholder={m.modpack_includeFiles_placeholder()} />
				<Checklist
					class="mt-1"
					title={m.modpack_includeFiles_list_title()}
					items={shownFileIncludes}
					getLabel={(item) => item}
					get={(item) => includeFiles.get(item) ?? false}
					set={(item, _, value) => {
						includeFiles.set(item, value);
						includeFiles = includeFiles;
					}}
				/>
			</details>
		</FormField>

		<div class="text-[#E8ECF1] mt-3 flex items-center rounded-xl border border-[#1A2A42] bg-[#0B1628] px-4 py-3 text-base font-medium">
			<span class="max-w-96 grow">{m.modpack_NSFW_title()}</span>

			<Checkbox onCheckedChange={saveArgs} bind:checked={nsfw} />
		</div>

		<div class="text-[#E8ECF1] flex items-center rounded-xl border border-[#1A2A42] bg-[#0B1628] px-4 py-3 text-base font-medium">
			<span class="max-w-96 grow">{m.modpack_disabled_title()}</span>

			<Checkbox onCheckedChange={saveArgs} bind:checked={includeDisabled} />
		</div>

		<div class="mt-6 flex justify-end gap-3 border-t border-[#1A2A42] pt-6">
			<Button color="primary" icon="mdi:export" onclick={exportToFile}>
				{m.modpack_button_export()}
			</Button>
			<Button color="accent" icon="mdi:upload" onclick={uploadToThunderstore}>
				{m.modpack_button_publish()}
			</Button>
		</div>
	{/if}
</div>

<ApiKeyDialog />

<Dialog bind:open={doneDialogOpen} title={m.modpack_dialog_title()}>
	<p class="text-[#8899AA] mt-2">
		{m.modpack_dialog_content_1({name, versionNumber})}
		<Link href="https://thunderstore.io/c/{games.active?.slug}/p/{author}/{name}">
			{m.modpack_dialog_content_2()}
		</Link>
	</p>

	<div class="text-[#556677] mt-3 text-sm">
		{m.modpack_dialog_content_3()}
		<br />
		{m.modpack_dialog_content_4()}
	</div>
</Dialog>
