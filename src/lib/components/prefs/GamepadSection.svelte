<script lang="ts">
	import Icon from '@iconify/svelte';
	import PrefSection from './PrefSection.svelte';
	import PrefRow from './PrefRow.svelte';
	import Toggle from '$lib/components/ui/Toggle.svelte';
	import type { Prefs } from '$lib/types';
	import { onMount } from 'svelte';
	import {
		setGamepadEnabled,
		getConnectedGamepad,
		getGamepadDisplayName
	} from '$lib/gamepad.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		prefs: Prefs;
		onsave: () => Promise<void>;
	};

	let { prefs, onsave }: Props = $props();

	let gamepadName = $state<string | null>(null);

	function toggleGamepad() {
		prefs.gamepadEnabled = !prefs.gamepadEnabled;
		setGamepadEnabled(prefs.gamepadEnabled);
		onsave();
	}

	onMount(() => {
		const gp = getConnectedGamepad();
		if (gp) gamepadName = getGamepadDisplayName(gp);

		const onConnect = (e: GamepadEvent) => {
			gamepadName = getGamepadDisplayName(e.gamepad.id);
		};
		const onDisconnect = () => {
			gamepadName = null;
		};
		window.addEventListener('gamepadconnected', onConnect);
		window.addEventListener('gamepaddisconnected', onDisconnect);
		return () => {
			window.removeEventListener('gamepadconnected', onConnect);
			window.removeEventListener('gamepaddisconnected', onDisconnect);
		};
	});
</script>

<PrefSection icon="mdi:controller" title={(i18nState.locale && m.prefs_gamepad_title()) ?? ''}>
	<PrefRow
		title={(i18nState.locale && m.prefs_gamepad_enabled_title()) ?? ''}
		description={(i18nState.locale && m.prefs_gamepad_enabled_desc()) ?? ''}
	>
		{#snippet control()}
			<Toggle checked={prefs.gamepadEnabled} onchange={toggleGamepad} />
		{/snippet}
	</PrefRow>

	<div class="z-gamepad-status">
		{#if gamepadName}
			<span class="z-gamepad-connected">
				<Icon icon="mdi:controller" />
				{i18nState.locale && m.prefs_gamepad_connected({ name: gamepadName })}
			</span>
		{:else}
			<span class="z-gamepad-none">
				<Icon icon="mdi:controller-off" />
				{i18nState.locale && m.prefs_gamepad_none()}
			</span>
		{/if}
	</div>
</PrefSection>

<style>
	.z-gamepad-status {
		padding: var(--space-lg) 0;
	}

	.z-gamepad-connected {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-size: 13px;
		color: var(--text-accent);
	}

	.z-gamepad-none {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-size: 13px;
		color: var(--text-muted);
	}
</style>
