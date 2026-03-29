<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		children?: Snippet;
	};

	let { children }: Props = $props();
</script>

<h3 class="text-[11px] font-semibold uppercase tracking-[0.1em] text-[#556677] mt-5 mb-2">{@render children?.()}</h3>
