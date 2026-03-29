<script lang="ts">
	import { Menubar } from 'bits-ui';

	type Props = {
		text: string;
		key?: string | undefined;
		onclick: () => void;
	};

	let { text, key, onclick }: Props = $props();
</script>

<Menubar.Item
	class={[
		key ? 'pr-4' : 'pr-10',
		'group hover:bg-[#142240] mx-1 flex cursor-default items-center justify-between rounded py-1 pl-4 text-left'
	]}
	{onclick}
>
	<span class="text-[#556677] group-hover:text-[#8899AA]">{text}</span>

	{#if key}
		<span class="text-[#556677] group-hover:text-[#8899AA] pl-6 font-mono text-sm">{key}</span>
	{/if}
</Menubar.Item>
