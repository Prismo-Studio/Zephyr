<script lang="ts">
	import Icon from '@iconify/svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	let open = $state(false);
	let query = $state('');
	let matchCount = $state(0);
	let currentMatch = $state(0);
	let inputEl: HTMLInputElement;
	let marks: HTMLElement[] = [];

	const MARK_CLASS = 'z-search-highlight';
	const ACTIVE_CLASS = 'z-search-highlight-active';

	function clearHighlights() {
		for (const mark of marks) {
			const parent = mark.parentNode;
			if (parent) {
				parent.replaceChild(document.createTextNode(mark.textContent ?? ''), mark);
				parent.normalize();
			}
		}
		marks = [];
		matchCount = 0;
		currentMatch = 0;
	}

	function highlightText(root: Node, term: string) {
		if (!term) return;
		const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
		const textNodes: Text[] = [];

		while (walker.nextNode()) {
			const node = walker.currentNode as Text;
			const parent = node.parentElement;
			if (!parent) continue;
			if (parent.closest('.z-global-search, script, style, .z-titlebar')) continue;
			if (parent.tagName === 'SCRIPT' || parent.tagName === 'STYLE') continue;
			if (node.textContent && node.textContent.toLowerCase().includes(term.toLowerCase())) {
				textNodes.push(node);
			}
		}

		for (const node of textNodes) {
			const text = node.textContent ?? '';
			const regex = new RegExp(`(${term.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi');
			const parts = text.split(regex);
			if (parts.length <= 1) continue;

			const accentColor = getComputedStyle(document.documentElement)
				.getPropertyValue('--text-accent')
				.trim();
			const frag = document.createDocumentFragment();
			for (const part of parts) {
				if (part.toLowerCase() === term.toLowerCase()) {
					const mark = document.createElement('span');
					mark.className = MARK_CLASS;
					mark.style.cssText = `color:${accentColor};background:transparent;font-weight:bold`;
					mark.textContent = part;
					marks.push(mark);
					frag.appendChild(mark);
				} else {
					frag.appendChild(document.createTextNode(part));
				}
			}
			node.parentNode?.replaceChild(frag, node);
		}

		matchCount = marks.length;
		if (matchCount > 0) {
			currentMatch = 1;
			scrollToMatch(0);
		}
	}

	function scrollToMatch(idx: number) {
		const accentColor = getComputedStyle(document.documentElement)
			.getPropertyValue('--text-accent')
			.trim();
		const bgColor = getComputedStyle(document.documentElement).getPropertyValue('--bg-base').trim();
		marks.forEach((m) => {
			m.classList.remove(ACTIVE_CLASS);
			m.style.cssText = `color:${accentColor};background:transparent;font-weight:bold`;
		});
		if (idx >= 0 && idx < marks.length) {
			marks[idx].classList.add(ACTIVE_CLASS);
			marks[idx].style.cssText =
				`background:${accentColor};color:${bgColor};border-radius:2px;padding:0 2px;font-weight:bold`;
			marks[idx].scrollIntoView({ block: 'center', behavior: 'smooth' });
		}
	}

	function nextMatch() {
		if (matchCount === 0) return;
		currentMatch = (currentMatch % matchCount) + 1;
		scrollToMatch(currentMatch - 1);
	}

	function prevMatch() {
		if (matchCount === 0) return;
		currentMatch = currentMatch <= 1 ? matchCount : currentMatch - 1;
		scrollToMatch(currentMatch - 1);
	}

	function doSearch() {
		isHighlighting = true;
		clearHighlights();
		if (query.trim().length >= 2) {
			const content = document.querySelector('.z-content') ?? document.body;
			highlightText(content, query.trim());
		}
		isHighlighting = false;
	}

	function toggle() {
		open = !open;
		if (open) {
			requestAnimationFrame(() => inputEl?.focus());
		} else {
			clearHighlights();
			query = '';
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			toggle();
			e.preventDefault();
		} else if (e.key === 'Enter') {
			if (e.shiftKey) prevMatch();
			else nextMatch();
			e.preventDefault();
		}
	}

	let observer: MutationObserver | null = null;
	let debounceTimer: ReturnType<typeof setTimeout> | null = null;
	let isHighlighting = false;

	function startObserving() {
		stopObserving();
		const content = document.querySelector('.z-content') ?? document.body;
		observer = new MutationObserver(() => {
			if (isHighlighting || !open || query.trim().length < 2) return;
			if (debounceTimer) clearTimeout(debounceTimer);
			debounceTimer = setTimeout(() => {
				isHighlighting = true;
				clearHighlights();
				const root = document.querySelector('.z-content') ?? document.body;
				highlightText(root, query.trim());
				isHighlighting = false;
			}, 200);
		});
		observer.observe(content, { childList: true, subtree: true });
	}

	function stopObserving() {
		if (observer) {
			observer.disconnect();
			observer = null;
		}
		if (debounceTimer) {
			clearTimeout(debounceTimer);
			debounceTimer = null;
		}
	}

	$effect(() => {
		if (open && query.trim().length >= 2) {
			startObserving();
		} else {
			stopObserving();
		}
		return () => stopObserving();
	});

	$effect(() => {
		const handler = (e: KeyboardEvent) => {
			if (e.ctrlKey && !e.shiftKey && !e.altKey && e.key.toLowerCase() === 'f') {
				e.preventDefault();
				e.stopImmediatePropagation();
				toggle();
			}
		};
		document.addEventListener('keydown', handler, true);
		return () => document.removeEventListener('keydown', handler, true);
	});
</script>

{#if open}
	<div class="z-global-search">
		<div class="z-search-bar">
			<Icon icon="mdi:magnify" />
			<input
				bind:this={inputEl}
				bind:value={query}
				oninput={doSearch}
				onkeydown={handleKeydown}
				placeholder={i18nState.locale ? m.modListFilters_searchBar_placeholder() : 'Search...'}
			/>
			{#if matchCount > 0}
				<span class="z-search-count">{currentMatch} / {matchCount}</span>
			{/if}
			<button class="z-search-nav" onclick={prevMatch} disabled={matchCount === 0}>
				<Icon icon="mdi:chevron-up" />
			</button>
			<button class="z-search-nav" onclick={nextMatch} disabled={matchCount === 0}>
				<Icon icon="mdi:chevron-down" />
			</button>
			<button class="z-search-close" onclick={toggle}>
				<Icon icon="mdi:close" />
			</button>
		</div>
	</div>
{/if}

<style>
	.z-global-search {
		position: fixed;
		top: 36px;
		right: 16px;
		z-index: 9998;
		animation: slideDown 150ms ease;
	}

	.z-search-bar {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-accent);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-lg);
		font-size: 13px;
		color: var(--text-primary);
		min-width: 320px;
	}

	.z-search-bar input {
		flex: 1;
		background: none;
		border: none;
		outline: none;
		color: var(--text-primary);
		font-family: var(--font-body);
		font-size: 13px;
		min-width: 0;
	}

	.z-search-bar input::placeholder {
		color: var(--text-muted);
	}

	.z-search-count {
		font-size: 11px;
		color: var(--text-muted);
		white-space: nowrap;
		font-family: var(--font-mono);
	}

	.z-search-nav {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		border-radius: var(--radius-sm);
		font-size: 16px;
		transition: all 100ms ease;
	}

	.z-search-nav:hover:not(:disabled) {
		color: var(--text-primary);
		background: var(--bg-hover);
	}

	.z-search-nav:disabled {
		opacity: 0.3;
		cursor: default;
	}

	.z-search-close {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		border-radius: var(--radius-sm);
		font-size: 16px;
		transition: all 100ms ease;
	}

	.z-search-close:hover {
		color: var(--text-primary);
		background: var(--bg-hover);
	}

	@keyframes slideDown {
		from {
			opacity: 0;
			transform: translateY(-8px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
</style>
