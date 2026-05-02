<script lang="ts">
	import LaunchOverlay from '$lib/components/dialogs/LaunchOverlay.svelte';
	import PlayMenuPopover from './PlayMenuPopover.svelte';
	import * as api from '$lib/api';
	import { launchGameWithBepInExFallback } from '$lib/launch';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	let launching = $state(false);
	let menuOpen = $state(false);
	let btnEl: HTMLButtonElement | null = $state(null);
	let menuPos = $state<{ bottom: number; left: number } | null>(null);

	function toggleMenu() {
		if (menuOpen) {
			menuOpen = false;
			return;
		}
		if (!btnEl) return;
		const r = btnEl.getBoundingClientRect();
		menuPos = { bottom: window.innerHeight - r.bottom, left: r.right + 10 };
		menuOpen = true;
	}

	function handleClickOutside(e: MouseEvent) {
		if (!menuOpen) return;
		const target = e.target as Node;
		if (btnEl?.contains(target)) return;
		const menu = document.getElementById('z-play-menu');
		if (menu?.contains(target)) return;
		menuOpen = false;
	}

	$effect(() => {
		if (menuOpen) {
			document.addEventListener('mousedown', handleClickOutside, true);
			return () => document.removeEventListener('mousedown', handleClickOutside, true);
		}
	});

	async function launchModded() {
		menuOpen = false;
		launching = true;
		try {
			await launchGameWithBepInExFallback();
		} finally {
			launching = false;
		}
	}

	async function launchVanilla() {
		menuOpen = false;
		launching = true;
		try {
			await api.profile.launch.launchVanilla();
		} catch {
			launching = false;
		}
	}
</script>

<button
	class="z-launch-btn"
	class:open={menuOpen}
	onclick={toggleMenu}
	bind:this={btnEl}
	aria-haspopup="menu"
	aria-expanded={menuOpen}
	aria-label={i18nState.locale && m.toolBar_launchGame_button()}
>
	<svg width="28" height="28" viewBox="0 0 24 24" fill="currentColor" class="z-custom-launch-icon">
		<path d="M8 5v14l11-7z" />
	</svg>
</button>

{#if menuOpen && menuPos}
	<PlayMenuPopover
		bottom={menuPos.bottom}
		left={menuPos.left}
		onlaunchModded={launchModded}
		onlaunchVanilla={launchVanilla}
	/>
{/if}

<LaunchOverlay bind:visible={launching} onclose={() => (launching = false)} />

<style>
	.z-launch-btn {
		width: 44px;
		height: 44px;
		border-radius: var(--radius-lg);
		background: transparent;
		border: 2px solid var(--accent-400);
		color: var(--accent-400);
		font-size: 22px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all var(--transition-normal);
	}

	.z-launch-btn:hover {
		background: var(--bg-active);
		transform: translateY(-2px);
		box-shadow: var(--shadow-glow);
	}

	.z-launch-btn:active {
		transform: scale(0.97);
	}

	.z-launch-btn.open {
		box-shadow: 0 0 0 2px var(--accent-400);
	}

	.z-custom-launch-icon {
		width: 28px;
		height: 28px;
	}
</style>
