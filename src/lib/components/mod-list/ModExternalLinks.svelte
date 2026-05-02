<script lang="ts">
	import Icon from '@iconify/svelte';
	import { open } from '@tauri-apps/plugin-shell';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import type { Mod } from '$lib/types';

	type Props = {
		mod: Mod;
	};

	let { mod }: Props = $props();

	function isExternal(): boolean {
		return mod.uuid.includes(':');
	}

	function isZephyrSource(): boolean {
		return mod.uuid.startsWith('zephyr:');
	}

	function repoUrl(): string {
		// Zephyr-source URLs include a /releases/download/... suffix; strip it
		// to get the repo root for the secondary "GitHub" link.
		return mod.websiteUrl!.replace(/\/releases\/download\/.*$/, '');
	}
</script>

{#if isZephyrSource() && mod.websiteUrl}
	<div class="z-external-buttons">
		<a
			href={mod.websiteUrl}
			target="_blank"
			rel="noopener"
			class="z-external-link-btn"
			onclick={(e) => {
				e.preventDefault();
				e.stopPropagation();
				open(mod.websiteUrl!);
			}}
		>
			<Icon icon="mdi:download" />
			<span>Download</span>
		</a>
		<a
			href={repoUrl()}
			target="_blank"
			rel="noopener"
			class="z-external-link-btn z-external-link-secondary"
			onclick={(e) => {
				e.preventDefault();
				e.stopPropagation();
				open(repoUrl());
			}}
		>
			<Icon icon="mdi:open-in-new" />
			<span>GitHub</span>
		</a>
	</div>
{:else if isExternal() && mod.websiteUrl}
	<a
		href={mod.websiteUrl}
		target="_blank"
		rel="noopener"
		class="z-external-link-btn"
		onclick={(e) => {
			e.preventDefault();
			e.stopPropagation();
			open(mod.websiteUrl!);
		}}
	>
		<Icon icon="mdi:open-in-new" />
		<span>{i18nState.locale && m.mods_contextMenu_openThunderstore()}</span>
	</a>
{/if}

<style>
	.z-external-link-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-sm);
		padding: var(--space-sm) var(--space-lg);
		border-radius: var(--radius-md);
		background: var(--bg-active);
		color: var(--text-accent);
		font-size: 13px;
		font-weight: 600;
		text-decoration: none;
		cursor: pointer;
		transition: all var(--transition-fast);
		border: 1px solid var(--border-accent);
	}

	.z-external-link-btn:hover {
		background: var(--bg-active);
	}

	.z-external-buttons {
		display: flex;
		gap: 6px;
	}

	.z-external-buttons .z-external-link-btn {
		flex: 1;
	}

	.z-external-link-secondary {
		background: var(--bg-overlay);
		color: var(--text-secondary);
		border-color: var(--border-subtle);
	}

	.z-external-link-secondary:hover {
		background: var(--bg-active);
		color: var(--text-primary);
	}
</style>
