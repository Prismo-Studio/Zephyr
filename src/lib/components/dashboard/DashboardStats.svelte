<script lang="ts">
	import Icon from '@iconify/svelte';
	import games from '$lib/state/game.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
</script>

<div class="z-stats-row">
	<div class="z-stat-card">
		<div class="z-stat-icon"><Icon icon="mdi:gamepad-variant" /></div>
		<div class="z-stat-info">
			<span class="z-stat-value">{games.list.length}</span>
			<span class="z-stat-label">{i18nState.locale && m.dashboard_games()}</span>
		</div>
	</div>
	<div class="z-stat-card">
		<div class="z-stat-icon"><Icon icon="mdi:account-group" /></div>
		<div class="z-stat-info">
			<span class="z-stat-value">{profiles.list.length}</span>
			<span class="z-stat-label">{i18nState.locale && m.dashboard_stats_profiles()}</span>
		</div>
	</div>
	<div class="z-stat-card">
		<div class="z-stat-icon"><Icon icon="mdi:puzzle" /></div>
		<div class="z-stat-info">
			<span class="z-stat-value">{profiles.active?.modCount ?? 0}</span>
			<span class="z-stat-label">{i18nState.locale && m.dashboard_stats_modsInstalled()}</span>
		</div>
	</div>
</div>

<style>
	.z-stats-row {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: var(--space-md);
	}

	.z-stat-card {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-lg);
		border-radius: var(--radius-lg);
		background: var(--bg-surface);
		border: 1px solid var(--border-subtle);
	}

	.z-stat-icon {
		font-size: 24px;
		color: var(--text-accent);
		flex-shrink: 0;
	}

	.z-stat-info {
		display: flex;
		flex-direction: column;
	}

	.z-stat-value {
		font-family: var(--font-display);
		font-size: 22px;
		font-weight: 800;
		color: var(--text-primary);
	}

	.z-stat-label {
		font-size: 11px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}
</style>
