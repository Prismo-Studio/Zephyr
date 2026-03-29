<script lang="ts">
	import { clearToast, pushInfoToast, toasts, type Toast } from '$lib/toast';
	import Icon from '@iconify/svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import clsx from 'clsx';
	import { expoOut } from 'svelte/easing';
	import { fade, slide } from 'svelte/transition';
	import IconButton from '../ui/IconButton.svelte';
	import { m } from '$lib/paraglide/messages';

	async function copyError(toast: Toast) {
		await writeText(`${toast.name ? `${toast.name}: ` : ''}${toast.message}`);
		pushInfoToast({
			message: m.toasts_copyError_message()
		});
	}
</script>

<div class="absolute bottom-0 z-50 flex w-full flex-col items-end gap-1.5 p-3">
	{#each $toasts as toast, i (toast.id)}
		<div
			class="zephyr-toast relative max-w-3xl overflow-hidden rounded-xl shadow-2xl"
			in:slide={{ duration: 200, easing: expoOut }}
			out:fade={{ duration: 100 }}
		>
			<!-- Accent bar -->
			<div
				class={[
					toast.type === 'error' ? 'bg-red-500' : 'bg-gradient-to-b from-[#2D8CF0] to-[#00D4AA]',
					'absolute left-0 h-full w-1'
				]}
			></div>

			<div class="flex items-center p-3 pl-4">
				<Icon
					class={clsx(
						toast.type === 'error' ? 'text-red-400' : 'text-[#00D4AA]',
						'mr-3 shrink-0 text-xl'
					)}
					icon={toast.type === 'error' ? 'mdi:alert-circle' : 'mdi:check-circle'}
				/>

				<div class="mr-4 grow overflow-hidden">
					{#if toast.name}
						<span class="text-[#8899AA] text-sm">{toast.name}:</span>
					{/if}

					<span class="text-[#E8ECF1] text-sm font-medium break-words">{toast.message}</span>
				</div>

				{#if toast.type === 'error'}
					<IconButton
						icon="mdi:content-copy"
						label={m.toasts_button_copy()}
						onclick={() => copyError(toast)}
					/>
				{/if}

				<IconButton
					icon="mdi:close"
					label={m.toasts_button_clear()}
					onclick={() => clearToast(i)}
				/>
			</div>
		</div>
	{/each}
</div>

<style>
	.zephyr-toast {
		background: #142240;
		border: 1px solid #1A2A42;
		backdrop-filter: blur(12px);
	}
</style>
