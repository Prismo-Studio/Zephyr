<script lang="ts">
	import { Menubar } from 'bits-ui';
	import type { Snippet } from 'svelte';

	type Props = {
		label: string;
		children?: Snippet;
	};

	let { label, children }: Props = $props();
</script>

<Menubar.Menu>
	<Menubar.Trigger
		class="text-[#556677] hover:bg-[#142240] hover:text-[#E8ECF1] mx-[-1px] h-full cursor-default rounded-sm px-3 text-sm"
	>
		{label}
	</Menubar.Trigger>
	<Menubar.Content
		class="border-[#1A2A42] bg-[#0F1D32] z-50 mt-0.5 flex flex-col gap-0.5 rounded-lg border py-1 shadow-xl"
	>
		{@render children?.()}
	</Menubar.Content>
</Menubar.Menu>
