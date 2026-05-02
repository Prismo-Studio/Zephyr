<script lang="ts">
	import Icon from '@iconify/svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import games from '$lib/state/game.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { gameIconSrc } from '$lib/util';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
</script>

{#if games.active}
	<div class="z-hero-card glass">
		<img src={gameIconSrc(games.active)} alt={games.active.name} class="z-hero-bg" />
		<div class="z-hero-overlay">
			<div class="z-hero-info">
				<h2 class="z-hero-title">{games.active.name}</h2>
				<div class="z-hero-meta">
					<Badge variant="accent">{games.active.modLoader}</Badge>
					{#if profiles.active}
						<span class="z-hero-profile">
							<Icon icon="mdi:account-circle" />
							{profiles.active.name}
							<span class="z-hero-mods">· {profiles.active.modCount} mods</span>
						</span>
					{/if}
				</div>
			</div>
			<div class="z-hero-actions">
				<a href="/" class="z-hero-action">
					<Icon icon="mdi:package-variant" />
					<span>{i18nState.locale && m.dashboard_viewMods()}</span>
				</a>
				<a href="/browse" class="z-hero-action">
					<Icon icon="mdi:store-search" />
					<span>{i18nState.locale && m.navBar_label_browse()}</span>
				</a>
			</div>
		</div>
	</div>
{/if}

<style>
	.z-hero-card {
		position: relative;
		border-radius: var(--radius-xl);
		overflow: hidden;
		min-height: 180px;
	}

	.z-hero-bg {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		object-fit: cover;
		filter: blur(40px) brightness(0.3);
		transform: scale(1.2);
	}

	.z-hero-overlay {
		position: relative;
		z-index: 1;
		padding: var(--space-xl);
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		min-height: 180px;
		background: linear-gradient(135deg, rgba(0, 0, 0, 0.6), rgba(0, 0, 0, 0.2));
		color: #ffffff;
	}

	.z-hero-title {
		font-family: var(--font-display);
		font-size: 28px;
		font-weight: 800;
		letter-spacing: -0.03em;
	}

	.z-hero-meta {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		margin-top: var(--space-sm);
	}

	.z-hero-profile {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 13px;
		color: rgba(255, 255, 255, 0.8);
	}

	.z-hero-mods {
		color: rgba(255, 255, 255, 0.5);
	}

	.z-hero-actions {
		display: flex;
		gap: var(--space-sm);
		margin-top: var(--space-lg);
	}

	.z-hero-action {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: var(--space-sm) var(--space-lg);
		border-radius: var(--radius-md);
		background: rgba(255, 255, 255, 0.1);
		color: #ffffff;
		text-decoration: none;
		font-size: 13px;
		font-weight: 600;
		transition: all var(--transition-fast);
		backdrop-filter: blur(8px);
	}

	.z-hero-action:hover {
		background: rgba(255, 255, 255, 0.18);
	}
</style>
