<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';

	type Props = {
		id: string;
		name: string;
		size?: number;
	};

	const { id, name, size = 64 }: Props = $props();

	let imageUrl: string | null = $state(null);
	let loaded = $state(false);
	let failed = $state(false);

	const CACHE_KEY = 'rdz-logo-cache-v1';
	const CACHE_TTL = 1000 * 60 * 60 * 24 * 30; // 30 days

	type CacheEntry = { url: string | null; ts: number };

	function readCache(): Record<string, CacheEntry> {
		try {
			const raw = localStorage.getItem(CACHE_KEY);
			return raw ? JSON.parse(raw) : {};
		} catch {
			return {};
		}
	}

	function writeCache(c: Record<string, CacheEntry>) {
		try {
			localStorage.setItem(CACHE_KEY, JSON.stringify(c));
		} catch {
			/* quota or disabled */
		}
	}

	function hashHue(s: string): number {
		let h = 0;
		for (let i = 0; i < s.length; i++) {
			h = (h << 5) - h + s.charCodeAt(i);
			h |= 0;
		}
		return Math.abs(h) % 360;
	}

	async function fetchWikipediaImage(query: string): Promise<string | null> {
		// REST summary endpoint — single GET, returns thumbnail if the page has one
		const url = `https://en.wikipedia.org/api/rest_v1/page/summary/${encodeURIComponent(
			query
		)}?redirect=true`;
		try {
			const res = await fetch(url, {
				headers: { Accept: 'application/json' }
			});
			if (!res.ok) return null;
			const data = await res.json();
			return data?.thumbnail?.source ?? data?.originalimage?.source ?? null;
		} catch {
			return null;
		}
	}

	async function resolveLogo() {
		const cache = readCache();
		const cached = cache[id];
		if (cached && Date.now() - cached.ts < CACHE_TTL) {
			imageUrl = cached.url;
			loaded = !!cached.url;
			failed = !cached.url;
			return;
		}

		// Try a couple of query variants
		const variants = [
			`${name} (video game)`,
			`${name} video game`,
			name
		];
		let found: string | null = null;
		for (const q of variants) {
			found = await fetchWikipediaImage(q);
			if (found) break;
		}

		cache[id] = { url: found, ts: Date.now() };
		writeCache(cache);

		imageUrl = found;
		failed = !found;
	}

	onMount(() => {
		resolveLogo();
	});

	const hue = $derived(hashHue(id));
</script>

<div
	class="z-game-logo"
	style="
		--logo-size: {size}px;
		--logo-hue: {hue};
	"
>
	{#if imageUrl && !failed}
		<img
			src={imageUrl}
			alt={name}
			class="z-logo-img"
			class:loaded
			loading="lazy"
			onload={() => (loaded = true)}
			onerror={() => {
				failed = true;
				loaded = false;
			}}
		/>
	{/if}
	{#if !imageUrl || failed || !loaded}
		<div class="z-logo-fallback">
			<img src="/logo.png" alt="Zephyr" class="z-logo-placeholder" />
		</div>
	{/if}
</div>

<style>
	.z-game-logo {
		position: relative;
		width: var(--logo-size);
		height: var(--logo-size);
		flex-shrink: 0;
		border-radius: var(--radius-md);
		overflow: hidden;
		background:
			radial-gradient(
				circle at 30% 25%,
				hsla(var(--logo-hue), 75%, 55%, 0.5),
				hsla(calc(var(--logo-hue) + 40), 65%, 25%, 0.95) 70%
			),
			linear-gradient(135deg, hsl(var(--logo-hue), 45%, 14%), hsl(var(--logo-hue), 55%, 8%));
		border: 1px solid hsla(var(--logo-hue), 60%, 50%, 0.35);
		box-shadow:
			inset 0 1px 0 hsla(var(--logo-hue), 90%, 80%, 0.2),
			0 2px 8px hsla(var(--logo-hue), 70%, 15%, 0.5);
	}

	.z-logo-img {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		object-fit: cover;
		opacity: 0;
		transition: opacity 250ms ease;
	}

	.z-logo-img.loaded {
		opacity: 1;
	}

	.z-logo-fallback {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		color: hsla(var(--logo-hue), 90%, 85%, 0.55);
		font-size: calc(var(--logo-size) * 0.45);
	}

	.z-logo-placeholder {
		width: 55%;
		height: 55%;
		object-fit: contain;
		opacity: 0.3;
		filter: grayscale(1) brightness(2);
	}
</style>
