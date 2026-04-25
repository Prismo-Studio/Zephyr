<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { PythonStatus } from './types';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	let { python }: { python: PythonStatus | null } = $props();
</script>

<div class="rdz-block-body">
	{#if python === null}
		<p class="rdz-muted">{i18nState.locale && m.randomizer_checking()}</p>
	{:else if python.available}
		<p class="rdz-ok">
			<Icon icon="mdi:check-circle" />
			{i18nState.locale && m.randomizer_pythonFound({ version: python.version ?? '' })} ({python.executable})
		</p>
		{#if !python.ap_present}
			<p class="rdz-warn">
				<Icon icon="mdi:alert" />
				{i18nState.locale && m.randomizer_apNotFound()} <code>{python.ap_dir}</code>
			</p>
		{:else}
			<p class="rdz-muted"><code>{python.ap_dir}</code></p>
		{/if}
	{:else}
		<p class="rdz-err">
			<Icon icon="mdi:close-circle" />
			{i18nState.locale && m.randomizer_pythonNotDetected()}
		</p>
	{/if}
</div>

<style>
	.rdz-block-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		padding: 0;
		animation: rdz-block-in 180ms ease;
	}

	@keyframes rdz-block-in {
		from {
			opacity: 0;
			transform: translateY(-2px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.rdz-muted {
		margin: 0;
		color: var(--text-muted);
		font-size: 12px;
	}

	.rdz-ok {
		display: flex;
		align-items: center;
		gap: 6px;
		margin: 0;
		color: #66d9b0;
		font-size: 12px;
	}

	.rdz-warn {
		display: flex;
		align-items: center;
		gap: 6px;
		margin: 0;
		color: #ffcc80;
		font-size: 12px;
	}

	.rdz-err {
		display: flex;
		align-items: center;
		gap: 6px;
		margin: 0;
		color: #ef9a9a;
		font-size: 12px;
	}

	code {
		font-family: var(--font-mono, monospace);
		font-size: 12px;
		color: var(--text-accent);
		background: var(--bg-active);
		padding: 2px 6px;
		border-radius: var(--radius-sm);
	}
</style>
