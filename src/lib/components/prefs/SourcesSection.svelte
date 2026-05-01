<script lang="ts">
	import Icon from '@iconify/svelte';
	import PrefSection from './PrefSection.svelte';
	import PrefRow from './PrefRow.svelte';
	import Toggle from '$lib/components/ui/Toggle.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import { curseForgeEnabled } from '$lib/themeSystem';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

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
