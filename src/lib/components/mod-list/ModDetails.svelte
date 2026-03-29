<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import Markdown from '$lib/components/ui/Markdown.svelte';

	import ModInfoDialog from '../dialogs/ModInfoDialog.svelte';
	import ModCardList from '../ui/ModCardList.svelte';
	import ModContextMenuContent from './ModContextMenuContent.svelte';

	import { ModType, type Mod, type ModContextItem } from '$lib/types';
	import {
		communityUrl,
		formatModName,
		getMarkdown,
		modIconSrc,
		shortenFileSize,
		shortenNum,
		timeSince
	} from '$lib/util';

	import { DropdownMenu } from 'bits-ui';

	import Icon from '@iconify/svelte';
	import { type Snippet } from 'svelte';
	import * as api from '$lib/api';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		mod: Mod;
		contextItems?: ModContextItem[];
		locked: boolean;
		onclose: () => void;
		children?: Snippet;
	};

	let { mod, contextItems = [], locked, onclose, children }: Props = $props();

	let dependenciesOpen = $state(false);

	let readmeOpen = $state(false);
	let readme: ModInfoDialog;

	let changelogOpen = $state(false);
	let changelog: ModInfoDialog;

	let allContextItems = $derived([
		...contextItems,
		{
			label: m.modDetails_allContextItems_close(),
			icon: 'mdi:close',
			onclick: onclose
		}
	]);

	let readmePromise: Promise<string | null> | null = $state(null);

	function formatReadme(readme: string | null) {
		if (readme === null) return null;

		return readme
			.split('\n')
			.filter((line) => !line.startsWith('# '))
			.join('\n');
	}

	$effect(() => {
		readmePromise = getMarkdown(mod, 'readme').then(formatReadme);
	});
</script>

