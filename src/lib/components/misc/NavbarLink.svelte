<script lang="ts">
	import Icon from '@iconify/svelte';
	import { page } from '$app/state';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';

	type Props = {
		to: string;
		icon: string;
		label: string;
		tooltip: string;
	};

	let { to, icon, label, tooltip }: Props = $props();

	let active = $derived(page.url.pathname === to);
</script>

<Tooltip text={tooltip} side="right">
	<a
		href={to}
		class="zephyr-nav-link group relative flex flex-col items-center gap-0.5 rounded-xl px-2 py-2 transition-all duration-200"
		class:active
	>
		<!-- Active indicator bar -->
		{#if active}
			<div class="absolute left-0 top-1/2 -translate-y-1/2 h-7 w-[3px] rounded-r-full bg-gradient-to-b from-[#2D8CF0] to-[#00D4AA]"></div>
		{/if}

		<Icon
			class="text-[1.4rem] transition-colors duration-200"
			{icon}
		/>
		<span class="text-[10px] font-medium leading-tight transition-colors duration-200">{label}</span>
	</a>
</Tooltip>

<style>
	.zephyr-nav-link {
		color: #556677;
		min-width: 52px;
	}

	.zephyr-nav-link:hover {
		color: #8899AA;
		background: rgba(45, 140, 240, 0.06);
	}

	.zephyr-nav-link.active {
		color: #E8ECF1;
		background: rgba(45, 140, 240, 0.1);
	}

	.zephyr-nav-link.active :global(svg) {
		color: #2D8CF0;
		filter: drop-shadow(0 0 6px rgba(45, 140, 240, 0.3));
	}
</style>
