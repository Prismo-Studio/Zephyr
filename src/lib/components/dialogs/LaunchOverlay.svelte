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
		class="z-overlay"
		onclick={() => {
			visible = false;
			onclose();
		}}
	>
		<div class="z-overlay-glow" aria-hidden="true"></div>

		<div class="z-launch-center">
			{#if games.active}
				<img
					src={gameIconSrc(games.active)}
					alt={games.active.name}
					class="z-launch-icon"
				/>
			{:else}
				<div class="z-launch-icon z-launch-placeholder">
					<Icon icon="mdi:gamepad-variant" />
				</div>
			{/if}

			<h2 class="z-launch-name">{games.active?.name ?? 'Game'}</h2>
			<span class="z-launch-status">{i18nState.locale && m.launch_launching()}</span>

			<div class="z-launch-bar">
				<div class="z-launch-bar-fill"></div>
			</div>
		</div>
	</div>
{/if}

<style>
	.z-overlay {
		position: fixed;
		inset: 0;
		z-index: 9999;
		background: rgba(4, 8, 18, 0.92);
		backdrop-filter: blur(24px);
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		animation: fadeIn 0.3s ease;
		cursor: pointer;
	}

	.z-overlay-glow {
		position: absolute;
		width: 500px;
		height: 500px;
		border-radius: 50%;
		background: radial-gradient(circle, rgba(26, 255, 250, 0.07) 0%, transparent 70%);
		filter: blur(80px);
		animation: glowPulse 3s ease-in-out infinite;
	}

	@keyframes fadeIn {
		from { opacity: 0; }
		to { opacity: 1; }
	}

	@keyframes glowPulse {
		0%, 100% { transform: scale(1); opacity: 0.5; }
		50% { transform: scale(1.15); opacity: 1; }
	}

	.z-launch-center {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 16px;
		position: relative;
		z-index: 1;
		animation: contentIn 0.4s cubic-bezier(0.16, 1, 0.3, 1);
	}

	@keyframes contentIn {
		from {
			opacity: 0;
			transform: scale(0.9) translateY(16px);
		}
		to {
			opacity: 1;
			transform: scale(1) translateY(0);
		}
	}

	.z-launch-icon {
		width: 120px;
		height: 120px;
		border-radius: 24px;
		object-fit: cover;
		border: 2px solid rgba(26, 255, 250, 0.2);
		box-shadow: 0 0 40px rgba(26, 255, 250, 0.2), 0 12px 40px rgba(0, 0, 0, 0.4);
		animation: iconPulse 2.5s ease-in-out infinite;
	}

	.z-launch-placeholder {
		background: var(--bg-elevated);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 48px;
		color: var(--text-muted);
	}

	@keyframes iconPulse {
		0%, 100% { box-shadow: 0 0 40px rgba(26, 255, 250, 0.2), 0 12px 40px rgba(0, 0, 0, 0.4); }
		50% { box-shadow: 0 0 60px rgba(26, 255, 250, 0.35), 0 12px 40px rgba(0, 0, 0, 0.4); }
	}

	.z-launch-name {
		font-size: 28px;
		font-weight: 800;
		color: var(--text-primary);
		letter-spacing: -0.02em;
		margin-top: 8px;
	}

	.z-launch-status {
		font-size: 14px;
		color: var(--text-muted);
		font-weight: 500;
	}

	.z-launch-bar {
		width: 240px;
		height: 4px;
		border-radius: 99px;
		background: rgba(255, 255, 255, 0.06);
		overflow: hidden;
		margin-top: 8px;
	}

	.z-launch-bar-fill {
		height: 100%;
		border-radius: 99px;
		background: linear-gradient(90deg, var(--accent-500), var(--accent-400), var(--accent-300), var(--accent-400));
		background-size: 300% 100%;
		box-shadow: 0 0 12px rgba(26, 255, 250, 0.5);
		animation: barFill 3.5s cubic-bezier(0.25, 0.46, 0.45, 0.94) forwards, barShine 1.5s linear infinite;
	}

	@keyframes barFill {
		from { width: 0%; }
		to { width: 100%; }
	}

	@keyframes barShine {
		from { background-position: 0% 50%; }
		to { background-position: 300% 50%; }
	}
</style>