<div class="zephyr-details">
	<!-- Close & menu buttons -->
	<button
		class="absolute top-4 right-12 z-10 rounded-lg p-1.5 text-[#556677] transition-colors hover:bg-[#142240] hover:text-[#E8ECF1]"
		onclick={onclose}
	>
		<Icon icon="mdi:close" class="text-xl" />
	</button>

	<DropdownMenu.Root>
		<DropdownMenu.Trigger
			class="absolute top-4 right-4 z-10 rounded-lg p-1.5 text-[#556677] transition-colors hover:bg-[#142240] hover:text-[#E8ECF1]"
		>
			<Icon icon="mdi:dots-vertical" class="text-xl" />
		</DropdownMenu.Trigger>
		<ModContextMenuContent style="light" {mod} {locked} items={allContextItems} type="dropdown" />
	</DropdownMenu.Root>

	<div class="light-scrollbar grow overflow-x-hidden overflow-y-auto">
		<!-- Hero section -->
		<div class="zephyr-details-hero">
			<img src={modIconSrc(mod)} class="zephyr-details-icon" alt="" />

			<div class="mt-4">
				<svelte:element
					this={mod.type === ModType.Remote ? 'a' : 'div'}
					class={[
						'text-2xl font-bold text-white break-words leading-tight',
						mod.type === ModType.Remote && 'hover:underline'
					]}
					href={communityUrl(`${mod.author}/${mod.name}`)}
					target="_blank">{formatModName(mod.name)}</svelte:element
				>

				{#if mod.author}
					<div class="mt-1 text-sm text-[#8899AA]">
						{m.modDetails_by()}
						<a class="text-[#2D8CF0] hover:underline" href={communityUrl(mod.author)} target="_blank">
							{mod.author}
						</a>
					</div>
				{/if}

				{#if mod.version}
					<span class="mt-2 inline-block rounded-md bg-[#142240] px-2.5 py-0.5 text-xs font-mono text-[#8899AA]">
						v{mod.version}
					</span>
				{/if}
			</div>
		</div>

		<!-- Tags -->
		<div class="flex flex-wrap gap-1.5 px-5">
			{#if mod.isDeprecated}
				<div class="flex items-center gap-1 rounded-lg bg-red-500/15 px-2.5 py-1 text-xs font-medium text-red-400">
					<Icon icon="mdi:error" class="text-sm" />
					{m.modDetails_deprecated()}
				</div>
			{/if}

			{#if mod.containsNsfw}
				<div class="flex items-center gap-1 rounded-lg bg-red-500/15 px-2.5 py-1 text-xs font-medium text-red-400">
					<Icon icon="material-symbols:explicit" class="text-sm" />
					{m.modDetails_NSFW()}
				</div>
			{/if}
		</div>

		<!-- Stats -->
		<div class="mx-5 mt-4 grid grid-cols-3 gap-2">
			{#if mod.rating !== null}
				<div class="zephyr-stat">
					<Icon icon="mdi:star" class="text-yellow-400 text-lg" />
					<span class="text-sm font-semibold text-[#E8ECF1]">{shortenNum(mod.rating)}</span>
				</div>
			{/if}
			{#if mod.downloads !== null}
				<div class="zephyr-stat">
					<Icon icon="mdi:download" class="text-[#00D4AA] text-lg" />
					<span class="text-sm font-semibold text-[#E8ECF1]">{shortenNum(mod.downloads)}</span>
				</div>
			{/if}
			<div class="zephyr-stat">
				<Icon icon="mdi:weight" class="text-[#556677] text-lg" />
				<span class="text-sm font-semibold text-[#E8ECF1]">{shortenFileSize(mod.fileSize)}</span>
			</div>
		</div>

		{#if mod.lastUpdated !== null}
			<div class="mx-5 mt-2 text-xs text-[#556677]">
				{m.modDetails_lastUpdated({ time: timeSince(new Date(mod.lastUpdated)) })}
			</div>
		{/if}

		<!-- Categories -->
		{#if mod.categories}
			<div class="mx-5 mt-3 flex flex-wrap gap-1">
				{#each mod.categories as category}
					<span class="rounded-full bg-[#142240] px-3 py-0.5 text-[11px] font-medium text-[#8899AA]">
						{category}
					</span>
				{/each}
			</div>
		{/if}

		{#if mod.description !== null}
			<p class="mx-5 mt-3 text-sm leading-relaxed text-[#8899AA] lg:hidden">
				{mod.description}
			</p>
		{/if}

		<!-- Readme -->
		<div class="mx-5 hidden lg:block">
			{#await readmePromise}
				<div role="status" class="animate-pulse mt-4">
					<div class="bg-[#142240] h-6 w-48 rounded-lg"></div>
					<div class="bg-[#142240] mt-3 h-3 w-full rounded-full"></div>
					<div class="bg-[#142240] mt-2 h-3 w-4/5 rounded-full"></div>
					<div class="bg-[#142240] mt-2 h-3 w-3/5 rounded-full"></div>
				</div>
			{:then readme}
				<Markdown source={readme ?? m.modDetails_noFound()} />
			{/await}
		</div>
	</div>

	<!-- Bottom actions -->
	<div class="zephyr-details-actions">
		{#if mod.configFile}
			<a
				href={'/config?file=' + mod.configFile}
				class="zephyr-action-link"
			>
				<Icon icon="mdi:file-cog" class="text-base" />
				{m.modDetails_editConfig()}
			</a>
		{/if}

		<div class="flex gap-1.5">
			<button
				class="zephyr-action-btn grow"
				onmouseenter={() => changelog.fetchMarkdown()}
				onclick={() => (changelogOpen = true)}
			>
				<Icon icon="mdi:file-document" class="text-base" />
				{m.modDetails_changeLog()}
			</button>

			<button
				class="zephyr-action-btn grow"
				onmouseenter={() => readme.fetchMarkdown()}
				onclick={() => (readmeOpen = true)}
			>
				<Icon icon="mdi:info" class="text-base" />
				{m.modDetails_details()}
			</button>
		</div>

		{#if mod.dependencies !== null && mod.dependencies.length > 0}
			<button
				class="zephyr-action-btn"
				onclick={() => (dependenciesOpen = true)}
			>
				<Icon icon="material-symbols:network-node" class="text-base" />
				{m.modDetails_dependencies()}
				<span class="ml-auto rounded-md bg-[#0B1628] px-2 py-0.5 text-[11px] font-semibold text-[#2D8CF0]">
					{mod.dependencies.length}
				</span>
			</button>
		{/if}

		{@render children?.()}
	</div>
</div>

<Dialog title="Dependencies of {mod.name}" bind:open={dependenciesOpen}>
	{#if mod.dependencies}
		<ModCardList names={mod.dependencies} class="mt-4" />
	{/if}
</Dialog>

<ModInfoDialog bind:this={readme} bind:open={readmeOpen} {mod} type="readme" />
<ModInfoDialog
	bind:this={changelog}
	bind:open={changelogOpen}
	{mod}
	useLatest={true}
	type="changelog"
/>

<style>
	.zephyr-details {
		position: relative;
		display: flex;
		flex-direction: column;
		width: 40%;
		min-width: 18rem;
		border-left: 1px solid #1A2A42;
		background: #0B1628;
		color: white;
	}

	.zephyr-details-hero {
		padding: 1.5rem 1.25rem 1rem;
		display: flex;
		flex-direction: column;
		align-items: flex-start;
	}

	.zephyr-details-icon {
		width: 80px;
		height: 80px;
		border-radius: 16px;
		object-fit: cover;
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
		border: 1px solid #1A2A42;
	}

	.zephyr-stat {
		display: flex;
		align-items: center;
		gap: 6px;
		background: #0F1D32;
		border: 1px solid #1A2A42;
		border-radius: 10px;
		padding: 8px 10px;
	}

	.zephyr-details-actions {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 12px 16px;
		border-top: 1px solid #1A2A42;
	}

	.zephyr-action-btn {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		border-radius: 10px;
		background: #142240;
		color: #E8ECF1;
		font-size: 13px;
		font-weight: 500;
		transition: all 0.15s ease;
	}

	.zephyr-action-btn:hover {
		background: #1A2A42;
		transform: translateY(-1px);
	}

	.zephyr-action-link {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		border-radius: 10px;
		background: rgba(45, 140, 240, 0.1);
		color: #2D8CF0;
		font-size: 13px;
		font-weight: 500;
		transition: all 0.15s ease;
		text-decoration: none;
	}

	.zephyr-action-link:hover {
		background: rgba(45, 140, 240, 0.18);
		color: #4DA3FF;
	}
</style>
