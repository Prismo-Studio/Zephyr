<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	type Props = HTMLButtonAttributes;

	let { class: classProp, ...restProps }: Props = $props();
</script>

<button
	class={[
		classProp,
		'text-[#556677] disabled:text-[#3A4A5C] enabled:hover:bg-[#142240] enabled:hover:text-[#8899AA] rounded-md p-1.5 text-lg disabled:cursor-not-allowed'
	]}
	{...restProps}
>
	<Icon icon="mdi:refresh" />
</button>
