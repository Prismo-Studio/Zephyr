<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		children?: Snippet;
	};

	let { children }: Props = $props();
</script>

<h3 class="text-sm font-semibold uppercase tracking-wider text-[#556677] mt-4 mb-1">{@render children?.()}</h3>
