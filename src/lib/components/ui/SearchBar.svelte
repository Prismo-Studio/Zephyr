<script lang="ts">
	import Icon from '@iconify/svelte';

	type Props = {
		value: string;
		placeholder: string;
		brightness?: number;
		size?: 'sm' | 'base' | 'lg';
		onkeydown?: () => void;
	};

	let {
		value = $bindable(),
		placeholder,
		brightness = 900,
		size = 'base',
		onkeydown
	}: Props = $props();
</script>

<input
	type="text"
	class="w-full rounded-lg border border-[#1A2A42] bg-[#0B1628] px-10 py-2 text-[#E8ECF1] placeholder-[#556677] truncate hover:border-[#2D8CF0]/50 focus:border-[#2D8CF0] focus:ring-1 focus:ring-[#2D8CF0] focus:outline-hidden transition-colors text-{size}"
	bind:value
	{onkeydown}
	{placeholder}
/>
<Icon class="text-[#556677] absolute top-2.5 left-2.5 text-xl" icon="mdi:magnify" />

<button
	class="text-[#556677] hover:bg-[#142240] hover:text-[#E8ECF1] absolute top-[0.45rem] right-1.5 rounded-md p-1 text-xl transition-colors"
	onclick={() => (value = '')}
>
	<Icon icon="mdi:close" />
</button>
