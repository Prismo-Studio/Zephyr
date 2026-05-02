<script lang="ts">
	import Icon from '@iconify/svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import * as api from '$lib/api';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import type { Mod } from '$lib/types';

	type Props = {
		mod: Mod;
		locked: boolean;
		ontoggle?: () => void;
		onremove?: () => void;
	};

	let { mod, locked, ontoggle, onremove }: Props = $props();
</script>

<div class="z-details-actions">
	<Tooltip
		text={i18nState.locale &&
			(mod.enabled === false ? m.modDetails_enable() : m.modDetails_disable())}
		position="bottom"
		delay={300}
	>
		<button class="z-action-btn" class:disabled={locked} disabled={locked} onclick={ontoggle}>
			<Icon icon={mod.enabled === false ? 'mdi:eye' : 'mdi:eye-off'} />
			<span class="z-action-label">
				{i18nState.locale &&
					(mod.enabled === false ? m.modDetails_enable() : m.modDetails_disable())}
			</span>
		</button>
	</Tooltip>

	<Tooltip text={i18nState.locale && m.modDetails_openFolder()} position="bottom" delay={300}>
		<button class="z-action-btn" onclick={() => api.profile.openModDir(mod.uuid)}>
			<Icon icon="mdi:folder-open" />
			<span class="z-action-label">{i18nState.locale && m.modDetails_openFolder()}</span>
		</button>
	</Tooltip>

	<Tooltip text={i18nState.locale && m.modDetails_uninstall()} position="bottom" delay={300}>
		<button
			class="z-action-btn danger"
			class:disabled={locked}
			disabled={locked}
			onclick={onremove}
		>
			<Icon icon="mdi:delete" />
			<span class="z-action-label">{i18nState.locale && m.modDetails_uninstall()}</span>
		</button>
	</Tooltip>
</div>

<style>
	.z-details-actions {
		display: flex;
		gap: 6px;
	}

	.z-action-btn {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-md);
		border: 1px solid var(--border-subtle);
		background: var(--bg-elevated);
		color: var(--text-secondary);
		font-family: var(--font-body);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all var(--transition-fast);
		white-space: nowrap;
		overflow: hidden;
		min-width: 36px;
	}

	.z-action-label {
		overflow: hidden;
		text-overflow: ellipsis;
	}

	@container (max-width: 380px) {
		.z-action-label {
			display: none;
		}
	}

	.z-action-btn:hover:not(:disabled) {
		background: var(--bg-hover);
		border-color: var(--border-default);
		color: var(--text-primary);
	}

	.z-action-btn.danger:hover:not(:disabled) {
		background: rgba(255, 92, 92, 0.1);
		border-color: rgba(255, 92, 92, 0.3);
		color: var(--error);
	}

	.z-action-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
</style>
