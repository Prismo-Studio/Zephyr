<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		title: string;
		description?: string;
		extraClass?: string;
		control: Snippet;
	};

	let { title, description, extraClass = '', control }: Props = $props();
</script>

<div class="z-settings-row {extraClass}">
	<div class="z-settings-label">
		<span>{title}</span>
		{#if description}<span class="z-settings-desc">{description}</span>{/if}
	</div>
	{@render control()}
</div>

<style>
	.z-settings-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-lg) 0;
		border-bottom: 1px solid var(--border-subtle);
	}

	.z-settings-label {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.z-settings-label > span:first-child {
		font-size: 15px;
		font-weight: 500;
		color: var(--text-primary);
	}

	.z-settings-desc {
		font-size: 12px;
		color: var(--text-muted);
	}
</style>
