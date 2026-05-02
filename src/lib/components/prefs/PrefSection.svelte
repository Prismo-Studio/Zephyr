<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { Snippet } from 'svelte';

	type Props = {
		icon: string;
		title: string;
		children: Snippet;
		extraClass?: string;
		headingClass?: string;
	};

	let { icon, title, children, extraClass = '', headingClass = '' }: Props = $props();
</script>

<section class="z-settings-section {extraClass}">
	<h3 class="z-settings-heading {headingClass}">
		<Icon {icon} />
		{title}
	</h3>
	{@render children()}
</section>

<style>
	.z-settings-section {
		margin-bottom: var(--space-3xl);
	}

	.z-settings-heading {
		font-family: var(--font-display);
		font-size: 16px;
		font-weight: 700;
		color: var(--text-primary);
		display: flex;
		align-items: center;
		gap: var(--space-md);
		margin-bottom: var(--space-xl);
		padding-bottom: var(--space-sm);
		border-bottom: 1px solid var(--border-subtle);
	}
</style>
