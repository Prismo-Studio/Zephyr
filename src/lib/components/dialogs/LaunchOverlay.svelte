<script lang="ts">
	import Icon from '@iconify/svelte';
	import games from '$lib/state/game.svelte';
	import { gameIconSrc } from '$lib/util';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		visible: boolean;
		onclose: () => void;
	};

	let { visible = $bindable(), onclose }: Props = $props();

	$effect(() => {
		if (visible) {
			const timer = setTimeout(() => {
				visible = false;
				onclose();
			}, 3500);
			return () => clearTimeout(timer);
		}
	});
</script>

{#if visible}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="z-launch-overlay"
		onclick={() => {
			visible = false;
			onclose();
		}}
	>
		<div class="z-launch-content">
			<div class="z-launch-icon-wrapper">
				<div class="z-launch-ring z-ring-1"></div>
				<div class="z-launch-ring z-ring-2"></div>
				<div class="z-launch-ring z-ring-3"></div>
				{#if games.active}
					<img src={gameIconSrc(games.active)} alt={games.active.name} class="z-launch-game-icon" />
				{:else}
					<div class="z-launch-game-icon z-launch-placeholder">
						<Icon icon="mdi:gamepad-variant" />
					</div>
				{/if}
			</div>

			<h2 class="z-launch-title">{games.active?.name ?? 'Game'}</h2>
			<span class="z-launch-text">{i18nState.locale && m.launch_launching()}</span>

			<div class="z-launch-bar">
				<div class="z-launch-bar-fill"></div>
			</div>
		</div>
	</div>
{/if}

<style>
	.z-launch-overlay {
		position: fixed;
		inset: 0;
		z-index: 9999;
		background: rgba(0, 0, 0, 0.85);
		backdrop-filter: blur(12px);
		display: flex;
		align-items: center;
		justify-content: center;
		animation: overlayIn 0.3s ease;
		cursor: pointer;
	}

	@keyframes overlayIn {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}

	.z-launch-content {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-lg);
		animation: contentIn 0.4s cubic-bezier(0.16, 1, 0.3, 1);
	}

	@keyframes contentIn {
		from {
			opacity: 0;
			transform: scale(0.9) translateY(20px);
		}
		to {
			opacity: 1;
			transform: scale(1) translateY(0);
		}
	}

	.z-launch-icon-wrapper {
		position: relative;
		width: 120px;
		height: 120px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.z-launch-ring {
		position: absolute;
		border-radius: 50%;
		border: 2px solid transparent;
	}

	.z-ring-1 {
		inset: -8px;
		border-top-color: var(--accent-400);
		border-right-color: var(--accent-400);
		animation: ringRotate 1.2s cubic-bezier(0.5, 0, 0.5, 1) infinite;
	}

	.z-ring-2 {
		inset: -4px;
		border-bottom-color: var(--accent-300);
		animation: ringRotate 1.6s cubic-bezier(0.5, 0, 0.5, 1) infinite reverse;
		opacity: 0.6;
	}

	.z-ring-3 {
		inset: -12px;
		border-left-color: var(--accent-500);
		animation: ringRotate 2s cubic-bezier(0.5, 0, 0.5, 1) infinite;
		opacity: 0.3;
	}

	@keyframes ringRotate {
		to {
			transform: rotate(360deg);
		}
	}

	.z-launch-game-icon {
		width: 100px;
		height: 100px;
		border-radius: var(--radius-xl);
		object-fit: cover;
		box-shadow:
			0 0 30px rgba(26, 255, 250, 0.2),
			0 0 60px rgba(26, 255, 250, 0.1);
	}

	.z-launch-placeholder {
		background: var(--bg-elevated);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 40px;
		color: var(--text-muted);
	}

	.z-launch-title {
		font-size: 24px;
		font-weight: 700;
		color: var(--text-primary);
		letter-spacing: -0.02em;
		text-shadow: 0 0 20px rgba(26, 255, 250, 0.15);
	}

	.z-launch-text {
		font-size: 14px;
		color: var(--text-muted);
		font-weight: 500;
	}

	.z-launch-bar {
		width: 200px;
		height: 3px;
		border-radius: var(--radius-full);
		background: rgba(255, 255, 255, 0.08);
		overflow: hidden;
	}

	.z-launch-bar-fill {
		height: 100%;
		border-radius: var(--radius-full);
		background: linear-gradient(90deg, var(--accent-400), var(--accent-300));
		animation: barFill 3.5s cubic-bezier(0.25, 0.46, 0.45, 0.94) forwards;
		box-shadow: 0 0 8px rgba(26, 255, 250, 0.4);
	}

	@keyframes barFill {
		from {
			width: 0%;
		}
		to {
			width: 100%;
		}
	}
</style>
