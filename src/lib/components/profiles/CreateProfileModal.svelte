<script lang="ts">
	import Modal from '$lib/components/ui/Modal.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		open: boolean;
		oncreate: (name: string) => Promise<void>;
	};

	let { open = $bindable(), oncreate }: Props = $props();

	let name = $state('');

	async function submit() {
		if (!name.trim()) return;
		await oncreate(name.trim());
		name = '';
	}

	function close() {
		open = false;
		name = '';
	}
</script>

<Modal bind:open title={i18nState.locale && m.profiles_newProfile()}>
	<div class="z-modal-form">
		<Input
			bind:value={name}
			placeholder={i18nState.locale && m.profiles_profileName()}
			onkeydown={(e) => {
				if (e.key === 'Enter') submit();
			}}
		/>
	</div>

	{#snippet actions()}
		<Button variant="ghost" onclick={close}>{i18nState.locale && m.dialog_cancel()}</Button>
		<Button variant="primary" onclick={submit} disabled={!name.trim()}>
			{i18nState.locale && m.profiles_create()}
		</Button>
	{/snippet}
</Modal>

<style>
	.z-modal-form {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}
</style>
