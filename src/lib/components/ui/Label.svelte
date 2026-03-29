<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		children?: Snippet;
	};

	let { children }: Props = $props();
</script>

<label class="text-[#E8ECF1] w-[35%] min-w-52 cursor-default truncate text-left font-medium">
	{@render children?.()}
</label>
