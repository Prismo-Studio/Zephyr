<script lang="ts">
	import RandomizerCatalog from '$lib/features/randomizer/RandomizerCatalog.svelte';
	import RandomizerConfigurator from '$lib/features/randomizer/RandomizerConfigurator.svelte';
	import { randomizerStore } from '$lib/features/randomizer/randomizer.store.svelte';

	async function selectGame(gameId: string) {
		await randomizerStore.selectGame(gameId);
	}

	function back() {
		randomizerStore.clearGame();
	}
</script>

<div class="rdz-page">
	{#if randomizerStore.currentSchema}
		<RandomizerConfigurator onBack={back} />
	{:else}
		<RandomizerCatalog onSelect={selectGame} />
	{/if}
</div>

<style>
	.rdz-page {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		height: 100%;
		max-height: 100%;
		overflow: hidden;
		background: var(--bg-base);
		color: var(--text-primary);
	}
</style>
