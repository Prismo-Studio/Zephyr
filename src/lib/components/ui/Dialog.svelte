<script lang="ts">
	import { Dialog } from 'bits-ui';
	import { fade, fly } from 'svelte/transition';
	import Icon from '@iconify/svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';

	import { expoOut, linear } from 'svelte/easing';
	import type { Snippet } from 'svelte';

	type Props = {
		open: boolean;
		title?: string | null;
		confirmClose?: { message: string } | null;
		canClose?: boolean;
		large?: boolean;
		onclose?: () => void;
		children?: Snippet;
	};

	let {
		open = $bindable(),
		title = null,
		confirmClose = null,
		canClose = true,
		large = false,
		onclose,
		children
	}: Props = $props();

	async function close(evt: UIEvent) {
		if (!canClose) {
			evt.preventDefault();
			return;
		}

		if (confirmClose) {
			evt.preventDefault();
			let result = await confirm(confirmClose.message);
			if (!result) return;
		}

		open = false;
		onclose?.();
	}
</script>

<Dialog.Root
	bind:open
	onOpenChange={(open) => {
		if (!open) onclose?.();
	}}
>
	<Dialog.Portal>
		<Dialog.Overlay forceMount class="pointer-events-none" data-tauri-drag-region={!canClose}>
			{#snippet child({ props, open })}
				{#if open}
					<div
						{...props}
						transition:fade={{ duration: 80 }}
						class="fixed inset-0 z-40 bg-black/60"
					></div>
				{/if}
			{/snippet}
		</Dialog.Overlay>
		<Dialog.Content
			interactOutsideBehavior={canClose && confirmClose === null ? 'close' : 'ignore'}
			class="zephyr-dialog-content"
		>
			{#if open}
				<div
					class="zephyr-dialog-center"
					in:fly={{ duration: 200, easing: expoOut, y: 8 }}
					out:fly={{ duration: 50, easing: linear, y: 5 }}
				>
					<div
						class="zephyr-dialog-box"
						class:zephyr-dialog-large={large}
					>
						{#if title}
							<Dialog.Title class="w-full pr-10 text-2xl font-bold wrap-break-word text-white"
								>{title}</Dialog.Title
							>
						{/if}

						{@render children?.()}

						{#if canClose}
							<button
								class="text-[#556677] hover:bg-[#142240] hover:text-[#E8ECF1] absolute top-6 right-6 rounded-lg p-1 text-2xl transition-colors"
								onclick={close}
							>
								<Icon icon="mdi:close" />
							</button>
						{/if}
					</div>
				</div>
			{/if}
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>

<style>
	:global(.zephyr-dialog-content) {
		position: fixed !important;
		inset: 0 !important;
		z-index: 50 !important;
		display: flex !important;
		align-items: center !important;
		justify-content: center !important;
		pointer-events: none !important;
		transform: none !important;
		top: 0 !important;
		left: 0 !important;
		width: 100vw !important;
		height: 100vh !important;
	}

	.zephyr-dialog-center {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 100%;
		height: 100%;
		pointer-events: none;
	}

	.zephyr-dialog-box {
		pointer-events: auto;
		position: relative;
		z-index: 60;
		min-width: 500px;
		max-width: 700px;
		width: 55vw;
		max-height: 85vh;
		overflow-x: hidden;
		overflow-y: auto;
		border-radius: 16px;
		border: 1px solid #1A2A42;
		background: #0F1D32;
		padding: 2rem;
		box-shadow: 0 30px 80px rgba(0, 0, 0, 0.6), 0 0 1px rgba(45, 140, 240, 0.1);

		scrollbar-width: thin;
		scrollbar-color: #1A2A42 #0F1D32;
	}

	.zephyr-dialog-box.zephyr-dialog-large {
		min-width: 600px;
		max-width: 900px;
		width: 70vw;
	}
</style>
