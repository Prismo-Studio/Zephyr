<script lang="ts">
	import Modal from '$lib/components/ui/Modal.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import ModCard from '$lib/components/mod-list/ModCard.svelte';
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import type { Mod } from '$lib/types';
	import * as api from '$lib/api';
	import { m } from '$lib/paraglide/messages';

	let open = $state(false);
	let mod: Mod | null = $state(null);
	let unlisten: UnlistenFn | undefined;

	onMount(() => {
		listen<Mod>('install_mod', (evt) => {
			mod = evt.payload;
			open = true;
		}).then((cb) => (unlisten = cb));

		return () => unlisten?.();
	});

	async function install() {
		if (!mod || mod.versions.length === 0) return;
		await api.profile.install.mod({
			packageUuid: mod.uuid,
			versionUuid: mod.versions[0].uuid
		});
		open = false;
	}
</script>

<Modal bind:open title={m.installDialog_title()}>
	{#if mod}
		<div class="z-install-dialog">
			<ModCard {mod} showInstallBtn={false} />
			<p class="z-install-confirm">{m.installDialog_confirm()}</p>
		</div>
	{/if}

	{#snippet actions()}
		<Button variant="ghost" onclick={() => (open = false)}>{m.dialog_cancel()}</Button>
		<Button variant="primary" onclick={install}>
			{#snippet icon()}<Icon icon="mdi:download" />{/snippet}
			{m.dialog_install()}
		</Button>
	{/snippet}
</Modal>

<style>
	.z-install-dialog {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg);
	}

	.z-install-confirm {
		font-size: 13px;
		color: var(--text-secondary);
	}
</style>
