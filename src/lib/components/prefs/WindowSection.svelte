<script lang="ts">
	import Icon from '@iconify/svelte';
	import PrefSection from './PrefSection.svelte';
	import PrefRow from './PrefRow.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Toggle from '$lib/components/ui/Toggle.svelte';
	import ShortcutsModal from '$lib/components/dialogs/ShortcutsModal.svelte';
	import { useNativeTitlebar } from '$lib/themeSystem';
	import { fullscreenState, setFullscreen } from '$lib/fullscreen.svelte';
	import { platform } from '@tauri-apps/plugin-os';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	const isMac = platform() === 'macos';
	const fullscreenShortcut = isMac ? '⌘F' : 'F11';

	let showShortcutsModal = $state(false);
</script>

<PrefSection icon="mdi:application" title={(i18nState.locale && m.prefs_window_title()) ?? ''}>
	{#if !isMac}
		<PrefRow
			title={(i18nState.locale && m.prefs_window_nativeTitlebar()) ?? ''}
			description={(i18nState.locale && m.prefs_window_nativeTitlebar_desc()) ?? ''}
		>
			{#snippet control()}
				<Toggle
					checked={useNativeTitlebar.current}
					onchange={(v) => (useNativeTitlebar.current = v)}
				/>
			{/snippet}
		</PrefRow>
	{/if}

	<PrefRow
		title={(i18nState.locale && m.prefs_window_fullscreen()) ?? ''}
		description={(i18nState.locale &&
			m.prefs_window_fullscreen_desc({ shortcut: fullscreenShortcut })) ??
			''}
	>
		{#snippet control()}
			<Toggle checked={fullscreenState.active} onchange={(v) => setFullscreen(v)} />
		{/snippet}
	</PrefRow>

	{#key i18nState.locale}
		<PrefRow title={m.prefs_window_shortcuts()} description={m.prefs_window_shortcuts_desc()}>
			{#snippet control()}
				<Button variant="secondary" size="sm" onclick={() => (showShortcutsModal = true)}>
					{#snippet icon()}<Icon icon="mdi:keyboard" />{/snippet}
					{#snippet children()}{m.prefs_window_shortcuts_button()}{/snippet}
				</Button>
			{/snippet}
		</PrefRow>
	{/key}
</PrefSection>

<ShortcutsModal bind:open={showShortcutsModal} onclose={() => (showShortcutsModal = false)} />
