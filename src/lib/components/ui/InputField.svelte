<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';

	type Props = {
		value?: string;
		size?: 'sm' | 'md' | 'lg';
		class?: string;
		onsubmit?: (value: string) => void;
		onchange?: (value: string) => void;
	} & Omit<HTMLInputAttributes, 'size' | 'onchange'>;

	let {
		value = $bindable(''),
		size = 'md',
		class: classProp = '',
		onsubmit,
		onchange,
		spellcheck = false,
		autocomplete = 'off',
		...props
	}: Props = $props();
</script>

<input
	type="text"
	bind:value
	onkeydown={(evt) => {
		if (evt.key === 'Enter') {
			onsubmit?.(value);
		}
	}}
	{autocomplete}
	{spellcheck}
	{...props}
	onchange={() => onchange?.(value)}
	class={[
		classProp,
		`text-${size}`,
		`placeholder:text-${size}`,
		'valid:focus:ring-[#2D8CF0]! disabled:text-[#556677] bg-[#0B1628] text-[#E8ECF1] placeholder-[#556677] enabled:hover:ring-[#1A2A42] min-w-0 grow rounded-lg border border-[#1A2A42] px-3 py-2 invalid:ring-1 invalid:ring-red-500 invalid:hover:ring-2 invalid:hover:ring-red-500 focus:ring-2! focus:border-[#2D8CF0] focus:outline-hidden enabled:hover:ring-1 disabled:cursor-not-allowed transition-colors'
	]}
/>
