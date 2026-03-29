<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		children?: Snippet;
	};

	let { children }: Props = $props();
</script>

<h2 class="border-[#1A2A42] text-[#E8ECF1] mt-6 mb-2 border-b pb-2 text-2xl font-bold">
	{@render children?.()}
</h2>
