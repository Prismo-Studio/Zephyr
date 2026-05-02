<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import * as api from './api';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';
	import { pushToast, pushInfoToast } from '$lib/toast.svelte';
	import { randomizerStore } from './randomizer.store.svelte';
	import { tick } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		onschemaRefreshing: (active: boolean) => void;
	};

	let { onschemaRefreshing }: Props = $props();

	// File extensions for known Archipelago patch formats — kept here rather than
	// in a constants file because they are specific to this picker's filter list.
	const PATCH_EXTENSIONS = [
		'apemerald',
		'apfirered',
		'apleafgreen',
		'appkmnrb',
		'appkmnye',
		'apcrystal',
		'apgold',
		'apsilver',
		'apsms',
		'apmw',
		'apsmz3',
		'aplttp',
		'apz3',
		'apdkc3',
		'apkdl3',
		'apeb',
		'apsoe',
		'apzl',
		'aptloz',
		'aptloz2',
		'aptunic',
		'apladx',
		'aposrs',
		'apterraria'
	];

	let open = $state(false);
	let busy = $state(false);
	let wrapperEl: HTMLDivElement | null = $state(null);
	let menuPos = $state<{ top: number; right: number } | null>(null);

	function updatePos() {
		if (!wrapperEl) return;
		const r = wrapperEl.getBoundingClientRect();
		menuPos = { top: r.bottom + 6, right: window.innerWidth - r.right };
	}

	function handleClickOutside(e: MouseEvent) {
		if (!open) return;
		const target = e.target as Node;
		const inWrapper = wrapperEl?.contains(target);
		const menu = document.getElementById('rdz-qa-menu');
		const inMenu = menu?.contains(target);
		if (!inWrapper && !inMenu) open = false;
	}

	$effect(() => {
		if (open) {
			updatePos();
			document.addEventListener('mousedown', handleClickOutside, true);
			window.addEventListener('resize', updatePos);
			window.addEventListener('scroll', updatePos, true);
		} else {
			menuPos = null;
			document.removeEventListener('mousedown', handleClickOutside, true);
			window.removeEventListener('resize', updatePos);
			window.removeEventListener('scroll', updatePos, true);
		}
		return () => {
			document.removeEventListener('mousedown', handleClickOutside, true);
			window.removeEventListener('resize', updatePos);
			window.removeEventListener('scroll', updatePos, true);
		};
	});

	async function patchGame() {
		open = false;
		const picked = await openDialog({
			filters: [
				{ name: 'Archipelago patch', extensions: PATCH_EXTENSIONS },
				{ name: 'All files', extensions: ['*'] }
			],
			multiple: false
		});
		if (!picked || Array.isArray(picked)) return;
		busy = true;
		try {
			try {
				await api.openConsoleWindow();
				await new Promise((r) => setTimeout(r, 500));
			} catch {
				// Console window may already be open or fail silently — proceed with patch
			}
			await api.applyPatch(picked);
			const fileName = picked.split(/[\\/]/).pop() ?? picked;
			pushInfoToast({ message: m.randomizer_patches_launching({ name: fileName }) });
		} catch (err) {
			pushToast({
				type: 'error',
				name: m.randomizer_patches_applyFailed(),
				message: (err as any)?.message ?? String(err)
			});
		} finally {
			busy = false;
		}
	}

	async function openConsole() {
		open = false;
		busy = true;
		try {
			await api.openConsoleWindow();
		} catch (err) {
			pushToast({
				type: 'error',
				name: m.randomizer_quickActions_launchFailed(),
				message: (err as any)?.message ?? String(err)
			});
		} finally {
			busy = false;
		}
	}

	async function refreshSchemas() {
		open = false;
		busy = true;
		onschemaRefreshing(true);
		await tick();
		await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(() => r(null))));
		const shownAt = performance.now();
		try {
			const res = await api.refreshApworldSchemas();
			if (res.success) {
				pushInfoToast({ message: m.randomizer_customApworlds_refreshed() });
				await randomizerStore.loadCatalog();
				await randomizerStore.reloadCurrentSchema();
			} else {
				pushToast({
					type: 'error',
					name: m.randomizer_customApworlds_refreshFailed(),
					message: (res.stderr || res.stdout || '').slice(0, 300)
				});
			}
		} catch (err) {
			pushToast({
				type: 'error',
				name: m.randomizer_customApworlds_refreshFailed(),
				message: (err as any)?.message ?? String(err)
			});
		} finally {
			// Keep the spinner visible for at least 400 ms so quick refreshes
			// don't flash the loader on/off.
			const elapsed = performance.now() - shownAt;
			if (elapsed < 400) {
				await new Promise((r) => setTimeout(r, 400 - elapsed));
			}
			busy = false;
			onschemaRefreshing(false);
		}
	}
