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
	import { open as openDialog } from '@tauri-apps/plugin-dialog';
	import { pushToast } from '$lib/toast.svelte';
	import type { Plugin } from '$lib/types';

	let busyId: string | null = $state(null);
	let confirmTarget: Plugin | null = $state(null);
	let refreshing = $state(false);
	let devBusy = $state(false);

	async function pickDevPlugin() {
		if (devBusy) return;
		try {
			const picked = await openDialog({ directory: true, multiple: false });
			if (!picked || typeof picked !== 'string') return;
			devBusy = true;
			await plugins.registerLocal(picked);
			pushToast({
				type: 'success',
				message: (i18nState.locale && m.plugins_devMode_toast_loaded()) || ''
			});
		} catch (err) {
			pushToast({
				type: 'error',
				name: (i18nState.locale && m.plugins_devMode_toast_loadFailed()) || '',
				message: err instanceof Error ? err.message : String(err)
			});
		} finally {
			devBusy = false;
		}
	}

	async function reloadDevPlugin(plugin: Plugin) {
		busyId = plugin.id;
		try {
			await plugins.reloadLocal(plugin.id);
			pushToast({
				type: 'success',
				message: (i18nState.locale && m.plugins_devMode_toast_reloaded({ name: plugin.name })) || ''
			});
		} catch (err) {
			pushToast({
				type: 'error',
				name: (i18nState.locale && m.plugins_devMode_toast_reloadFailed()) || '',
				message: err instanceof Error ? err.message : String(err)
			});
		} finally {
			busyId = null;
		}
	}

	async function removeDevPlugin(plugin: Plugin) {
		busyId = plugin.id;
		try {
			await plugins.unregisterLocal(plugin.id);
		} finally {
			busyId = null;
		}
	}

	// Split `feature` into two sections so users distinguish the runtime
	// bundled in Zephyr (archipelago, etc., un-uninstallable) from community
	// plugins authored against the SDK.
	type SectionKey = 'builtIn' | 'feature' | 'theme' | 'game' | 'mod';

	const grouped = $derived.by(() => {
		const order: SectionKey[] = ['builtIn', 'feature', 'theme', 'game', 'mod'];
		const buckets: Record<SectionKey, Plugin[]> = {
			builtIn: [],
			feature: [],
			theme: [],
			game: [],
			mod: []
		};
		for (const plugin of plugins.list) {
			if (plugin.kind === 'feature' && plugin.builtIn) buckets.builtIn.push(plugin);
			else buckets[plugin.kind].push(plugin);
		}
		return order
			.filter((kind) => buckets[kind].length > 0)
			.map((kind) => ({ kind, plugins: buckets[kind] }));
	});

	const sectionTitle: Record<SectionKey, () => string> = {
		builtIn: () => m.plugins_section_builtIn(),
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
		<section class="z-dev-mode">
			<div class="z-dev-mode-head">
				<div>
					<h2 class="z-dev-mode-title">
						<Icon icon="mdi:dev-to" />
						{i18nState.locale && m.plugins_devMode_title()}
					</h2>
					<p class="z-dev-mode-desc">{i18nState.locale && m.plugins_devMode_desc()}</p>
				</div>
				<Button variant="secondary" size="sm" loading={devBusy} onclick={pickDevPlugin}>
					{#snippet icon()}<Icon icon="mdi:folder-open" />{/snippet}
					{i18nState.locale && m.plugins_devMode_load()}
				</Button>
			</div>
			{#if plugins.list.some((p) => p.dev)}
				<div class="z-dev-list">
					{#each plugins.list.filter((p) => p.dev) as plugin (plugin.id)}
						<div class="z-dev-item">
							<div class="z-dev-item-info">
								<span class="z-dev-item-name">{plugin.name}</span>
								<span class="z-dev-item-path" title={plugin.devPath}>{plugin.devPath}</span>
							</div>
							<div class="z-dev-item-actions">
								<Button
									variant="ghost"
									size="sm"
									loading={busyId === plugin.id}
									onclick={() => reloadDevPlugin(plugin)}
								>
									{#snippet icon()}<Icon icon="mdi:refresh" />{/snippet}
									{i18nState.locale && m.plugins_devMode_reload()}
								</Button>
								<Button
									variant="ghost"
									size="sm"
									loading={busyId === plugin.id}
									onclick={() => removeDevPlugin(plugin)}
								>
									{#snippet icon()}<Icon icon="mdi:close" />{/snippet}
									{i18nState.locale && m.plugins_devMode_remove()}
								</Button>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</section>

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
		overflow-y: auto;
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

	.z-dev-mode {
		padding: var(--space-lg) var(--space-xl);
		border-radius: var(--radius-lg);
		background: var(--bg-elevated);
		border: 1px dashed var(--border-default);
	}

	.z-dev-mode-head {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: var(--space-lg);
	}

	.z-dev-mode-title {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-family: var(--font-display);
		font-size: 14px;
		font-weight: 700;
		margin: 0 0 4px;
		color: var(--text-primary);
	}

	.z-dev-mode-desc {
		margin: 0;
		font-size: 12px;
		color: var(--text-muted);
		line-height: 1.5;
		max-width: 560px;
	}

	.z-dev-mode-desc code {
		font-family: var(--font-mono);
		font-size: 11px;
		padding: 1px 5px;
		border-radius: 4px;
		background: var(--bg-base);
		color: var(--text-accent);
	}

	.z-dev-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		margin-top: var(--space-md);
	}

	.z-dev-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-md);
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-md);
		background: var(--bg-base);
		border: 1px solid var(--border-subtle);
	}

	.z-dev-item-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.z-dev-item-name {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.z-dev-item-path {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.z-dev-item-actions {
		display: flex;
		gap: 4px;
		flex-shrink: 0;
	}
</style>
