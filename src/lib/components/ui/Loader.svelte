<script lang="ts">
	type Props = {
		message?: string;
	};

	let { message = '' }: Props = $props();
</script>

<div class="z-loader">
	<div class="z-loader-ring">
		<div class="z-ring z-ring-1"></div>
		<div class="z-ring z-ring-2"></div>
		<div class="z-ring z-ring-3"></div>
		<div class="z-loader-logo">Z</div>
	</div>
	{#if message}
		<p class="z-loader-text">{message}</p>
	{/if}
</div>

<style>
	.z-loader {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-lg);
		width: 100%;
		height: 100%;
		min-height: 200px;
	}

	.z-loader-ring {
		position: relative;
		width: 64px;
		height: 64px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.z-ring {
		position: absolute;
		inset: 0;
		border-radius: 50%;
		border: 2px solid transparent;
	}

	.z-ring-1 {
		border-top-color: var(--accent-400);
		animation: spin 1.2s cubic-bezier(0.5, 0, 0.5, 1) infinite;
	}

	.z-ring-2 {
		inset: 4px;
		border-right-color: var(--accent-300);
		animation: spin 1.6s cubic-bezier(0.5, 0, 0.5, 1) infinite reverse;
	}

	.z-ring-3 {
		inset: 8px;
		border-bottom-color: var(--accent-500);
		animation: spin 2s cubic-bezier(0.5, 0, 0.5, 1) infinite;
	}

	.z-loader-logo {
		font-size: 18px;
		font-weight: 800;
		color: var(--accent-400);
		text-shadow: 0 0 12px rgba(26, 255, 250, 0.4);
		animation: pulse 2s ease-in-out infinite;
	}

	.z-loader-text {
		font-size: 13px;
		color: var(--text-muted);
		font-weight: 500;
		letter-spacing: 0.02em;
		animation: pulse 2s ease-in-out infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	@keyframes pulse {
		0%,
		100% {
			opacity: 0.6;
		}
		50% {
			opacity: 1;
		}
	}
</style>
