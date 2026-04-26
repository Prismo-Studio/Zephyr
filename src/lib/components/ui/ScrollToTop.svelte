<script lang="ts">
	import { tick } from 'svelte';
	import Icon from '@iconify/svelte';

	type Props = {
		target: HTMLElement | null | undefined;
		threshold?: number;
		lifted?: boolean;
	};

	let { target, threshold = 400, lifted = false }: Props = $props();

	let visible = $state(false);
	let buttonEl: HTMLButtonElement | undefined = $state();
	let actuallyLifted = $state(false);

	function onScroll() {
		if (!target) return;
		visible = target.scrollTop > threshold;
	}

	function refreshOverlap() {
		if (!lifted || !buttonEl) {
			actuallyLifted = false;
			return;
		}
		const batchBar = document.querySelector('.z-batch-bar-container .z-batch-bar');
		if (!batchBar) {
			actuallyLifted = false;
			return;
		}
		const btnRect = buttonEl.getBoundingClientRect();
		const barRect = batchBar.getBoundingClientRect();
		const margin = 12;
		actuallyLifted = barRect.right + margin > btnRect.left && barRect.left < btnRect.right + margin;
	}

	$effect(() => {
		if (!target) return;
		target.addEventListener('scroll', onScroll, { passive: true });
		onScroll();
		return () => target?.removeEventListener('scroll', onScroll);
	});

	$effect(() => {
		void lifted;
		void tick().then(refreshOverlap);
		window.addEventListener('resize', refreshOverlap);
		return () => window.removeEventListener('resize', refreshOverlap);
	});

	function scrollUp() {
		target?.scrollTo({ top: 0, behavior: 'smooth' });
	}
</script>

<button
	type="button"
	class="z-scroll-top"
	class:visible
	class:lifted={actuallyLifted}
	bind:this={buttonEl}
	onclick={scrollUp}
	aria-label="Scroll to top"
	tabindex={visible ? 0 : -1}
>
	<Icon icon="mdi:chevron-up" />
</button>

<style>
	.z-scroll-top {
		position: absolute;
		right: var(--space-md);
		bottom: var(--space-md);
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 38px;
		height: 38px;
		border-radius: var(--radius-full);
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		color: var(--text-secondary);
		cursor: pointer;
		opacity: 0;
		transform: translateY(8px);
		pointer-events: none;
		transition:
			opacity var(--transition-fast),
			transform var(--transition-fast),
			bottom var(--transition-base),
			background var(--transition-fast),
			color var(--transition-fast),
			border-color var(--transition-fast);
		box-shadow: var(--shadow-md);
		z-index: 5;
	}

	.z-scroll-top.visible {
		opacity: 1;
		transform: translateY(0);
		pointer-events: auto;
	}

	.z-scroll-top.lifted {
		bottom: calc(var(--space-md) + 72px);
	}

	.z-scroll-top:hover {
		background: var(--bg-active);
		color: var(--accent-400);
		border-color: var(--accent-400);
	}

	.z-scroll-top :global(svg) {
		font-size: 20px;
	}
</style>
