<script lang="ts" module>
	import { convertFileSrc } from '@tauri-apps/api/core';
	import * as api from '$lib/api';

	const resolved = new Map<string, string>();
	const inflight = new Map<string, Promise<string>>();

	function isRemote(url: string): boolean {
		return url.startsWith('http://') || url.startsWith('https://');
	}

	export async function resolveCached(url: string): Promise<string> {
		const cached = resolved.get(url);
		if (cached) return cached;

		const pending = inflight.get(url);
		if (pending) return pending;

		const promise = (async () => {
			try {
				const path = await api.iconCache.getCachedIcon(url);
				const final = path && path !== url ? convertFileSrc(path) : url;
				resolved.set(url, final);
				return final;
			} catch {
				resolved.set(url, url);
				return url;
			} finally {
				inflight.delete(url);
			}
		})();

		inflight.set(url, promise);
		return promise;
	}
</script>

<script lang="ts">
	type Props = {
		src: string | null | undefined;
		alt?: string;
		class?: string;
		loading?: 'eager' | 'lazy';
		width?: number | string;
		height?: number | string;
	};

	let {
		src,
		alt = '',
		class: className = '',
		loading = 'lazy',
		width,
		height
	}: Props = $props();

	let displaySrc = $state<string | null>(null);
	let failed = $state(false);

	$effect(() => {
		failed = false;

		if (!src) {
			displaySrc = null;
			failed = true;
			return;
		}

		if (!isRemote(src)) {
			displaySrc = src;
			return;
		}

		const cached = resolved.get(src);
		if (cached) {
			displaySrc = cached;
			return;
		}

		displaySrc = src;
		const requested = src;
		void resolveCached(requested).then((url) => {
			if (requested === src && url !== displaySrc) {
				displaySrc = url;
			}
		});
	});

	function onError() {
		failed = true;
	}
</script>

{#if displaySrc && !failed}
	<img
		src={displaySrc}
		{alt}
		class={className}
		{loading}
		{width}
		{height}
		onerror={onError}
	/>
{:else}
	<div class="z-cached-img-fallback {className}" aria-label={alt}>
		<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
			<rect x="3" y="3" width="18" height="18" rx="2" />
			<circle cx="9" cy="9" r="2" />
			<path d="M21 15l-5-5L5 21" />
		</svg>
	</div>
{/if}

<style>
	.z-cached-img-fallback {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--bg-overlay);
		color: var(--text-muted);
		border-radius: inherit;
	}

	.z-cached-img-fallback svg {
		width: 50%;
		height: 50%;
		max-width: 32px;
		max-height: 32px;
	}
</style>
