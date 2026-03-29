<script lang="ts">
	import * as api from '$lib/api';
	import games from '$lib/state/game.svelte';
	import type { Game } from '$lib/types';
	import { gameIconSrc } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { toHeaderCase } from 'js-convert-case';

	type Props = { game: Game; onselect?: () => void; onfavorite?: (favorite: boolean) => void };

	let { game, onselect, onfavorite }: Props = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
	class={[
		games.active?.slug === game.slug
			? 'border-[rgba(45,140,240,0.3)] bg-[#142240] shadow-[0_0_12px_rgba(45,140,240,0.05)]'
			: 'hover:bg-[#142240]/60 border-transparent',
		'group mr-2 flex cursor-pointer items-center rounded-xl border p-2 transition-all duration-150'
	]}
	onclick={() => {
		games.setActive(game.slug);
		onselect?.();
	}}
	role="button"
	tabindex="0"
>
	<img src={gameIconSrc(game)} alt={game.name} class="mr-3 size-11 rounded-lg object-cover" />

	<div class="grow text-left">
		<div class="text-sm font-semibold text-[#E8ECF1]">
			{game.name}
		</div>

		<div class="mt-0.5 text-xs text-[#556677]">
			<span>{game.modLoader}</span>

			{#if game.platforms.length > 0}
				<span class="text-[#3A4A5C] mx-1">&middot;</span>
				<span>{game.platforms.map(toHeaderCase).join(', ')}</span>
			{/if}
		</div>
	</div>

	<button
		class={[
			game.favorite ? 'block' : 'hidden group-hover:block',
			'hover:bg-[#1A2A42] mr-1 rounded-lg p-1.5 transition-colors'
		]}
		onclick={(evt) => {
			evt.stopPropagation();
			onfavorite?.(!game.favorite);
			api.profile.favoriteGame(game.slug);
		}}
	>
		<Icon icon={game.favorite ? 'mdi:star' : 'mdi:star-outline'} class="text-yellow-400 text-lg" />
	</button>
</div>
