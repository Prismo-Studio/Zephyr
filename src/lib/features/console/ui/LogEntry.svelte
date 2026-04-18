<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import type { LogEntry } from '../core/protocol';

	type Props = {
		entry: LogEntry;
		onFilterSource?: (source: string) => void;
	};
	let { entry, onFilterSource }: Props = $props();

	const time = $derived(() => {
		const d = new Date(entry.ts);
		const hh = String(d.getHours()).padStart(2, '0');
		const mm = String(d.getMinutes()).padStart(2, '0');
		const ss = String(d.getSeconds()).padStart(2, '0');
		return `${hh}:${mm}:${ss}`;
	});
</script>

<div class="zc-entry zc-level-{entry.level}" data-origin={entry.origin}>
	<span class="zc-time">{time()}</span>
	{#if entry.source}
		<button
			class="zc-badge zc-badge-{entry.level}"
			onclick={() => onFilterSource?.(entry.source!)}
			title={i18nState.locale && m.console_feed_filterTooltip({ source: entry.source })}
		>{entry.source}</button>
	{:else}
		<span class="zc-badge-spacer"></span>
	{/if}
	<span class="zc-text">{entry.text}</span>
</div>

<style>
	.zc-entry {
		display: grid;
		grid-template-columns: 62px 72px 1fr;
		align-items: baseline;
		gap: 8px;
		padding: 2px 12px;
		font-family: 'JetBrains Mono', ui-monospace, Menlo, Consolas, monospace;
		font-size: 12.5px;
		line-height: 1.7;
		color: var(--text-secondary);
		border-left: 2px solid transparent;
	}

	.zc-entry:hover {
		background: var(--bg-hover);
	}

	.zc-time {
		color: var(--text-muted);
		font-size: 11px;
		font-variant-numeric: tabular-nums;
	}

	.zc-badge {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		padding: 1px 8px;
		border-radius: var(--radius-sm);
		border: none;
		font-family: inherit;
		font-size: 10.5px;
		font-weight: 700;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		cursor: pointer;
		white-space: nowrap;
		max-width: 70px;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.zc-badge:hover {
		filter: brightness(1.2);
	}

	.zc-badge-spacer {
		display: inline-block;
	}

	.zc-text {
		color: var(--text-primary);
		word-break: break-word;
		white-space: pre-wrap;
	}

	/* Level-specific accents */
	.zc-level-info .zc-badge {
		background: color-mix(in srgb, var(--text-secondary) 15%, transparent);
		color: var(--text-secondary);
	}
	.zc-level-warn .zc-badge {
		background: color-mix(in srgb, var(--warning) 18%, transparent);
		color: var(--warning);
	}
	.zc-level-warn {
		border-left-color: color-mix(in srgb, var(--warning) 60%, transparent);
	}
	.zc-level-error .zc-badge {
		background: color-mix(in srgb, var(--error) 18%, transparent);
		color: var(--error);
	}
	.zc-level-error {
		border-left-color: var(--error);
	}
	.zc-level-error .zc-text {
		color: var(--error);
	}
	.zc-level-hint .zc-badge {
		background: color-mix(in srgb, var(--accent-400) 15%, transparent);
		color: var(--accent-400);
	}
	.zc-level-item .zc-badge {
		background: color-mix(in srgb, var(--success) 18%, transparent);
		color: var(--success);
	}
	.zc-level-item .zc-text {
		color: color-mix(in srgb, var(--success) 85%, var(--text-primary));
	}
	.zc-level-chat .zc-badge {
		background: color-mix(in srgb, var(--text-primary) 14%, transparent);
		color: var(--text-primary);
	}
	.zc-level-admin .zc-badge {
		background: color-mix(in srgb, var(--accent-400) 18%, transparent);
		color: var(--accent-400);
	}
	.zc-level-admin {
		border-left-color: color-mix(in srgb, var(--accent-400) 50%, transparent);
	}
	.zc-level-system .zc-badge {
		background: color-mix(in srgb, var(--text-muted) 18%, transparent);
		color: var(--text-muted);
	}
	.zc-level-system .zc-text {
		color: var(--text-muted);
		font-style: italic;
	}
</style>
