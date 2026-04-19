<script lang="ts">
	type Props = {
		src: string | null | undefined;
		alt?: string;
		class?: string;
		loading?: 'eager' | 'lazy';
		width?: number | string;
		height?: number | string;
	};

	let { src, alt = '', class: className = '', loading = 'lazy', width, height }: Props = $props();

	let displaySrc = $state<string | null>(null);
	let failed = $state(false);

	$effect(() => {
		failed = false;

		if (!src) {
			displaySrc = null;
			failed = true;
			return;
		}

		displaySrc = src;
	});

	function onError() {
		failed = true;
	}
</script>

{#if displaySrc && !failed}
	<img src={displaySrc} {alt} class={className} {loading} {width} {height} onerror={onError} />
{:else}
	<div class="z-cached-img-fallback {className}" aria-label={alt}>
		<svg
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="1.5"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
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
