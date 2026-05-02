<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import ColorPicker from '$lib/components/ui/ColorPicker.svelte';
	import CustomBgPanel from './CustomBgPanel.svelte';
	import {
		getCustomColors,
		setCustomColors,
		DEFAULT_CUSTOM_COLORS,
		type CustomThemeColors
	} from '$lib/design-system/tokens';

	type Props = {
		open: boolean;
	};

	let { open = $bindable() }: Props = $props();

	let customDraft = $state<CustomThemeColors>(getCustomColors());

	function reset() {
		customDraft = { ...DEFAULT_CUSTOM_COLORS };
	}

	$effect(() => {
		setCustomColors(customDraft);
	});

	const colorRows: { key: keyof CustomThemeColors; label: string; desc: string }[] = [
		{ key: 'accent', label: 'Accent', desc: 'Buttons, links, highlights' },
		{ key: 'bg', label: 'Background', desc: 'Main app background' },
		{ key: 'bgCard', label: 'Cards', desc: 'Surfaces, cards, modals' },
		{ key: 'text', label: 'Text', desc: 'Main foreground color' }
	];
</script>

{#if open}
	<Modal bind:open title="Custom theme" onclose={() => (open = false)}>
		{#snippet children()}
			<div class="z-custom-theme">
				<p class="z-custom-hint">
					Pick four colors. The interface updates live as you change them.
				</p>

				{#each colorRows as row}
					<div class="z-color-row">
						<div class="z-color-label">
							<span class="z-color-name">{row.label}</span>
							<span class="z-color-desc">{row.desc}</span>
						</div>
						<ColorPicker bind:value={customDraft[row.key]} />
					</div>
				{/each}

				<CustomBgPanel />
			</div>
		{/snippet}
		{#snippet actions()}
			<Button variant="ghost" onclick={reset}>Reset</Button>
			<Button variant="primary" onclick={() => (open = false)}>
				{#snippet icon()}<Icon icon="mdi:check" />{/snippet}
				Done
			</Button>
		{/snippet}
	</Modal>
{/if}

<style>
	.z-custom-theme {
		display: flex;
		flex-direction: column;
		gap: 16px;
		min-width: 360px;
	}

	.z-custom-hint {
		font-size: 13px;
		color: var(--text-secondary);
		margin: 0 0 4px;
	}

	.z-color-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		padding: 12px;
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		border: 1px solid var(--border-subtle);
	}

	.z-color-label {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
		flex: 1;
		cursor: pointer;
	}

	.z-color-name {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.z-color-desc {
		font-size: 11px;
		color: var(--text-muted);
	}
</style>
