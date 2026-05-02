<script lang="ts">
	import Icon from '@iconify/svelte';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		markdown: string;
		loading: boolean;
	};

	let { markdown, loading }: Props = $props();

	let copied = $state(false);
	let copyTimeoutId: number | null = null;

	function stripHtml(html: string): string {
		const doc = new DOMParser().parseFromString(html, 'text/html');
		return doc.body.textContent ?? '';
	}

	async function copy() {
		try {
			await writeText(stripHtml(markdown));
			copied = true;
			if (copyTimeoutId !== null) clearTimeout(copyTimeoutId);
			copyTimeoutId = window.setTimeout(() => (copied = false), 2000);
		} catch (err) {
			console.error('Failed to copy:', err);
		}
	}
</script>

{#if markdown && !loading}
	<button class="z-copy-btn" class:copied onclick={copy} title="Copy content">
		<Icon icon={copied ? 'mdi:check' : 'mdi:content-copy'} />
		<span class="z-copy-text">{copied ? 'Copied!' : 'Copy'}</span>
	</button>
{/if}

{#if loading}
	<div class="z-details-loading">
		<Spinner size={20} />
	</div>
{:else if markdown}
	<div class="markdown">
		{@html markdown}
	</div>
{:else}
	<p class="z-details-empty">{i18nState.locale && m.modDetails_noContent()}</p>
{/if}

<style>
	.z-copy-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-subtle);
		background: var(--bg-elevated);
		color: var(--text-secondary);
		font-family: var(--font-body);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all var(--transition-fast);
		white-space: nowrap;
	}

	.z-copy-btn:hover:not(.copied) {
		background: var(--bg-hover);
		border-color: var(--border-default);
		color: var(--text-primary);
	}

	.z-copy-btn.copied {
		background: rgba(0, 212, 170, 0.1);
		border-color: rgba(0, 212, 170, 0.3);
		color: var(--success);
	}

	.z-copy-text {
		display: none;
	}

	@media (min-width: 420px) {
		.z-copy-text {
			display: inline;
		}
	}

	.z-details-loading {
		display: flex;
		justify-content: center;
		padding: var(--space-2xl);
		color: var(--text-muted);
	}

	.z-details-empty {
		text-align: center;
		color: var(--text-muted);
		font-size: 13px;
		padding: var(--space-2xl);
	}
</style>
