<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Checkbox } from 'bits-ui';
	import type { ClassValue } from 'clsx';

	type Props = {
		checked?: boolean;
		disabled?: boolean;
		onCheckedChange?: (newValue: boolean) => void;
		class?: ClassValue;
	};

	let {
		checked = $bindable(false),
		disabled = false,
		onCheckedChange,
		class: classProp
	}: Props = $props();

	let stateClasses = $derived(
		checked
			? [!disabled && 'hover:bg-[#2D8CF0]/80', 'bg-[#2D8CF0]']
			: [!disabled && 'hover:bg-[#142240]', 'bg-[#0B1628] border border-[#1A2A42]']
	);
</script>

<Checkbox.Root {disabled} bind:checked {onCheckedChange} class="group">
	<div
		class={[
			classProp,
			stateClasses,
			'size-6 cursor-pointer rounded-md p-1 group-data-[disabled]:cursor-default'
		]}
	>
		{#if checked}
			<Icon class="h-full w-full font-bold text-white" icon="mdi:check" />
		{/if}
	</div>
</Checkbox.Root>
