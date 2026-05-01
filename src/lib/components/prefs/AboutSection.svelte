<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import updates from '$lib/state/update.svelte';
	import { onMount } from 'svelte';
	import { getVersion } from '@tauri-apps/api/app';
	import { pushInfoToast } from '$lib/toast.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	let appVersion = $state('');

	onMount(async () => {
		appVersion = await getVersion();
	});

	async function checkForUpdates() {
		await updates.refresh(true);
		if (!updates.next?.available) {
			pushInfoToast({ message: m.aboutDialog_latestVersion() });
		}
	}
</script>

<section class="z-settings-section z-about">
	<div class="z-about-brand">
		<span class="z-about-name text-gradient">Zephyr</span>
		<span class="z-about-version">v{appVersion}</span>
	</div>
	<p class="z-about-desc">{m.prefs_aboutDesc()}</p>
	<div class="z-about-actions">
		<Button variant="secondary" size="sm" onclick={checkForUpdates} disabled={updates.isChecking}>
			{#snippet icon()}
				<Icon
					icon={updates.isChecking ? 'mdi:loading' : 'mdi:update'}
					class={updates.isChecking ? 'z-spin' : ''}
				/>
			{/snippet}
			{i18nState.locale && m.aboutDialog_checkUpdate()}
		</Button>
	</div>
</section>

<style>
	.z-about {
		text-align: center;
		padding-top: var(--space-xl);
		margin-bottom: var(--space-3xl);
	}

	.z-about-brand {
		display: flex;
		align-items: baseline;
		justify-content: center;
		gap: var(--space-sm);
	}

	.z-about-name {
		font-family: var(--font-display);
		font-size: 28px;
		font-weight: 800;
	}

	.z-about-version {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
	}

	.z-about-desc {
		font-size: 13px;
		color: var(--text-muted);
		margin-top: var(--space-sm);
	}

	.z-about-actions {
		display: flex;
		justify-content: center;
		margin-top: var(--space-lg);
	}

	:global(.z-spin) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
</style>