</script>

<div class="rdz-qa-wrapper" bind:this={wrapperEl}>
	<Button size="sm" variant="ghost" onclick={() => (open = !open)} disabled={busy}>
		{#snippet icon()}<Icon icon="mdi:menu" />{/snippet}
		{i18nState.locale && m.randomizer_quickActions_title()}
	</Button>
	{#if open && menuPos}
		<div
			id="rdz-qa-menu"
			class="rdz-qa-menu"
			role="menu"
			style="top: {menuPos.top}px; right: {menuPos.right}px;"
		>
			<button class="rdz-qa-item" role="menuitem" onclick={patchGame} disabled={busy}>
				<Icon icon="mdi:puzzle" />
				<div class="rdz-qa-item-body">
					<span class="rdz-qa-item-title">
						{i18nState.locale && m.randomizer_quickActions_patchGame()}
					</span>
					<span class="rdz-qa-item-desc">
						{i18nState.locale && m.randomizer_quickActions_patchGameDesc()}
					</span>
				</div>
			</button>
			<button class="rdz-qa-item" role="menuitem" onclick={openConsole} disabled={busy}>
				<Icon icon="mdi:console-line" />
				<div class="rdz-qa-item-body">
					<span class="rdz-qa-item-title">
						{i18nState.locale && m.randomizer_quickActions_console()}
					</span>
					<span class="rdz-qa-item-desc">
						{i18nState.locale && m.randomizer_quickActions_consoleDesc()}
					</span>
				</div>
			</button>
			<button class="rdz-qa-item" role="menuitem" onclick={refreshSchemas} disabled={busy}>
				<Icon icon="mdi:database-refresh" />
				<div class="rdz-qa-item-body">
					<span class="rdz-qa-item-title">
						{i18nState.locale && m.randomizer_quickActions_refreshSchemas()}
					</span>
					<span class="rdz-qa-item-desc">
						{i18nState.locale && m.randomizer_quickActions_refreshSchemasDesc()}
					</span>
				</div>
			</button>
		</div>
	{/if}
</div>

<style>
	.rdz-qa-wrapper {
		position: relative;
	}

	:global(#rdz-qa-menu.rdz-qa-menu) {
		position: fixed;
		min-width: 260px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		padding: 4px;
		box-shadow:
			0 8px 24px rgba(0, 0, 0, 0.45),
			0 2px 6px rgba(0, 0, 0, 0.3);
		z-index: 9999;
		animation: rdz-qa-in 150ms ease;
	}

	@keyframes rdz-qa-in {
		from {
			opacity: 0;
			transform: translateY(-4px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.rdz-qa-item {
		display: flex;
		align-items: flex-start;
		gap: var(--space-sm);
		width: 100%;
		padding: var(--space-sm) var(--space-md);
		border: none;
		border-radius: var(--radius-md);
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		text-align: left;
		transition: all var(--transition-fast);
		font-family: var(--font-body);
	}

	.rdz-qa-item:hover:not(:disabled) {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.rdz-qa-item:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.rdz-qa-item :global(svg) {
		font-size: 18px;
		color: var(--accent-400);
		flex-shrink: 0;
		margin-top: 2px;
	}

	.rdz-qa-item-body {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.rdz-qa-item-title {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.rdz-qa-item-desc {
		font-size: 11px;
		color: var(--text-muted);
	}
</style>
