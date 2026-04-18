<script lang="ts">
	import { tick } from 'svelte';
	import Icon from '@iconify/svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import type { LogStore } from '../core/log-store.svelte';
	import LogEntry from './LogEntry.svelte';

	type Props = {
		store: LogStore;
	};
	let { store }: Props = $props();

	let scrollEl: HTMLDivElement | undefined = $state();
	let autoscroll = $state(true);
	let prevLen = 0;

	// Append-on-scroll-bottom: only autoscroll if the user is already near
	// the bottom. If they scrolled up to read history, we leave them alone.
	function onScroll() {
		if (!scrollEl) return;
		const distanceFromBottom = scrollEl.scrollHeight - scrollEl.scrollTop - scrollEl.clientHeight;
		autoscroll = distanceFromBottom < 40;
	}

	$effect(() => {
		// Track entries length as the reactive trigger.
		const len = store.filtered.length;
		if (len === prevLen) return;
		prevLen = len;
		if (!autoscroll) return;
		tick().then(() => {
			if (scrollEl) scrollEl.scrollTop = scrollEl.scrollHeight;
		});
	});

	function clearFilter() {
		store.sourceFilter = null;
		store.search = '';
	}
</script>

{#if store.sourceFilter || store.search}
	<div class="zc-filter-bar">
		<Icon icon="mdi:filter" />
		<span class="zc-filter-label">
			{#if store.sourceFilter}
				{i18nState.locale && m.console_feed_filtering()}:
				<strong>{store.sourceFilter}</strong>
			{/if}
			{#if store.search}
				{i18nState.locale && m.console_feed_search()}:
				<strong>"{store.search}"</strong>
			{/if}
		</span>
		<button class="zc-clear" onclick={clearFilter}>
			{i18nState.locale && m.console_feed_clear()}
		</button>
	</div>
{/if}

<div class="zc-feed" bind:this={scrollEl} onscroll={onScroll}>
	{#each store.filtered as entry (entry.id)}
		<LogEntry {entry} onFilterSource={(src) => (store.sourceFilter = src)} />
	{/each}

	{#if store.filtered.length === 0}
		<div class="zc-empty">
			<Icon icon="mdi:chat-processing-outline" />
			<p>{i18nState.locale && m.console_feed_empty()}</p>
			<small>
				{i18nState.locale && m.console_feed_emptyHint()}
				<code>/help</code>.
			</small>
		</div>
	{/if}
</div>

{#if !autoscroll}
	<button class="zc-scroll-pin" onclick={() => { autoscroll = true; if (scrollEl) scrollEl.scrollTop = scrollEl.scrollHeight; }}>
		<Icon icon="mdi:arrow-down-bold" />
		{i18nState.locale && m.console_feed_jumpLatest()}
	</button>
{/if}

<style>
	.zc-feed {
		flex: 1 1 0;
		min-height: 0;
		overflow-y: auto;
		overflow-x: hidden;
		padding: 8px 0;
		background: var(--bg-base);
		scroll-behavior: smooth;
	}

	.zc-filter-bar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 16px;
		background: var(--bg-elevated);
		border-bottom: 1px solid var(--border-subtle);
		font-size: 12px;
		color: var(--text-muted);
		flex-shrink: 0;
	}

	.zc-filter-bar strong {
		color: var(--text-primary);
	}

	.zc-filter-bar :global(svg) {
		font-size: 14px;
		color: var(--accent-400);
	}

	.zc-filter-label {
		flex: 1;
	}

	.zc-clear {
		background: transparent;
		border: 1px solid var(--border-default);
		border-radius: var(--radius-sm);
		color: var(--text-muted);
		padding: 2px 10px;
		font-size: 11px;
		cursor: pointer;
	}
	.zc-clear:hover {
		border-color: var(--accent-400);
		color: var(--accent-400);
	}

	.zc-empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 80px 24px;
		color: var(--text-muted);
	}
	.zc-empty :global(svg) {
		font-size: 40px;
		opacity: 0.5;
	}
	.zc-empty p {
		margin: 0;
		font-size: 13px;
	}
	.zc-empty small {
		font-size: 11px;
	}
	.zc-empty code {
		background: var(--bg-active);
		padding: 1px 6px;
		border-radius: 3px;
		font-family: 'JetBrains Mono', ui-monospace, monospace;
	}

	.zc-scroll-pin {
		position: absolute;
		bottom: 70px;
		right: 20px;
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 6px 12px;
		border: 1px solid var(--border-accent);
		background: var(--bg-elevated);
		color: var(--accent-400);
		border-radius: var(--radius-full);
		font-size: 11px;
		font-weight: 700;
		cursor: pointer;
		box-shadow: var(--shadow-glow);
	}
	.zc-scroll-pin:hover {
		background: var(--bg-active);
	}
</style>
