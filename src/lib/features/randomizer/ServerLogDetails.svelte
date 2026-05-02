<script lang="ts">
	import Icon from '@iconify/svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		lines: string[];
		copiedKey: string | null;
		oncopy: () => void;
	};

	let { lines, copiedKey, oncopy }: Props = $props();

	let isCopied = $derived(copiedKey === 'srv');
</script>

<details class="rdz-log-details" open>
	<summary>
		<span>{i18nState.locale && m.randomizer_serverLog()} ({lines.length} lines)</span>
		<button
			class="rdz-log-copy"
			title={i18nState.locale && m.randomizer_copy()}
			onclick={(e) => {
				e.preventDefault();
				oncopy();
			}}
		>
			<Icon icon={isCopied ? 'mdi:check' : 'mdi:content-copy'} />
			{isCopied
				? i18nState.locale && m.randomizer_copied()
				: i18nState.locale && m.randomizer_copy()}
		</button>
	</summary>
	<pre class="rdz-log">{lines.join('\n')}</pre>
</details>

<style>
	.rdz-log {
		margin: 0;
		padding: var(--space-sm);
		background: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-sm);
		font-family: var(--font-mono, monospace);
		font-size: 11px;
		color: var(--text-secondary);
		max-height: 240px;
		overflow: auto;
		white-space: pre-wrap;
		word-break: break-all;
		user-select: text !important;
		-webkit-user-select: text !important;
		cursor: text !important;
	}

	.rdz-log::selection {
		background: var(--accent-400);
		color: var(--text-primary);
	}

	.rdz-log-details summary {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		cursor: pointer;
		font-size: 11px;
		color: var(--text-muted);
		padding: 4px 0;
		list-style: none;
	}

	.rdz-log-details summary::-webkit-details-marker {
		display: none;
	}

	.rdz-log-details summary::before {
		content: '▶';
		display: inline-block;
		font-size: 9px;
		transition: transform var(--transition-fast);
		color: var(--text-muted);
	}

	.rdz-log-details[open] summary::before {
		transform: rotate(90deg);
	}

	.rdz-log-details summary span {
		flex: 1;
	}

	.rdz-log-copy {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 3px 8px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		color: var(--text-secondary);
		font-size: 10px;
		font-weight: 600;
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.rdz-log-copy :global(svg) {
		font-size: 12px;
	}

	.rdz-log-copy:hover {
		border-color: var(--accent-400);
		color: var(--accent-400);
	}
</style>
