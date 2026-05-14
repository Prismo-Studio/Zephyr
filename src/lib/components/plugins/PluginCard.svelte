<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import type { Plugin } from '$lib/types';

	type Props = {
		plugin: Plugin;
		busy?: boolean;
		onuninstall?: () => void;
		onreinstall?: () => void;
	};

	let { plugin, busy = false, onuninstall, onreinstall }: Props = $props();

	const typeLabel: Record<Plugin['kind'], () => string> = {
		feature: () => m.plugins_type_feature(),
		theme: () => m.plugins_type_theme(),
		game: () => m.plugins_type_game(),
		mod: () => m.plugins_type_mod()
	};

	// Built-in plugins ship with translated descriptions in the locale catalogue;
	// third-party plugins from the registry expose their own description verbatim.
	const builtInDescription: Record<string, () => string> = {
		archipelago: () => m.plugins_description_archipelago()
	};

	const description = $derived(
		(i18nState.locale && builtInDescription[plugin.id]?.()) || plugin.description
	);

	// Iconify ids look like "prefix:name". Anything else (paths, URLs) renders as an image.
	const isIconifyId = $derived(/^[a-z0-9-]+:[a-z0-9-]+$/i.test(plugin.icon));

	// Dev plugins send the icon as `file://<absolute-path>` so the backend
	// doesn't have to know about Tauri's asset URL format. Convert here.
	const iconSrc = $derived(
		plugin.icon.startsWith('file://')
			? convertFileSrc(plugin.icon.slice('file://'.length))
			: plugin.icon
	);
</script>

<article class="z-plugin-card">
	<div class="z-plugin-top">
		<div class="z-plugin-icon">
			{#if isIconifyId}
				<Icon icon={plugin.icon} />
			{:else}
				<img src={iconSrc} alt="" class="z-plugin-icon-img" />
			{/if}
		</div>

		<div class="z-plugin-body">
			<h3 class="z-plugin-name">{plugin.name}</h3>
			<p class="z-plugin-meta">
				{i18nState.locale &&
					m.plugins_meta_versionBy({ version: plugin.version, author: plugin.author })}
			</p>

			<div class="z-plugin-badges">
				<span class="z-plugin-type">{i18nState.locale && typeLabel[plugin.kind]()}</span>
				{#if plugin.builtIn}
					<span class="z-plugin-builtin">{i18nState.locale && m.plugins_badge_builtIn()}</span>
				{/if}
				{#if plugin.dev}
					<span class="z-plugin-dev">Dev</span>
				{/if}
				{#if plugin.enabled}
					<span class="z-plugin-status">{i18nState.locale && m.plugins_badge_installed()}</span>
				{/if}
			</div>
		</div>
	</div>

	<p class="z-plugin-desc">{description}</p>

	<div class="z-plugin-actions">
		{#if plugin.dev}
			<!-- Dev plugins are managed exclusively from the Dev Mode panel
			     above (Reload / Remove). Showing Uninstall here would be
			     misleading — it would unregister from dev mode AND delete the
			     installed-flag bookkeeping, which isn't what the author wants
			     while iterating. -->
		{:else if !plugin.removable}
			<Button variant="ghost" size="sm" disabled>
				{i18nState.locale && m.plugins_action_required()}
			</Button>
		{:else if plugin.enabled}
			<Button variant="danger" size="sm" loading={busy} onclick={onuninstall}>
				{i18nState.locale && m.plugins_action_uninstall()}
			</Button>
		{:else}
			<Button variant="primary" size="sm" loading={busy} onclick={onreinstall}>
				{i18nState.locale && m.plugins_action_install()}
			</Button>
		{/if}
	</div>
</article>

<style>
	.z-plugin-card {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		padding: var(--space-lg);
		background: var(--bg-surface);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-lg);
		transition: all var(--transition-fast);
		height: 100%;
	}

	.z-plugin-top {
		display: flex;
		gap: var(--space-md);
		align-items: flex-start;
		min-width: 0;
	}

	.z-plugin-card:hover {
		border-color: var(--border-default);
	}

	.z-plugin-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 56px;
		height: 56px;
		flex-shrink: 0;
		border-radius: var(--radius-md);
		background: var(--bg-elevated);
		color: var(--text-accent);
		font-size: 28px;
		overflow: hidden;
	}

	.z-plugin-icon-img {
		width: 100%;
		height: 100%;
		object-fit: contain;
	}

	.z-plugin-body {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.z-plugin-name {
		font-family: var(--font-display);
		font-size: 16px;
		font-weight: 700;
		color: var(--text-primary);
		line-height: 1.2;
	}

	.z-plugin-meta {
		font-size: 12px;
		color: var(--text-muted);
	}

	.z-plugin-badges {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		margin-top: 2px;
	}

	.z-plugin-type,
	.z-plugin-builtin,
	.z-plugin-status,
	.z-plugin-dev {
		display: inline-flex;
		align-items: center;
		padding: 2px 8px;
		font-size: 10px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.03em;
		border-radius: var(--radius-sm);
	}

	.z-plugin-dev {
		background: color-mix(in srgb, #a855f7 18%, transparent);
		color: #c084fc;
	}

	.z-plugin-type {
		background: var(--bg-elevated);
		color: var(--text-muted);
	}

	.z-plugin-builtin {
		background: color-mix(in srgb, var(--accent-400) 15%, transparent);
		color: var(--text-accent);
	}

	.z-plugin-status {
		background: color-mix(in srgb, var(--success) 15%, transparent);
		color: var(--success);
	}

	.z-plugin-desc {
		font-size: 13px;
		color: var(--text-secondary);
		line-height: 1.5;
		display: -webkit-box;
		-webkit-line-clamp: 4;
		line-clamp: 4;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}

	.z-plugin-actions {
		display: flex;
		justify-content: flex-end;
		margin-top: var(--space-sm);
	}
</style>
