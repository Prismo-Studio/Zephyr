<script lang="ts">
	import type { HTMLTextareaAttributes } from 'svelte/elements';

	type Props = {
		value: string;
		mono?: boolean;
	} & HTMLTextareaAttributes;

	let { value = $bindable(), mono = false, ...restProps }: Props = $props();
</script>

<textarea
	bind:value
	{...restProps}
	autocomplete="off"
	class="valid:focus:ring-[#2D8CF0] bg-[#0B1628] text-[#8899AA] placeholder-[#556677] hover:border-[#1A2A42] h-40 min-h-11 w-full rounded-lg border border-transparent px-3 py-2 invalid:ring-red-500 focus:border-transparent focus:ring-2 focus:outline-hidden"
	class:font-mono={mono}
></textarea>
