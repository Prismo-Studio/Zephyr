<script lang="ts">
	import Header from '$lib/components/layout/Header.svelte';
	import Icon from '@iconify/svelte';
	import PluginCard from '$lib/components/plugins/PluginCard.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import plugins from '$lib/state/plugins.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { onMount } from 'svelte';
	import type { Plugin } from '$lib/types';

	let busyId: string | null = $state(null);
	let confirmTarget: Plugin | null = $state(null);
	let refreshing = $state(false);

	const grouped = $derived.by(() => {
		const order: Plugin['kind'][] = ['feature', 'theme', 'game', 'mod'];
		const buckets: Record<Plugin['kind'], Plugin[]> = {
			feature: [],
			theme: [],
			game: [],
			mod: []
		};
		for (const plugin of plugins.list) buckets[plugin.kind].push(plugin);
		return order.filter((kind) => buckets[kind].length > 0).map((kind) => ({ kind, plugins: buckets[kind] }));
	});

	const sectionTitle: Record<Plugin['kind'], () => string> = {
		feature: () => m.plugins_section_feature(),
		theme: () => m.plugins_section_theme(),
		game: () => m.plugins_section_game(),
		mod: () => m.plugins_section_mod()
	};

	async function uninstall(plugin: Plugin) {
		busyId = plugin.id;
		try {
			// Built-in features just toggle off; everything else is a real uninstall
			// that removes the on-disk asset and registry-installed flag.
			if (plugin.builtIn) {
				await plugins.setEnabled(plugin.id, false);
			} else {
				await plugins.uninstall(plugin.id);
			}
		} finally {
			busyId = null;
			confirmTarget = null;
		}
	}

	async function reinstall(plugin: Plugin) {
		busyId = plugin.id;
		try {
			if (plugin.builtIn) {
				await plugins.setEnabled(plugin.id, true);
			} else {
				await plugins.install(plugin.id);
			}
		} finally {
			busyId = null;
		}
	}

	async function refreshRegistry() {
		if (refreshing) return;
		refreshing = true;
		try {
			await plugins.refetch();
		} finally {
			refreshing = false;
		}
	}

	onMount(() => {
		// Re-pull from GitHub when the page opens so manifest edits show up
		// without requiring an app restart. Falls back silently if offline.
		plugins.refetch().catch(() => plugins.refresh());
	});
</script>

<div class="z-plugins-page">
	<div class="z-plugins-header-wrapper">
		<Header
			title={i18nState.locale && m.plugins_page_title()}
			subtitle={i18nState.locale && m.plugins_page_subtitle()}
		>
			{#snippet actions()}
				<Button variant="ghost" size="sm" loading={refreshing} onclick={refreshRegistry}>
					<span class="z-plugins-refresh-label">
						<Icon icon="mdi:refresh" />
						{i18nState.locale && m.plugins_action_refresh()}
					</span>
				</Button>
			{/snippet}
		</Header>
	</div>

	<div class="z-plugins-content">
		{#if !plugins.ready}
			<p class="z-plugins-loading">{i18nState.locale && m.plugins_page_loading()}</p>
		{:else if plugins.list.length === 0}
			<p class="z-plugins-empty">{i18nState.locale && m.plugins_page_empty()}</p>
		{:else}
			{#each grouped as group (group.kind)}
				<section class="z-plugin-group">
					<h2 class="z-plugin-group-title">{i18nState.locale && sectionTitle[group.kind]()}</h2>
					<div class="z-plugin-list">
						{#each group.plugins as plugin (plugin.id)}
							<PluginCard
								{plugin}
								busy={busyId === plugin.id}
								onuninstall={() => (confirmTarget = plugin)}
								onreinstall={() => reinstall(plugin)}
							/>
						{/each}
					</div>
				</section>
			{/each}
		{/if}
	</div>
</div>

<Modal
	open={confirmTarget !== null}
	title={i18nState.locale && m.plugins_uninstall_title()}
	onclose={() => (confirmTarget = null)}
>
	{#snippet children()}
		{#if confirmTarget}
			<p>
				{i18nState.locale && m.plugins_uninstall_question({ name: confirmTarget.name })}
				{#if confirmTarget.builtIn}
					{i18nState.locale && m.plugins_uninstall_builtIn_note()}
				{:else}
					{i18nState.locale && m.plugins_uninstall_external_note()}
				{/if}
			</p>
		{/if}
	{/snippet}
	{#snippet actions()}
		<Button variant="ghost" onclick={() => (confirmTarget = null)}>
			{i18nState.locale && m.plugins_uninstall_cancel()}
		</Button>
		<Button
			variant="danger"
			loading={busyId !== null}
			onclick={() => confirmTarget && uninstall(confirmTarget)}
		>
			{i18nState.locale && m.plugins_action_uninstall()}
		</Button>
	{/snippet}
</Modal>

<style>
	.z-plugins-page {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.z-plugins-header-wrapper {
		width: 100%;
		max-width: 1200px;
		margin: 0 auto;
		padding: var(--space-xl) var(--space-xl) 0;
	}

	.z-plugins-refresh-label {
		display: inline-flex;
		align-items: center;
		gap: var(--space-xs);
	}

	.z-plugins-content {
		flex: 1;
		overflow-y: auto;
		width: 100%;
		max-width: 1200px;
		margin: 0 auto;
		padding: var(--space-lg) var(--space-xl) var(--space-3xl);
		display: flex;
		flex-direction: column;
		gap: var(--space-2xl);
	}

	.z-plugin-group-title {
		font-family: var(--font-display);
		font-size: 13px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-muted);
		margin-bottom: var(--space-md);
	}

	.z-plugin-list {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
		gap: var(--space-md);
		align-items: stretch;
	}

	.z-plugins-loading,
	.z-plugins-empty {
		padding: var(--space-2xl);
		text-align: center;
		color: var(--text-muted);
		font-size: 14px;
	}
</style>
