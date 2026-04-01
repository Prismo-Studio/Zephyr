<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		text: string;
		position?: 'top' | 'bottom' | 'left' | 'right';
		delay?: number;
		children?: Snippet;
	};

	let { text, position = 'top', delay = 0, children }: Props = $props();

	let visible = $state(false);
	let timer: ReturnType<typeof setTimeout> | null = null;

	function show() {
		if (delay > 0) {
			timer = setTimeout(() => {
				visible = true;
			}, delay);
		} else {
			visible = true;
		}
	}

	function hide() {
		if (timer) {
			clearTimeout(timer);
			timer = null;
		}
		visible = false;
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="z-tooltip-trigger" onmouseenter={show} onmouseleave={hide}>
	{#if children}{@render children()}{/if}
	{#if visible && text}
		<div class="z-tooltip z-tooltip-{position}" role="tooltip">
			<div class="z-tooltip-arrow"></div>
			{text}
		</div>
	{/if}
</div>

<style>
	.z-tooltip-trigger {
		position: relative;
		display: inline-flex;
	}

	.z-tooltip {
		position: absolute;
		padding: 6px 12px;
		font-size: 12px;
		font-weight: 500;
		color: var(--text-primary);
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		white-space: nowrap;
		pointer-events: none;
		z-index: var(--z-tooltip);
		animation: tooltipIn 150ms ease;
		box-shadow: var(--shadow-lg);
	}

	.z-tooltip-arrow {
		position: absolute;
		width: 8px;
		height: 8px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		transform: rotate(45deg);
	}

	/* Positions */
	.z-tooltip-top {
		bottom: calc(100% + 8px);
		left: 50%;
		transform: translateX(-50%);
	}
	.z-tooltip-top .z-tooltip-arrow {
		bottom: -5px;
		left: 50%;
		margin-left: -4px;
		border-top: none;
		border-left: none;
	}

	.z-tooltip-bottom {
		top: calc(100% + 8px);
		left: 50%;
		transform: translateX(-50%);
	}
	.z-tooltip-bottom .z-tooltip-arrow {
		top: -5px;
		left: 50%;
		margin-left: -4px;
		border-bottom: none;
		border-right: none;
	}

	.z-tooltip-left {
		right: calc(100% + 8px);
		top: 50%;
		transform: translateY(-50%);
	}
	.z-tooltip-left .z-tooltip-arrow {
		right: -5px;
		top: 50%;
		margin-top: -4px;
		border-bottom: none;
		border-left: none;
	}

	.z-tooltip-right {
		left: calc(100% + 8px);
		top: 50%;
		transform: translateY(-50%);
	}
	.z-tooltip-right .z-tooltip-arrow {
		left: -5px;
		top: 50%;
		margin-top: -4px;
		border-top: none;
		border-right: none;
	}

	@keyframes tooltipIn {
		from {
			opacity: 0;
			transform: translateX(-50%) scale(0.95);
		}
		to {
			opacity: 1;
			transform: translateX(-50%) scale(1);
		}
	}

	.z-tooltip-left {
		animation: tooltipInY 150ms ease;
	}
	.z-tooltip-right {
		animation: tooltipInY 150ms ease;
	}

	@keyframes tooltipInY {
		from {
			opacity: 0;
			transform: translateY(-50%) scale(0.95);
		}
		to {
			opacity: 1;
			transform: translateY(-50%) scale(1);
		}
	}
</style>
