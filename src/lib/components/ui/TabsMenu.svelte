<script lang="ts">
	import { Tabs } from 'bits-ui';
	import type { Snippet } from 'svelte';

	type Props = {
		value: string;
		options: { value: string; label: string }[];
		children?: Snippet;
	};

	let { value = $bindable(), options, children }: Props = $props();
</script>

<Tabs.Root bind:value>
	<Tabs.List class="bg-[#0B1628] text-[#8899AA] my-1 flex gap-1 rounded-xl p-1">
		{#each options as option}
			<Tabs.Trigger
				value={option.value}
				class="hover:bg-[#0F1D32] hover:text-[#E8ECF1] data-[state=active]:bg-[#142240] data-[state=active]:text-[#E8ECF1] grow rounded-lg px-2 py-0.5 data-[state=active]:font-semibold"
			>
				{option.label}
			</Tabs.Trigger>
		{/each}
	</Tabs.List>

	{@render children?.()}
</Tabs.Root>
