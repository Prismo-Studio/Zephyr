<script lang="ts">
	import type { HTMLAnchorAttributes } from 'svelte/elements';

	type Props = Omit<HTMLAnchorAttributes, 'target' | 'rel'>;

	let { class: classProp, children, ...props }: Props = $props();
</script>

<a
	target="_blank"
	rel="noopener noreferrer"
	class={[classProp, 'text-[#2D8CF0] hover:text-[#00D4AA] cursor-pointer hover:underline']}
	{...props}
>
	{@render children?.()}
</a>
