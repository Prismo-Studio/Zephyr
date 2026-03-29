<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';
	import * as api from '$lib/api';
	import type { SourceInfo } from '$lib/api/sources';
	import type { ManagedGameInfo } from '$lib/types';
	import games from '$lib/state/game.svelte';
	import { gameIconSrc } from '$lib/util';
	import { m } from '$lib/paraglide/messages';

	let sources = $state<SourceInfo[]>([]);
	let gameInfo = $state<ManagedGameInfo | null>(null);

	let modCount = $derived(
		gameInfo?.profiles.find((p: { id: number }) => p.id === gameInfo?.activeId)?.modCount ?? 0
	);
	let profileCount = $derived(gameInfo?.profiles.length ?? 0);
	let gameName = $derived(games.active?.name ?? m.dashboard_noGame());
	let gameIcon = $derived(games.active ? gameIconSrc(games.active) : '');

	onMount(async () => {
		try {
			sources = await api.sources.getSources();
		} catch {
			sources = [];
		}

		try {
			gameInfo = await api.profile.getInfo();
		} catch {
			gameInfo = null;
		}
	});
</script>

<div class="zephyr-dashboard h-full overflow-y-auto p-6">
	<!-- Header -->
	<div class="mb-8 flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold text-[#E8ECF1]">{m.dashboard_title()}</h1>
			<p class="mt-1 text-sm text-[#8899AA]">{m.dashboard_subtitle()}</p>
		</div>

		{#if games.active}
			<div class="flex items-center gap-3 rounded-xl border border-[#1A2A42] bg-[#0B1628] px-4 py-2.5">
				<img src={gameIcon} alt={gameName} class="size-8 rounded" />
				<div>
					<div class="text-sm font-semibold text-[#E8ECF1]">{gameName}</div>
					<div class="text-xs text-[#556677]">{m.dashboard_activeGame()}</div>
				</div>
			</div>
		{/if}
	</div>

	<!-- Stats Grid -->
	<div class="mb-8 grid grid-cols-3 gap-4">
		<a href="/" class="zephyr-stat-card">
			<div class="flex items-center justify-between">
				<div class="rounded-lg bg-[#2D8CF0]/10 p-2">
					<Icon icon="mdi:package-variant" class="text-xl text-[#2D8CF0]" />
				</div>
				<Icon icon="mdi:arrow-right" class="text-[#1A2A42] group-hover:text-[#556677] text-lg transition-colors" />
			</div>
			<div class="mt-3 text-left">
				<div class="text-2xl font-bold text-[#E8ECF1]">{modCount}</div>
				<div class="text-xs text-[#8899AA]">{m.dashboard_stats_modsInstalled()}</div>
			</div>
		</a>

		<div class="zephyr-stat-card">
			<div class="flex items-center justify-between">
				<div class="rounded-lg bg-[#00D4AA]/10 p-2">
					<Icon icon="mdi:folder-multiple" class="text-xl text-[#00D4AA]" />
				</div>
			</div>
			<div class="mt-3">
				<div class="text-2xl font-bold text-[#E8ECF1]">{profileCount}</div>
				<div class="text-xs text-[#8899AA]">{m.dashboard_stats_profiles()}</div>
			</div>
		</div>

		<a href="/browse" class="zephyr-stat-card">
			<div class="flex items-center justify-between">
				<div class="rounded-lg bg-[#FFAA00]/10 p-2">
					<Icon icon="mdi:store-search" class="text-xl text-[#FFAA00]" />
				</div>
				<Icon icon="mdi:arrow-right" class="text-[#1A2A42] group-hover:text-[#556677] text-lg transition-colors" />
			</div>
			<div class="mt-3 text-left">
				<div class="text-2xl font-bold text-[#E8ECF1]">{m.dashboard_stats_browse()}</div>
				<div class="text-xs text-[#8899AA]">{m.dashboard_stats_findMods()}</div>
			</div>
		</a>
	</div>

	<!-- Quick Actions -->
	<div class="mb-8">
		<h2 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[#556677]">{m.dashboard_quickActions_title()}</h2>
		<div class="grid grid-cols-4 gap-3">
			<a href="/browse" class="zephyr-action-card">
				<Icon icon="mdi:store-search-outline" class="text-2xl text-[#2D8CF0]" />
				<span class="mt-2 text-sm font-medium text-[#E8ECF1]">{m.dashboard_quickActions_browse()}</span>
				<span class="text-xs text-[#556677]">{m.dashboard_quickActions_browse_desc()}</span>
			</a>

			<a href="/" class="zephyr-action-card">
				<Icon icon="mdi:view-dashboard" class="text-2xl text-[#00D4AA]" />
				<span class="mt-2 text-sm font-medium text-[#E8ECF1]">{m.dashboard_quickActions_myMods()}</span>
				<span class="text-xs text-[#556677]">{m.dashboard_quickActions_myMods_desc()}</span>
			</a>

			<a href="/config" class="zephyr-action-card">
				<Icon icon="mdi:tune-variant" class="text-2xl text-[#FFAA00]" />
				<span class="mt-2 text-sm font-medium text-[#E8ECF1]">{m.dashboard_quickActions_config()}</span>
				<span class="text-xs text-[#556677]">{m.dashboard_quickActions_config_desc()}</span>
			</a>

			<a href="/modpack" class="zephyr-action-card">
				<Icon icon="mdi:package-variant-closed" class="text-2xl text-[#FF4757]" />
				<span class="mt-2 text-sm font-medium text-[#E8ECF1]">{m.dashboard_quickActions_export()}</span>
				<span class="text-xs text-[#556677]">{m.dashboard_quickActions_export_desc()}</span>
			</a>
		</div>
	</div>

	<!-- Sources Status -->
	<div class="mb-8">
		<h2 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[#556677]">{m.dashboard_sources_title()}</h2>
		<div class="grid grid-cols-2 gap-3">
			{#each sources as source}
				<div class="zephyr-source-card">
					<div class="flex items-center gap-3">
						<div class="rounded-lg p-2" style:background={source.isEnabled ? 'rgba(45, 140, 240, 0.1)' : '#1A2A42'}>
							<Icon
								icon={source.id === 'thunderstore' ? 'mdi:store' :
									source.id === 'nexusmods' ? 'mdi:hexagon-multiple' :
									source.id === 'curseforge' ? 'mdi:fire' :
									'mdi:github'}
								class="text-lg {source.isEnabled ? 'text-[#2D8CF0]' : 'text-[#556677]'}"
							/>
						</div>
						<div class="grow">
							<div class="text-sm font-medium text-[#E8ECF1]">{source.displayName}</div>
							<div class="text-xs text-[#556677]">
								{#if source.isEnabled}
									{#if source.requiresAuth && !source.isAuthenticated}
										<span class="text-[#FFAA00]">{m.dashboard_sources_needsApiKey()}</span>
									{:else}
										<span class="text-[#00D4AA]">{m.dashboard_sources_connected()}</span>
									{/if}
								{:else}
									{m.dashboard_sources_comingSoon()}
								{/if}
							</div>
						</div>
						<div
							class="size-2 rounded-full"
							style:background={!source.isEnabled ? '#556677' : source.isAuthenticated ? '#00D4AA' : '#FFAA00'}
						></div>
					</div>
				</div>
			{/each}
		</div>
	</div>

	<!-- Profiles List -->
	{#if gameInfo && gameInfo.profiles.length > 0}
		<div class="mb-8">
			<h2 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[#556677]">{m.dashboard_profiles_title()}</h2>
			<div class="space-y-2">
				{#each gameInfo.profiles as profile}
					<div
						class="zephyr-profile-item flex items-center gap-3 {profile.id === gameInfo?.activeId ? 'active' : ''}"
					>
						<div class="rounded-lg bg-[#142240] p-2">
							<Icon icon="mdi:folder-account" class="text-lg text-[#8899AA]" />
						</div>
						<div class="grow">
							<div class="text-sm font-medium text-[#E8ECF1]">{profile.name}</div>
							<div class="text-xs text-[#556677]">{m.dashboard_profiles_modCount({ count: profile.modCount })}</div>
						</div>
						{#if profile.id === gameInfo?.activeId}
							<span class="rounded-md bg-[#2D8CF0]/15 px-2 py-0.5 text-xs font-medium text-[#2D8CF0]">{m.dashboard_profiles_active()}</span>
						{/if}
					</div>
				{/each}
			</div>
		</div>
	{/if}

	<!-- Footer -->
	<div class="mt-4 pb-4 text-center">
		<p class="text-xs text-[#556677]">
			{m.dashboard_footer()} <a href="https://prismo-studio.com" target="_blank" class="text-[#2D8CF0] hover:text-[#00D4AA] transition-colors">Prismo Studio</a>
		</p>
	</div>
</div>

<style>
	.zephyr-dashboard {
		scrollbar-width: thin;
		scrollbar-color: #1A2A42 transparent;
	}

	.zephyr-stat-card {
		display: block;
		background: #0B1628;
		border: 1px solid #1A2A42;
		border-radius: 12px;
		padding: 1.25rem;
		transition: all 0.2s ease;
		text-align: left;
		text-decoration: none;
		color: inherit;
	}

	.zephyr-stat-card:hover {
		border-color: #2D8CF0;
		box-shadow: 0 0 20px rgba(45, 140, 240, 0.08);
	}

	.zephyr-action-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		background: #0B1628;
		border: 1px solid #1A2A42;
		border-radius: 12px;
		padding: 1.5rem 1rem;
		transition: all 0.2s ease;
		text-align: center;
		text-decoration: none;
		color: inherit;
	}

	.zephyr-action-card:hover {
		border-color: #2D8CF0;
		background: rgba(45, 140, 240, 0.05);
		transform: translateY(-2px);
	}

	.zephyr-source-card {
		background: #0B1628;
		border: 1px solid #1A2A42;
		border-radius: 10px;
		padding: 0.875rem;
		transition: all 0.2s ease;
	}

	.zephyr-source-card:hover {
		border-color: #1A2A42;
		background: #0D1A30;
	}

	.zephyr-profile-item {
		background: #0B1628;
		border: 1px solid #1A2A42;
		border-radius: 10px;
		padding: 0.75rem;
		transition: all 0.15s ease;
	}

	.zephyr-profile-item:hover {
		background: #0D1A30;
	}

	.zephyr-profile-item.active {
		border-color: rgba(45, 140, 240, 0.3);
		background: rgba(45, 140, 240, 0.05);
	}
</style>
