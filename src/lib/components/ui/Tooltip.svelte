<script lang="ts">
	import type { Snippet } from 'svelte';

	// Moves the node to document.body on mount so `position: fixed` is not
	// trapped inside an ancestor that establishes a containing block
	// (e.g. a parent with `transform`, `filter`, `perspective`, etc.).
	function portal(node: HTMLElement) {
		document.body.appendChild(node);
		return {
			destroy() {
				node.remove();
			}
		};
	}

	type Props = {
		text: string;
		position?: 'top' | 'bottom' | 'left' | 'right';
		delay?: number;
		block?: boolean;
		children?: Snippet;
	};

	let { text, position = 'top', delay = 0, block = false, children }: Props = $props();

	let visible = $state(false);
	let timer: ReturnType<typeof setTimeout> | null = null;
	let triggerEl: HTMLDivElement;
	let tooltipEl: HTMLDivElement | undefined = $state();
	let style = $state('');

	function setStyleForPosition(pos: 'top' | 'bottom' | 'left' | 'right', rect: DOMRect) {
		const gap = 8;
		switch (pos) {
			case 'top':
				style = `left:${rect.left + rect.width / 2}px;top:${rect.top - gap}px;transform:translateX(-50%) translateY(-100%)`;
				break;
			case 'bottom':
				style = `left:${rect.left + rect.width / 2}px;top:${rect.bottom + gap}px;transform:translateX(-50%)`;
				break;
			case 'left':
				style = `left:${rect.left - gap}px;top:${rect.top + rect.height / 2}px;transform:translateX(-100%) translateY(-50%)`;
				break;
			case 'right':
				style = `left:${rect.right + gap}px;top:${rect.top + rect.height / 2}px;transform:translateY(-50%)`;
				break;
		}
	}

	function updatePosition() {
		if (!triggerEl) return;
		const rect = triggerEl.getBoundingClientRect();
		setStyleForPosition(position, rect);
	}

	/** After the tooltip renders, measure its own rect and nudge the final
	 *  translate so the whole box stays inside the viewport. */
	function clampToViewport() {
		if (!tooltipEl || !triggerEl) return;
		const margin = 8;
		const rect = tooltipEl.getBoundingClientRect();
		const vw = window.innerWidth;
		const vh = window.innerHeight;

		let dx = 0;
		let dy = 0;
		if (rect.left < margin) dx = margin - rect.left;
		else if (rect.right > vw - margin) dx = vw - margin - rect.right;
		if (rect.top < margin) dy = margin - rect.top;
		else if (rect.bottom > vh - margin) dy = vh - margin - rect.bottom;

		if (dx !== 0 || dy !== 0) {
			// Append viewport-correction translate to the existing transform.
			style = `${style};margin-left:${dx}px;margin-top:${dy}px`;
		}
	}

	function show() {
		if (delay > 0) {
			timer = setTimeout(() => {
				updatePosition();
				visible = true;
				requestAnimationFrame(clampToViewport);
			}, delay);
		} else {
			updatePosition();
			visible = true;
			requestAnimationFrame(clampToViewport);
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
<div
	class="z-tooltip-trigger"
	class:block
	bind:this={triggerEl}
	onmouseenter={show}
	onmouseleave={hide}
>
	{#if children}{@render children()}{/if}
</div>

{#if visible && text}
	<div class="z-tooltip" role="tooltip" bind:this={tooltipEl} {style} use:portal>
		{text}
	</div>
{/if}

<style>
	.z-tooltip-trigger {
		position: relative;
		display: inline-flex;
	}

	.z-tooltip-trigger.block {
		display: flex;
		width: 100%;
		min-width: 0;
	}

	.z-tooltip {
		position: fixed;
		padding: 6px 12px;
		font-size: 12px;
		font-weight: 500;
		color: var(--text-primary);
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		max-width: min(420px, calc(100vw - 24px));
		overflow-wrap: anywhere;
		word-break: break-word;
		pointer-events: none;
		z-index: 9999;
		animation: tooltipIn 150ms ease;
		box-shadow: var(--shadow-lg);
	}

	@keyframes tooltipIn {
		from {
			opacity: 0;
			scale: 0.95;
		}
		to {
			opacity: 1;
			scale: 1;
		}
	}
</style>
