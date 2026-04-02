<script lang="ts">
	import * as api from '$lib/api';
	import type { Mod, ModId } from '$lib/types';
	import { shortenFileSize } from '$lib/util';
	import Icon from '@iconify/svelte';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { installState } from '$lib/state/misc.svelte';

	type Props = {
		mod: Mod;
		locked: boolean;
		install: (mod: ModId) => void;
	};

	let { mod, locked, install }: Props = $props();

	let loading = $state(false);
	let downloadSize: number | null = $state(null);

	let disabled = $derived(mod.isInstalled || locked || loading);

	let modId = $derived({
		packageUuid: mod.uuid,
		versionUuid: mod.versionUuid
	});

	$effect(() => {
		loading = false;
		api.profile.install.getDownloadSize(modId).then((size) => (downloadSize = size));
	});

	// Reset loading when install cycle ends
	$effect(() => {
		if (!installState.active) {
			loading = false;
		}
	});
</script>

<div class="z-install-btn-group">
	<button
		class="z-install-main"
		class:installed={mod.isInstalled}
		class:is-locked={locked}
		onclick={() => {
			install(modId);
			loading = true;
		}}
		{disabled}
	>
		{#if locked}
			<Icon icon="mdi:lock" class="text-base" />
			<span>{i18nState.locale && m.installButton_locked()}</span>
		{:else if mod.isInstalled}
			<Icon icon="mdi:check-circle" class="text-base" style="color: var(--success)" />
			<span>{i18nState.locale && m.installButton_installed()}</span>
		{:else if loading}
			<Spinner size={16} />
			<span>{i18nState.locale && m.installButton_installing()}</span>
		{:else}
			<Icon icon="mdi:download" class="text-lg" />
			<span>{i18nState.locale && m.installButton_install()}</span>
			{#if downloadSize}
				<span class="z-install-size">({shortenFileSize(downloadSize)})</span>
			{/if}
		{/if}
	</button>

	<!-- Version dropdown -->
	{#if mod.versions.length > 1 && !locked}
		<details class="z-version-dropdown">
			<summary class="z-version-trigger" class:installed={mod.isInstalled || locked}>
				<Icon icon="mdi:chevron-down" class="text-base" />
			</summary>
			<div class="z-version-list">
				{#each mod.versions as version}
					<button
						class="z-version-item"
						onclick={() => install({ packageUuid: mod.uuid, versionUuid: version.uuid })}
					>
						{version.name}
					</button>
				{/each}
			</div>
		</details>
	{/if}
</div>

<style>
	.z-install-btn-group {
		display: flex;
		align-items: stretch;
		margin-top: var(--space-md);
		font-size: 14px;
	}

	.z-install-main {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-sm);
		padding: var(--space-md) var(--space-lg);
		border-radius: var(--radius-lg) 0 0 var(--radius-lg);
		border: none;
		font-weight: 700;
		font-family: var(--font-body);
		cursor: pointer;
		transition: all var(--transition-normal);
		background: linear-gradient(135deg, var(--accent-400), var(--accent-600));
		color: var(--text-inverse);
		box-shadow: 0 0 12px rgba(26, 255, 250, 0.15);
	}

	.z-install-main:hover:not(:disabled) {
		box-shadow: 0 0 24px rgba(26, 255, 250, 0.3);
		transform: translateY(-1px);
	}

	.z-install-main:disabled {
		cursor: not-allowed;
		opacity: 0.7;
	}

	.z-install-main.installed,
	.z-install-main.is-locked {
		background: var(--bg-overlay);
		color: var(--text-secondary);
		box-shadow: none;
	}

	.z-install-size {
		font-size: 12px;
		font-weight: 400;
		opacity: 0.8;
	}

	/* Version dropdown */
	.z-version-dropdown {
		position: relative;
	}

	.z-version-trigger {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		padding: 0 var(--space-sm);
		border-radius: 0 var(--radius-lg) var(--radius-lg) 0;
		background: linear-gradient(135deg, var(--accent-600), var(--accent-700));
		color: var(--text-inverse);
		cursor: pointer;
		margin-left: 1px;
		transition: all var(--transition-fast);
		list-style: none;
	}

	.z-version-trigger::-webkit-details-marker {
		display: none;
	}

	.z-version-trigger.installed {
		background: var(--bg-overlay);
		color: var(--text-secondary);
	}

	.z-version-trigger:hover {
		background: var(--accent-500);
	}

	.z-version-list {
		position: absolute;
		top: calc(100% + 4px);
		right: 0;
		min-width: 160px;
		max-height: 240px;
		overflow-y: auto;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		padding: var(--space-xs);
		z-index: var(--z-dropdown);
		box-shadow: var(--shadow-lg);
		animation: slideDown var(--transition-fast) ease;
	}

	.z-version-item {
		display: block;
		width: 100%;
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		font-family: var(--font-mono);
		font-size: 12px;
		cursor: pointer;
		text-align: left;
		transition: all var(--transition-fast);
	}

	.z-version-item:hover {
		background: var(--bg-hover);
		color: var(--text-accent);
	}
</style>
