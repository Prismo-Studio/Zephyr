<script lang="ts">
	import Icon from '@iconify/svelte';
	import PrefSection from './PrefSection.svelte';
	import PrefRow from './PrefRow.svelte';
	import Toggle from '$lib/components/ui/Toggle.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import { curseForgeEnabled, zephyrModsEnabled } from '$lib/themeSystem';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { open as openExternal } from '@tauri-apps/plugin-shell';

	let showCurseForgeModal = $state(false);

	function onCurseForgeToggle() {
		if (curseForgeEnabled.current) {
			curseForgeEnabled.current = false;
		} else {
			showCurseForgeModal = true;
		}
	}
</script>

<PrefSection icon="mdi:store-search" title={(i18nState.locale && m.prefs_sources_title()) ?? ''}>
	<PrefRow
		title="Zephyr Mods"
		description={(i18nState.locale && m.prefs_sources_zephyrmods_desc()) ?? ''}
	>
		{#snippet control()}
			<div class="z-source-controls">
				<Tooltip
					text={(i18nState.locale && m.prefs_sources_zephyrmods_submit()) ?? ''}
					position="left"
					delay={150}
				>
					<button
						class="z-source-link"
						onclick={() => openExternal('https://github.com/Prismo-Studio/zephyr-mods')}
						aria-label="Open Zephyr Mods registry"
					>
						<Icon icon="mdi:open-in-new" />
					</button>
				</Tooltip>
				<Toggle
					checked={zephyrModsEnabled.current}
					onchange={() => (zephyrModsEnabled.current = !zephyrModsEnabled.current)}
				/>
			</div>
		{/snippet}
	</PrefRow>

	<PrefRow
		title="CurseForge"
		description={(i18nState.locale && m.prefs_sources_curseforge_desc()) ?? ''}
	>
		{#snippet control()}
			<Toggle checked={curseForgeEnabled.current} onchange={onCurseForgeToggle} />
		{/snippet}
	</PrefRow>
</PrefSection>

{#if showCurseForgeModal}
	<Modal
		bind:open={showCurseForgeModal}
		title="CurseForge"
		onclose={() => (showCurseForgeModal = false)}
	>
		{#snippet children()}
			<div class="z-cf-modal">
				<div class="z-cf-modal-header">
					<img src="/logos/curseforge.png" alt="CurseForge" class="z-cf-modal-logo" />
					<div>
						<p class="z-cf-modal-title">{i18nState.locale && m.prefs_curseforge_modal_title()}</p>
						<p class="z-cf-modal-sub">{i18nState.locale && m.prefs_curseforge_modal_desc()}</p>
					</div>
				</div>
				<p class="z-cf-modal-warning">{i18nState.locale && m.prefs_curseforge_modal_warning()}</p>
			</div>
		{/snippet}
		{#snippet actions()}
			<Button
				variant="primary"
				onclick={() => {
					curseForgeEnabled.current = true;
					showCurseForgeModal = false;
				}}
			>
				{#snippet icon()}<Icon icon="mdi:check" />{/snippet}
				{i18nState.locale && m.prefs_curseforge_modal_confirm()}
			</Button>
		{/snippet}
	</Modal>
{/if}

<style>
	.z-source-controls {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
	}

	.z-source-link {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 30px;
		height: 30px;
		border-radius: var(--radius-md);
		border: 1px solid var(--border-subtle);
		background: var(--bg-elevated);
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
		font-size: 16px;
	}

	.z-source-link:hover {
		background: var(--bg-hover);
		border-color: var(--border-default);
		color: var(--text-accent);
	}

	.z-source-badge {
		display: inline-flex;
		align-items: center;
		padding: 4px 10px;
		border-radius: var(--radius-full);
		background: var(--bg-active);
		border: 1px solid var(--border-accent);
		color: var(--accent-400);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.04em;
		text-transform: uppercase;
	}

	.z-cf-modal {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.z-cf-modal-header {
		display: flex;
		align-items: center;
		gap: var(--space-md);
	}

	.z-cf-modal-logo {
		width: 40px;
		height: 40px;
		border-radius: var(--radius-md);
		flex-shrink: 0;
	}

	.z-cf-modal-title {
		font-size: 14px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.z-cf-modal-sub {
		font-size: 12px;
		color: var(--text-muted);
		margin-top: 2px;
	}

	.z-cf-modal-warning {
		font-size: 13px;
		color: var(--text-secondary);
		line-height: 1.6;
		padding: var(--space-md);
		border-radius: var(--radius-md);
		background: rgba(255, 179, 71, 0.06);
		border: 1px solid rgba(255, 179, 71, 0.15);
	}
</style>
