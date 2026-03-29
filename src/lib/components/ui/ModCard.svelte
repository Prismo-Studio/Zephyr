<script lang="ts">
	import games from '$lib/state/game.svelte';

	type Props = {
		fullName: string;
		showVersion?: boolean;
	};

	let { fullName, showVersion = true }: Props = $props();

	let split = $derived(fullName.split('-'));

	// kind of hacky, but we assume we either have (1) just a name, (2) name and version, or (3) all three
	let author = $derived(split.length === 3 ? split[0] : null);
	let name = $derived(split.length === 3 ? split[1] : split[0]);
	let version = $derived(split.length >= 2 ? (split.length === 3 ? split[2] : split[1]) : null);
</script>

<div class="zephyr-mod-card flex items-center overflow-hidden rounded-lg p-1.5 transition-colors">
	<img
		src="https://gcdn.thunderstore.io/live/repository/icons/{fullName}.png"
		alt={name}
		class="size-10 shrink-0 rounded-lg object-cover"
	/>
	<div class="shrink overflow-hidden pl-3 text-left">
		<div class="flex items-center gap-2">
			<a
				class="shrink truncate text-sm font-semibold text-[#E8ECF1] hover:text-[#2D8CF0] hover:underline transition-colors"
				href="https://thunderstore.io/c/{games.active?.slug}/p/{author}/{name}/"
				target="_blank"
				rel="noopener noreferrer"
			>
				{name.replace(/_/g, ' ')}
			</a>

			{#if showVersion && version !== null}
				<span class="text-[11px] font-mono text-[#556677] shrink-0 tabular-nums">
					{version}
				</span>
			{/if}
		</div>

		{#if author !== null}
			<a
				class="text-xs text-[#556677] truncate hover:text-[#8899AA] hover:underline transition-colors"
				href="https://thunderstore.io/c/{games.active?.slug}/p/{author}/"
				target="_blank"
			>
				{author}
			</a>
		{/if}
	</div>
</div>

<style>
	.zephyr-mod-card:hover {
		background: rgba(20, 34, 64, 0.4);
	}
</style>
