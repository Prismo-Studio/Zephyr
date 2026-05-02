<script lang="ts">
	import Icon from '@iconify/svelte';
	import PrefSection from './PrefSection.svelte';
	import { onMount } from 'svelte';
	import { getTheme, setTheme, getVisibleThemes, type ThemeId } from '$lib/design-system/tokens';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		onpickCustom: () => void;
	};

	let { onpickCustom }: Props = $props();

	let visibleThemes = $state(getVisibleThemes());
	let currentTheme: ThemeId = $state(getTheme());

	function switchTheme(id: ThemeId) {
		currentTheme = id;
		setTheme(id);
		if (id === 'custom') onpickCustom();
	}

	function scrollCarousel(direction: 1 | -1) {
		const track = document.querySelector('.z-theme-carousel');
		track?.scrollBy({ left: direction * 200, behavior: 'smooth' });
	}

	function scrollToActiveTheme() {
		const track = document.querySelector<HTMLElement>('.z-theme-carousel');
		if (!track) return;
		const active = track.querySelector<HTMLElement>('.z-theme-option.active');
		if (!active) return;
		const trackRect = track.getBoundingClientRect();
		const activeRect = active.getBoundingClientRect();
		const activeLeftInTrack = activeRect.left - trackRect.left + track.scrollLeft;
		const targetLeft = activeLeftInTrack - (track.clientWidth - active.offsetWidth) / 2;
		track.scrollLeft = Math.max(0, targetLeft);
	}

	onMount(() => {
		// Frame delay so the carousel has laid out before we measure widths
		requestAnimationFrame(() => requestAnimationFrame(scrollToActiveTheme));

		const onHotdog = () => {
			visibleThemes = getVisibleThemes();
			currentTheme = 'hotdog';
		};
		window.addEventListener('hotdog-unlocked', onHotdog);
		return () => window.removeEventListener('hotdog-unlocked', onHotdog);
	});
</script>

<PrefSection
	icon="mdi:palette"
	title={(i18nState.locale && m.prefs_appearance_title()) ?? ''}
	headingClass="appearance_margin"
>
	<div class="z-theme-carousel-wrapper">
		<button class="z-carousel-arrow z-carousel-left" onclick={() => scrollCarousel(-1)}>
			<Icon icon="mdi:chevron-left" />
		</button>
		<div class="z-theme-carousel">
			{#each visibleThemes as theme}
				<button
					class="z-theme-option"
					class:active={currentTheme === theme.id}
					onclick={() => switchTheme(theme.id)}
				>
					<div class="z-theme-preview" data-theme={theme.id}>
						<div class="z-theme-dots">
							<span style="background: var(--accent-400, var(--text-accent))"></span>
							<span style="background: var(--bg-elevated)"></span>
							<span style="background: var(--text-primary)"></span>
						</div>
					</div>
					<span>{theme.label}</span>
				</button>
			{/each}
		</div>
		<button class="z-carousel-arrow z-carousel-right" onclick={() => scrollCarousel(1)}>
			<Icon icon="mdi:chevron-right" />
		</button>
	</div>
</PrefSection>

<style>
	:global(.appearance_margin) {
		margin-top: var(--space-xl);
	}

	.z-theme-carousel-wrapper {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
	}

	.z-carousel-arrow {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border-radius: 50%;
		border: 1px solid var(--border-subtle);
		background: var(--bg-elevated);
		color: var(--text-muted);
		cursor: pointer;
		flex-shrink: 0;
		font-size: 18px;
		transition: all var(--transition-fast);
	}

	.z-carousel-arrow:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
		border-color: var(--border-default);
	}

	.z-theme-carousel {
		display: flex;
		gap: var(--space-md);
		overflow-x: auto;
		scroll-snap-type: x mandatory;
		scrollbar-width: none;
		flex: 1;
	}

	.z-theme-carousel::-webkit-scrollbar {
		display: none;
	}

	.z-theme-option {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-md);
		border-radius: var(--radius-lg);
		border: 2px solid var(--border-subtle);
		background: var(--bg-surface);
		cursor: pointer;
		transition: all var(--transition-fast);
		font-family: var(--font-body);
		font-size: 12px;
		color: var(--text-secondary);
		min-width: calc(25% - var(--space-md));
		flex-shrink: 0;
		scroll-snap-align: start;
	}

	.z-theme-option:hover {
		border-color: var(--border-default);
	}
	.z-theme-option.active {
		border-color: var(--accent-400);
		color: var(--text-accent);
		box-shadow: var(--shadow-glow);
	}

	.z-theme-preview {
		width: 100%;
		height: 40px;
		border-radius: var(--radius-sm);
		background: var(--bg-base);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.z-theme-dots {
		display: flex;
		gap: 6px;
	}

	.z-theme-dots span {
		width: 10px;
		height: 10px;
		border-radius: 50%;
	}
</style>
