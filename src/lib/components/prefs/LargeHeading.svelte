<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		children?: Snippet;
	};

	let { children }: Props = $props();
</script>

<h2 class="zephyr-large-heading mt-6 mb-3 pb-3 text-xl font-bold text-[#E8ECF1]">
	{@render children?.()}
</h2>

<style>
	.zephyr-large-heading {
		border-bottom: 1px solid #1A2A42;
		background: linear-gradient(90deg, rgba(45, 140, 240, 0.04) 0%, transparent 60%);
		margin-left: -0.75rem;
		margin-right: -0.75rem;
		padding-left: 0.75rem;
		padding-right: 0.75rem;
		border-radius: 8px 8px 0 0;
	}
</style>
