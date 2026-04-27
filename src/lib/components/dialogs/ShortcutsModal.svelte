<script lang="ts">
	import Modal from '$lib/components/ui/Modal.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { platform } from '@tauri-apps/plugin-os';

	type Props = {
		open?: boolean;
		onclose?: () => void;
	};

	let { open = $bindable(false), onclose }: Props = $props();

	const isMac = platform() === 'macos';
	const ctrl = isMac ? '⌘' : 'Ctrl';
	const shift = isMac ? '⇧' : 'Shift';

	type Shortcut = { keys: string[]; label: () => string };
	type Group = { title: () => string; items: Shortcut[] };

	const groups: Group[] = [
		{
			title: () => m.shortcuts_group_general(),
			items: [
				{
					keys: [isMac ? '⌘F' : 'F11'],
					label: () => m.shortcuts_toggleFullscreen()
				},
				{
					keys: [`${ctrl}+R`],
					label: () => m.shortcuts_refreshData()
				},
				{
					keys: [`${ctrl}+F`],
					label: () => m.shortcuts_globalSearch()
				}
			]
		},
		{
			title: () => m.shortcuts_group_profiles(),
			items: [
				{
					keys: [`${ctrl} + "←"`, `${ctrl} + "→"`],
					label: () => m.shortcuts_cycleProfiles()
				}
			]
		},
		{
			title: () => m.shortcuts_group_display(),
			items: [
				{
					keys: [`${ctrl} + "+"`, `${ctrl} + "wheel up"`],
					label: () => m.shortcuts_zoomIn()
				},
				{
					keys: [`${ctrl} + "-"`, `${ctrl} + "wheel down"`],
					label: () => m.shortcuts_zoomOut()
				}
			]
		},
		{
			title: () => m.shortcuts_group_randomizer(),
			items: [
				{
					keys: [`${ctrl}+B`],
					label: () => m.shortcuts_toggleMultiplayer()
				}
			]
		},
		{
			title: () => m.shortcuts_group_console(),
			items: [
				{
					keys: [`${ctrl}+${shift}+S`],
					label: () => m.shortcuts_consoleServer()
				},
				{
					keys: [`${ctrl}+${shift}+C`],
					label: () => m.shortcuts_consoleClient()
				},
				{
					keys: [`${ctrl}+${shift}+T`],
					label: () => m.shortcuts_consoleToggle()
				}
			]
		}
	];
</script>

{#if i18nState.locale}
	<Modal bind:open title={m.shortcuts_modalTitle()} {onclose}>
		{#snippet children()}
			<div class="z-shortcuts">
				<p class="z-shortcuts-intro">{m.shortcuts_intro()}</p>
				{#each groups as group}
					<section class="z-shortcuts-group">
						<h4 class="z-shortcuts-group-title">{group.title()}</h4>
						<ul class="z-shortcuts-list">
							{#each group.items as item}
								<li class="z-shortcuts-row">
									<span class="z-shortcuts-label">{item.label()}</span>
									<span class="z-shortcuts-keys">
										{#each item.keys as key, i}
											{#if i > 0}<span class="z-shortcuts-sep">/</span>{/if}
											<kbd class="z-shortcuts-key">{key}</kbd>
										{/each}
									</span>
								</li>
							{/each}
						</ul>
					</section>
				{/each}
			</div>
		{/snippet}
	</Modal>
{/if}

<style>
	.z-shortcuts {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg);
		min-width: 420px;
		max-width: 560px;
	}

	.z-shortcuts-intro {
		color: var(--text-secondary);
		font-size: 13px;
		margin: 0 0 var(--space-xs);
	}

	.z-shortcuts-group-title {
		margin: 0 0 var(--space-sm);
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--accent-400);
		font-weight: 700;
	}

	.z-shortcuts-list {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
	}

	.z-shortcuts-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 12px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-sm);
	}

	.z-shortcuts-label {
		color: var(--text-primary);
		font-size: 13px;
	}

	.z-shortcuts-keys {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}

	.z-shortcuts-sep {
		color: var(--text-muted);
		font-size: 12px;
	}

	.z-shortcuts-key {
		display: inline-block;
		padding: 2px 8px;
		min-width: 24px;
		text-align: center;
		font-family: var(--font-mono, monospace);
		font-size: 12px;
		color: var(--accent-300, var(--accent-400));
		background: var(--bg-base);
		border: 1px solid var(--accent-400);
		border-radius: var(--radius-sm);
		box-shadow: 0 1px 0 var(--accent-700, rgba(0, 0, 0, 0.2));
	}
</style>
