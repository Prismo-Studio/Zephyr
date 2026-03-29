<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	type Props = {
		color?: 'accent' | 'primary' | 'red' | 'gradient';
		icon?: string;
		loading?: boolean;
	} & HTMLButtonAttributes;

	let {
		disabled: disabledProp,
		color = 'accent',
		icon,
		loading = false,
		class: classProp,
		children,
		...restProps
	}: Props = $props();

	let disabled = $derived(disabledProp || loading);
	let renderedIcon = $derived(loading ? 'mdi:loading' : icon);
</script>

<button
	class={[
		classProp,
		color === 'gradient'
			? 'zephyr-btn-gradient font-medium text-white'
			: color === 'accent'
				? 'zephyr-btn-accent font-medium text-white'
				: color === 'red'
					? 'zephyr-btn-red text-white'
					: 'zephyr-btn-primary text-[#E8ECF1]',
		'inline-flex items-center overflow-hidden rounded-lg px-4 py-2 text-nowrap text-sm transition-all duration-200',
		'disabled:opacity-50 disabled:cursor-not-allowed'
	]}
	{disabled}
	{...restProps}
>
	{#if renderedIcon}
		<Icon icon={renderedIcon} class="mr-2 text-lg {loading && 'animate-spin'}" />
	{/if}

	{@render children?.()}
</button>

<style>
	.zephyr-btn-gradient {
		background: linear-gradient(135deg, #2D8CF0, #00D4AA);
	}
	.zephyr-btn-gradient:hover:not(:disabled) {
		background: linear-gradient(135deg, #3D9CFF, #10E4BA);
		box-shadow: 0 0 20px rgba(45, 140, 240, 0.3);
	}

	.zephyr-btn-accent {
		background: #2D8CF0;
	}
	.zephyr-btn-accent:hover:not(:disabled) {
		background: #3D9CFF;
		box-shadow: 0 0 16px rgba(45, 140, 240, 0.25);
	}

	.zephyr-btn-primary {
		background: #142240;
		border: 1px solid #1A2A42;
	}
	.zephyr-btn-primary:hover:not(:disabled) {
		background: #1A2A42;
		border-color: #2D8CF0;
	}

	.zephyr-btn-red {
		background: #991B1B;
	}
	.zephyr-btn-red:hover:not(:disabled) {
		background: #B91C1C;
	}
</style>
