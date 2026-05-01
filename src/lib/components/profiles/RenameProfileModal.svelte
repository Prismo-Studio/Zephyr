<script lang="ts">
	import Modal from '$lib/components/ui/Modal.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		open: boolean;
		initialName: string;
		onrename: (name: string) => Promise<void>;
		onclose: () => void;
	};

	let { open, initialName, onrename, onclose }: Props = $props();

	let name = $state(initialName);

	$effect(() => {
		if (open) name = initialName;
	});

	async function submit() {
		if (!name.trim()) return;
		await onrename(name.trim());
	}
</script>

<Modal {open} {onclose} title={i18nState.locale && m.profiles_renameProfile()}>
	<div class="z-modal-form">
		<Input
			bind:value={name}
			placeholder={i18nState.locale && m.profiles_newName()}
			onkeydown={(e) => {
				if (e.key === 'Enter') submit();
			}}
		/>
	</div>

	{#snippet actions()}
		<Button variant="ghost" onclick={onclose}>{i18nState.locale && m.dialog_cancel()}</Button>
		<Button variant="primary" onclick={submit} disabled={!name.trim()}>
			{i18nState.locale && m.profiles_rename()}
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
