<script lang="ts">
	import Icon from '@iconify/svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import { clickOutside } from '$lib/utils/clickOutside';
	import { PersistedState } from 'runed';
	import * as api from '$lib/api';
	import { formatModName } from '$lib/util';
	import { pushToast } from '$lib/toast.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import type { Mod } from '$lib/types';

	type Props = {
		mod: Mod;
	};

	let { mod }: Props = $props();

	let dropdownOpen = $state(false);
	let changing = $state(false);
	let confirm: { open: boolean; target: { name: string; uuid: string } | null } = $state({
		open: false,
		target: null
	});
	let dontAskAgain = $state(false);
	const skipConfirm = new PersistedState('skipVersionConfirm', false);

	function isExternal(): boolean {
		return mod.uuid.includes(':');
	}

	async function selectVersion(version: { name: string; uuid: string }) {
		if (changing) return;
		if (version.uuid === mod.versionUuid) {
			dropdownOpen = false;
			return;
		}
		dropdownOpen = false;
		if (skipConfirm.current) {
			await doChange(version);
		} else {
			confirm = { open: true, target: version };
			dontAskAgain = false;
		}
	}

	async function doChange(version: { name: string; uuid: string }) {
		if (changing) return;
		changing = true;
		confirm = { open: false, target: null };
		const packageUuid = mod.uuid;
		const modName = formatModName(mod.name);
		try {
			if (mod.isInstalled && !isExternal()) {
				await api.profile.forceRemoveMods([packageUuid]);
			}
			await api.profile.install.mod({ packageUuid, versionUuid: version.uuid });
			pushToast({
				type: 'info',
				message:
					(i18nState.locale &&
						m.modDetails_versionChange_success({
							mod: modName,
							version: version.name
						})) ||
					`${modName} changed to ${version.name}`
			});
		} catch (err) {
			pushToast({
				type: 'error',
				name: 'Version change failed',
				message: err instanceof Error ? err.message : 'Unknown error'
			});
		} finally {
			changing = false;
		}
	}

	function confirmAndChange() {
		if (!confirm.target || changing) return;
		if (dontAskAgain) skipConfirm.current = true;
		doChange(confirm.target);
	}
</script>

<div class="z-version-selector" use:clickOutside={() => (dropdownOpen = false)}>
	<button
		class="z-version-btn"
		disabled={changing}
		onclick={() => (dropdownOpen = !dropdownOpen)}
	>
		<Icon icon="mdi:tag" />
		<span>{mod.version}</span>
		<Icon
			icon="mdi:chevron-down"
			class="z-version-chevron {dropdownOpen ? 'open' : ''}"
		/>
	</button>

	{#if dropdownOpen}
		<div class="z-version-dropdown">
			{#each mod.versions as version}
				<button
					class="z-version-option"
					class:active={version.name === mod.version}
					onclick={() => selectVersion(version)}
				>
					{#if version.name === mod.version}
						<Icon icon="mdi:check" />
					{/if}
					<span>{version.name}</span>
					{#if version.name === mod.versions[0].name}
						<Badge variant="accent">{i18nState.locale && m.modDetails_latest()}</Badge>
					{/if}
				</button>
			{/each}
		</div>
	{/if}
</div>

{#if confirm.open && confirm.target}
	<Modal
		bind:open={confirm.open}
		title={i18nState.locale && m.modDetails_versionChange_title()}
		onclose={() => (confirm = { open: false, target: null })}
	>
		{#snippet children()}
			<div class="z-version-confirm-body">
				<p>
					{i18nState.locale &&
						m.modDetails_versionChange_desc({
							mod: formatModName(mod.name),
							from: mod.version ?? '',
							to: confirm.target?.name ?? ''
						})}
				</p>
				<label class="z-version-dont-ask">
					<Checkbox bind:checked={dontAskAgain} />
					<span>{i18nState.locale && m.modDetails_versionChange_dontAsk()}</span>
				</label>
			</div>
		{/snippet}

		{#snippet actions()}
			<Button variant="ghost" onclick={() => (confirm = { open: false, target: null })}>
				{i18nState.locale && m.modDetails_versionChange_cancel()}
			</Button>
			<Button variant="primary" onclick={confirmAndChange}>
				{#snippet icon()}<Icon icon="mdi:swap-vertical" />{/snippet}
				{i18nState.locale && m.modDetails_versionChange_confirm()}
			</Button>
		{/snippet}
	</Modal>
{/if}

<style>
	.z-version-selector {
		position: relative;
	}

	.z-version-btn {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 3px 8px;
		border-radius: var(--radius-full);
		border: 1px solid var(--accent-400);
		background: var(--bg-active);
		color: var(--accent-400);
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.z-version-btn:hover {
		background: var(--shadow-glow);
	}

	:global(.z-version-chevron) {
		font-size: 14px;
		transition: transform 150ms ease;
	}

	:global(.z-version-chevron.open) {
		transform: rotate(180deg);
	}

	.z-version-dropdown {
		position: absolute;
		top: 100%;
		left: 0;
		margin-top: 4px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		padding: var(--space-xs);
		min-width: 160px;
		max-height: 240px;
		overflow-y: auto;
		z-index: var(--z-dropdown);
		box-shadow: var(--shadow-lg);
	}

	.z-version-option {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		width: 100%;
		padding: var(--space-xs) var(--space-sm);
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		font-size: 12px;
		cursor: pointer;
		transition: all var(--transition-fast);
		font-family: var(--font-body);
	}

	.z-version-option:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.z-version-option.active {
		color: var(--text-accent);
	}

	.z-version-confirm-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		font-size: 13px;
		color: var(--text-secondary);
		line-height: 1.5;
	}

	.z-version-dont-ask {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-size: 12px;
		color: var(--text-muted);
		cursor: pointer;
	}
</style>
