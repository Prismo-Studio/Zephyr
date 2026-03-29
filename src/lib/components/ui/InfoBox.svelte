<script lang="ts">
	import Icon from '@iconify/svelte';
	import clsx, { type ClassValue } from 'clsx';
	import type { Snippet } from 'svelte';

	type Props = {
		type?: 'error' | 'warning' | 'info';
		class?: ClassValue;
		children: Snippet;
	};

	let { type = 'info', class: classProp, children }: Props = $props();

	let { icon, bgClass, iconClass } = $derived(
		{
			error: {
				icon: 'mdi:error',
				bgClass: 'bg-red-600',
				iconClass: 'text-red-600'
			},
			warning: {
				icon: 'mdi:warning',
				bgClass: 'bg-yellow-600',
				iconClass: 'text-yellow-600'
			},
			info: {
				icon: 'mdi:info',
				bgClass: 'bg-[#2D8CF0]',
				iconClass: 'text-[#2D8CF0]'
			}
		}[type]
	);
</script>

<div
	class={[
		'bg-[#0F1D32] border-[#1A2A42] relative my-2 overflow-hidden rounded-md border shadow',
		classProp
	]}
>
	<div class={['absolute left-0 h-full w-1.5', bgClass]}></div>

	<div class="ml-2 flex items-center gap-3 p-3">
		<Icon class={clsx('shrink-0 text-xl', iconClass)} {icon} />

		<div class="grow overflow-hidden text-white">
			{@render children()}
		</div>
	</div>
</div>
