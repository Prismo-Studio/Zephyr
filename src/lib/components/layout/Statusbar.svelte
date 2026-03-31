<script lang="ts">
	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';

	let status: string | null = $state(null);
	let unlisten: UnlistenFn | undefined;

	onMount(() => {
		listen<string | null>('status_update', (evt) => {
			status = evt.payload;
		}).then((cb) => (unlisten = cb));

		return () => unlisten?.();
	});
</script>

<footer class="z-statusbar">
	<div class="z-statusbar-left">
		{#if status}
			<span class="z-status-text">{status}</span>
		{/if}
	</div>
	<div class="z-statusbar-right">
		<span class="z-version">v0.2.0</span>
	</div>
</footer>

<style>
	.z-statusbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		height: 24px;
		padding: 0 var(--space-lg);
		background: var(--bg-surface);
		border-top: 1px solid var(--border-subtle);
		font-size: 11px;
		color: var(--text-muted);
		flex-shrink: 0;
		gap: var(--space-lg);
	}

	.z-statusbar-left,
	.z-statusbar-right {
		display: flex;
		align-items: center;
		gap: var(--space-md);
	}

	.z-status-text {
		animation: fadeIn var(--transition-fast) ease;
	}

	.z-version {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-muted);
		opacity: 0.6;
	}
</style>
