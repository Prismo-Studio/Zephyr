<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		label: any;
		description: any;
		required?: boolean;
		children?: Snippet;
	};

	let { label, description, required = false, children }: Props = $props();
</script>

<div class="my-2 rounded-xl border border-[#1A2A42] bg-[#0B1628]/50 p-4">
	<div class="mb-1">
		<span class="text-[#E8ECF1] text-base font-semibold">{label}</span>
		{#if required}
			<span class="text-[#2D8CF0] ml-0.5">*</span>
		{/if}
	</div>
	<p class="text-[#556677] pb-2 text-sm">{description}</p>

	{@render children?.()}
</div>
