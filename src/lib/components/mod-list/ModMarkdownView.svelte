<script lang="ts">
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		markdown: string;
		loading: boolean;
	};

	let { markdown, loading }: Props = $props();
</script>

{#if loading}
	<div class="z-details-loading">
		<Spinner size={20} />
	</div>
{:else if markdown}
	<div class="markdown">
		{@html markdown}
	</div>
{:else}
	<p class="z-details-empty">{i18nState.locale && m.modDetails_noContent()}</p>
{/if}

<style>
	.z-details-loading {
		display: flex;
		justify-content: center;
		padding: var(--space-2xl);
		color: var(--text-muted);
	}

	.z-details-empty {
		text-align: center;
		color: var(--text-muted);
		font-size: 13px;
		padding: var(--space-2xl);
	}
</style>
