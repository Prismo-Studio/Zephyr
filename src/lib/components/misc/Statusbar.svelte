<script lang="ts">
	import Icon from '@iconify/svelte';
	import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';
	import { expoOut } from 'svelte/easing';
	import { slide } from 'svelte/transition';

	let status: string | null = $state(null);

	onMount(() => {
		listen<string | null>('status_update', (evt) => {
			status = evt.payload;
		});
	});
</script>

{#if status !== null}
	<div
		class="zephyr-statusbar flex w-full items-center px-4 py-1.5"
		transition:slide={{ duration: 200, easing: expoOut }}
	>
		<div class="zephyr-spinner mr-2"></div>
		<span class="text-xs font-medium text-[#8899AA]">{status}</span>
	</div>
{/if}

<style>
	.zephyr-statusbar {
		background: #080F1C;
		border-top: 1px solid #1A2A42;
	}

	.zephyr-spinner {
		width: 12px;
		height: 12px;
		border: 2px solid #1A2A42;
		border-top-color: #2D8CF0;
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}
</style>
