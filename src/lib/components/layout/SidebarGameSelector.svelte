<script lang="ts">
	import Icon from '@iconify/svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import games from '$lib/state/game.svelte';
	import { gameIconSrc } from '$lib/util';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	let menuOpen = $state(false);
	let searchTerm = $state('');
	let dropdownEl: HTMLDivElement | null = null;
	let searchInput: HTMLInputElement | null = null;

	function normalizeName(s: string): string {
		return s.toLowerCase().replace(/[^a-z0-9]/g, '');
	}

	const filteredGames = $derived.by(() => {
		const q = normalizeName(searchTerm);
		const matches = q
			? games.list.filter((g) => normalizeName(g.name).includes(q))
			: games.list.slice();
		return matches.sort((a, b) => {
			if (a.favorite !== b.favorite) return a.favorite ? -1 : 1;
			return a.name.localeCompare(b.name);
		});
	});

	$effect(() => {
		if (menuOpen && searchInput) searchInput.focus();
	});

	function handleClickOutside(e: MouseEvent) {
		if (menuOpen && dropdownEl && !dropdownEl.contains(e.target as Node)) {
			close();
		}
	}

	$effect(() => {
		if (menuOpen) {
			document.addEventListener('mousedown', handleClickOutside, true);
		} else {
			document.removeEventListener('mousedown', handleClickOutside, true);
		}
		return () => document.removeEventListener('mousedown', handleClickOutside, true);
	});

	function close() {
		menuOpen = false;
		searchTerm = '';
	}

	async function selectGame(slug: string) {
		await games.setActive(slug);
		close();
	}
</script>

<div class="z-sidebar-game">
	{#if games.active}
		<Tooltip text={games.active.name} position="right" delay={300}>
			<button class="z-game-btn" onclick={() => (menuOpen = !menuOpen)}>
				<img src={gameIconSrc(games.active)} alt={games.active.name} class="z-game-icon" />
			</button>
		</Tooltip>
	{:else}
		<div class="z-game-btn z-game-placeholder">
			<Icon icon="mdi:gamepad-variant" />
		</div>
	{/if}
</div>

{#if menuOpen && games.list.length > 0}
	<div class="z-game-dropdown" bind:this={dropdownEl}>
		<div class="z-game-dropdown-search">
			<Icon icon="mdi:magnify" />
			<input
				type="text"
				placeholder={i18nState.locale && m.sidebar_searchGames()}
				bind:value={searchTerm}
				bind:this={searchInput}
				class="z-game-search-input"
				onkeydown={(e) => e.key === 'Escape' && close()}
			/>
		</div>
		<div class="z-game-dropdown-list">
			{#each filteredGames as game}
				<button
					class="z-game-dropdown-item"
					class:active={game.slug === games.active?.slug}
					onclick={() => selectGame(game.slug)}
				>
					<img src={gameIconSrc(game)} alt={game.name} class="z-game-dropdown-icon" />
					<span class="z-game-dropdown-name">{game.name}</span>
					{#if game.favorite}
						<Icon icon="mdi:star" class="text-xs text-amber-400" />
					{/if}
				</button>
			{/each}
			{#if filteredGames.length === 0}
				<div class="z-game-dropdown-empty">
					<span>{i18nState.locale && m.sidebar_noGamesFound()}</span>
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	.z-sidebar-game {
		display: flex;
		justify-content: center;
		padding: 0 var(--space-md);
		margin-bottom: var(--space-sm);
	}

	.z-game-btn {
		width: 44px;
		height: 44px;
		border-radius: var(--radius-lg);
		border: 2px solid var(--border-default);
		background: var(--bg-elevated);
		cursor: pointer;
		overflow: hidden;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all var(--transition-fast);
		color: var(--text-muted);
		font-size: 20px;
	}

	.z-game-btn:hover {
		border-color: var(--accent-400);
		box-shadow: var(--shadow-glow);
	}

	.z-game-icon {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.z-game-dropdown {
		position: absolute;
		top: var(--space-md);
		left: 72px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		padding: 0;
		min-width: 240px;
		max-height: 500px;
		overflow: hidden;
		z-index: var(--z-dropdown);
		box-shadow: var(--shadow-lg);
		animation: scaleIn var(--transition-fast) ease;
		display: flex;
		flex-direction: column;
	}

	.z-game-dropdown-search {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-md);
		border-bottom: 1px solid var(--border-subtle);
		background: var(--bg-surface);
		color: var(--text-muted);
		font-size: 16px;
		flex-shrink: 0;
	}

	.z-game-search-input {
		flex: 1;
		background: transparent;
		border: none;
		color: var(--text-primary);
		font-size: 13px;
		font-family: var(--font-body);
		outline: none;
		padding: 0;
	}

	.z-game-search-input::placeholder {
		color: var(--text-muted);
	}

	.z-game-dropdown-list {
		overflow-y: auto;
		max-height: 420px;
		padding: var(--space-xs);
	}

	.z-game-dropdown-empty {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: var(--space-md);
		color: var(--text-muted);
		font-size: 12px;
		text-align: center;
	}

	.z-game-dropdown-item {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		width: 100%;
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-md);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		transition: all var(--transition-fast);
		font-family: var(--font-body);
		font-size: 13px;
	}

	.z-game-dropdown-item:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.z-game-dropdown-item.active {
		background: var(--bg-active);
		color: var(--text-accent);
	}

	.z-game-dropdown-icon {
		width: 28px;
		height: 28px;
		border-radius: var(--radius-sm);
		object-fit: cover;
		flex-shrink: 0;
	}

	.z-game-dropdown-name {
		flex: 1;
		text-align: left;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>
