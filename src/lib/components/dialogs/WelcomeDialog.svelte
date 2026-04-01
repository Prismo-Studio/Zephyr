<script lang="ts">
	import Modal from '$lib/components/ui/Modal.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Icon from '@iconify/svelte';
	import * as api from '$lib/api';
	import { onMount } from 'svelte';
	import { m } from '$lib/paraglide/messages';

	let open = $state(false);

	onMount(async () => {
		const firstRun = await api.state.isFirstRun();
		if (firstRun) open = true;
	});
</script>

<Modal bind:open title={m.welcomeDialog_title()}>
	<div class="z-welcome">
		<div class="z-welcome-logo">
			<span class="text-gradient z-welcome-brand">Zephyr</span>
		</div>
		<p class="z-welcome-text">
			{m.prefs_aboutDesc()}
		</p>
	</div>

	{#snippet actions()}
		<Button variant="primary" onclick={() => (open = false)}>{m.dialog_getStarted()}</Button>
	{/snippet}
</Modal>

<style>
	.z-welcome {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-lg);
		padding: var(--space-xl) 0;
		text-align: center;
	}

	.z-welcome-brand {
		font-family: var(--font-display);
		font-size: 36px;
		font-weight: 800;
	}

	.z-welcome-text {
		font-size: 14px;
		color: var(--text-secondary);
		line-height: 1.6;
		max-width: 360px;
	}
</style>
