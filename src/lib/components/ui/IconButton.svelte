<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { ClassValue } from 'clsx';
	import type { MouseEventHandler } from 'svelte/elements';
	import Tooltip from './Tooltip.svelte';

	type Props = {
		icon: string;
		label: string;
		onclick?: MouseEventHandler<HTMLButtonElement>;
		class?: ClassValue;
		color?: 'primary' | 'accent' | 'red';
		showTooltip?: boolean;
	};

	let {
		icon,
		label,
		onclick,
		class: classProp,
		color = 'primary',
		showTooltip = false
	}: Props = $props();

	let colorClasses = $derived(
		{
			primary: 'text-[#556677] hover:bg-[#1A2A42] hover:text-[#8899AA]',
			accent: 'text-[#556677] hover:bg-[#2D8CF0] hover:text-[#00D4AA]',
			red: 'text-[#556677] hover:bg-red-800 hover:text-red-300'
		}[color]
	);
</script>

{#snippet button()}
	<button class={[classProp, colorClasses, 'shrink-0 rounded-sm p-1']} aria-label={label} {onclick}>
		<Icon {icon} />
	</button>
{/snippet}

{#if showTooltip}
	<Tooltip text={label}>
		{@render button()}
	</Tooltip>
{:else}
	{@render button()}
{/if}
