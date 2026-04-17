<script lang="ts">
	import Header from '$lib/components/layout/Header.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Toggle from '$lib/components/ui/Toggle.svelte';
	import Dropdown from '$lib/components/ui/Dropdown.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import Icon from '@iconify/svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';

	import updates from '$lib/state/update.svelte';
	import * as api from '$lib/api';
	import type { Prefs } from '$lib/types';
	import { updateAppLanguage, languageTitle, i18nState } from '$lib/i18nCore.svelte';
	import {
		refreshColor,
		refreshFont,
		setFont,
		getFont,
		useNativeMenu,
		useNativeTitlebar,
		curseForgeEnabled
	} from '$lib/themeSystem';

	import {
		getTheme,
		setTheme,
		getVisibleThemes,
		type ThemeId,
		isWingdingsUnlocked
	} from '$lib/design-system/tokens';
	import { m } from '$lib/paraglide/messages';
	import { getLocale, locales, type Locale } from '$lib/paraglide/runtime';
	import { onMount } from 'svelte';
	import { open as selectDirectory } from '@tauri-apps/plugin-dialog';
	import { pushInfoToast } from '$lib/toast';
	import { shortenFileSize } from '$lib/util';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { getVersion } from '@tauri-apps/api/app';
	import {
		setGamepadEnabled,
		getConnectedGamepad,
		getGamepadDisplayName
	} from '$lib/gamepad.svelte';

	let prefs: Prefs | null = $state(null);
	let showCurseForgeModal = $state(false);
	let gamepadName = $state<string | null>(null);

	const dpiScaleOptions = [
		{ value: '0.75', label: '75%' },
		{ value: '0.85', label: '85%' },
		{ value: '0.9', label: '90%' },
		{ value: '1', label: '100% (default)' },
		{ value: '1.1', label: '110%' },
		{ value: '1.15', label: '115%' },
		{ value: '1.25', label: '125%' },
		{ value: '1.5', label: '150%' },
		{ value: '1.75', label: '175%' },
		{ value: '2', label: '200%' }
	];

	let systemFonts: string[] = $state([]);
	let wingdingsUnlocked = $state(isWingdingsUnlocked());
	let visibleThemes = $state(getVisibleThemes());
	let currentTheme: ThemeId = $state(getTheme());
	let currentFont = $state(getFont());
	let currentLocale: Locale = $state(getLocale() as Locale);

	let fontOptions = $derived([
		{ value: 'Inter', label: 'Inter (default)' },
		{ value: 'Outfit', label: 'Outfit' },
		{ value: 'DM Sans', label: 'DM Sans' },
		...systemFonts.slice(0, 50).map((f: string) => ({ value: f, label: f })),
		...(wingdingsUnlocked ? [{ value: 'Wingdings', label: 'Wingdings' }] : [])
	]);

	let languageOptions = $derived(locales.map((l) => ({ value: l, label: languageTitle[l] })));
	let appVersion: string = $state('');

	function scrollToActiveTheme() {
		const track = document.querySelector<HTMLElement>('.z-theme-carousel');
		if (!track) return;
		const active = track.querySelector<HTMLElement>('.z-theme-option.active');
		if (!active) return;
		const trackRect = track.getBoundingClientRect();
		const activeRect = active.getBoundingClientRect();
		// Distance of active item from track's left edge, accounting for current scroll
		const activeLeftInTrack = activeRect.left - trackRect.left + track.scrollLeft;
		const targetLeft = activeLeftInTrack - (track.clientWidth - active.offsetWidth) / 2;
		track.scrollLeft = Math.max(0, targetLeft);
	}

	onMount(async () => {
		prefs = await api.prefs.get();
		systemFonts = await api.prefs.getSystemFonts();
		getVersion().then((v) => (appVersion = v));

		// Auto-scroll theme carousel to the currently selected theme (instant, not smooth)
		requestAnimationFrame(() => requestAnimationFrame(scrollToActiveTheme));

		// Check if a gamepad is connected
		const gp = getConnectedGamepad();
		if (gp) gamepadName = getGamepadDisplayName(gp);

		// Listen for gamepad connections while on this page
		const onGpConnect = (e: GamepadEvent) => {
			gamepadName = getGamepadDisplayName(e.gamepad.id);
		};
		const onGpDisconnect = () => {
			gamepadName = null;
		};
		window.addEventListener('gamepadconnected', onGpConnect);
		window.addEventListener('gamepaddisconnected', onGpDisconnect);

		window.addEventListener('hotdog-unlocked', () => {
			visibleThemes = getVisibleThemes();
			currentTheme = 'hotdog';
		});

		// Wingdings easter egg - session only, silently adds to dropdown
		window.addEventListener('wingdings-unlocked', () => {
			wingdingsUnlocked = true;
		});
	});

	$effect(() => {
		return () => {
			// Cleanup gamepad listeners handled by $effect teardown
		};
	});

	async function savePrefs() {
		if (prefs) await api.prefs.set(prefs);
	}

	function changeDpiScale(value: string) {
		if (!prefs) return;
		const scale = parseFloat(value);
		if (isNaN(scale)) return;
		prefs.dpiScale = scale;
		savePrefs();
	}

	function toggleGamepad() {
		if (!prefs) return;
		prefs.gamepadEnabled = !prefs.gamepadEnabled;
		setGamepadEnabled(prefs.gamepadEnabled);
		savePrefs();
	}

	function switchTheme(id: ThemeId) {
		currentTheme = id;
		setTheme(id);
	}

	function changeFont(font: string) {
		currentFont = font;
		setFont(font);
	}

	async function changeLanguage(locale: string) {
		currentLocale = locale as Locale;
		if (prefs) {
			prefs.language = locale;
			await savePrefs();
		}
		updateAppLanguage(locale);
	}

	async function clearCache(soft: boolean) {
		const freed = await api.profile.install.clearDownloadCache(soft);
		const messageText = soft
			? m.menuBar_clearModCache_message_unsed
			: m.menuBar_clearModCache_message;
		pushInfoToast({
			message: messageText({ size: shortenFileSize(freed) })
		});
	}

	async function openLog() {
		await api.logger.openZephyrLog();
	}

	async function copyPath(path: string) {
		await writeText(path);
		pushInfoToast({ message: m.prefs_copyPath_success() });
	}

	async function openPath(path: string) {
		await api.prefs.openDir(path);
	}

	async function changeDataDir() {
		if (!prefs) return;
		const selected = await selectDirectory({
			directory: true,
			multiple: false,
			defaultPath: prefs.dataDir
		});

		if (selected && typeof selected === 'string') {
			prefs.dataDir = selected;
			await savePrefs();
		}
	}

	async function changeCacheDir() {
		if (!prefs) return;
		const selected = await selectDirectory({
			directory: true,
			multiple: false,
			defaultPath: prefs.cacheDir
		});

		if (selected && typeof selected === 'string') {
			prefs.cacheDir = selected;
			await savePrefs();
		}
	}

	// ── Update checker ───────────────────────────────────────────────────────

	async function checkForUpdates() {
		await updates.refresh(true);
		if (!updates.next?.available) {
			pushInfoToast({ message: m.aboutDialog_latestVersion() });
		}
	}
</script>

<div class="z-settings-page">
	<div class="z-settings-header-wrapper">
		<Header title={i18nState.locale && m.navBar_label_settings()} />
	</div>

	<div class="z-settings-content">
		<!-- Theme -->
		<section class="z-settings-section">
			<h3 class="z-settings-heading appearance_margin">
				<Icon icon="mdi:palette" />
				{i18nState.locale && m.prefs_appearance_title()}
			</h3>
			<div class="z-theme-carousel-wrapper">
				<button
					class="z-carousel-arrow z-carousel-left"
					onclick={() => {
						const track = document.querySelector('.z-theme-carousel');
						if (track) track.scrollBy({ left: -200, behavior: 'smooth' });
					}}
				>
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
				<button
					class="z-carousel-arrow z-carousel-right"
					onclick={() => {
						const track = document.querySelector('.z-theme-carousel');
						if (track) track.scrollBy({ left: 200, behavior: 'smooth' });
					}}
				>
					<Icon icon="mdi:chevron-right" />
				</button>
			</div>
		</section>

		<!-- Font -->
		<section class="z-settings-section">
			<h3 class="z-settings-heading">
				<Icon icon="mdi:format-font" />
				{i18nState.locale && m.fontFamilyPref_title()}
			</h3>
			<Dropdown
				options={fontOptions}
				value={currentFont}
				onchange={changeFont}
				placeholder={m.fontFamilyPref_placeholder()}
			/>
		</section>

		<!-- Language -->
		<section class="z-settings-section">
			<h3 class="z-settings-heading">
				<Icon icon="mdi:translate" />
				{i18nState.locale && m.languagePref_title()}
			</h3>
			<Dropdown
				options={languageOptions}
				value={currentLocale}
				onchange={changeLanguage}
				placeholder={m.languagePref_title()}
			/>
		</section>

		<!-- Window -->
		<section class="z-settings-section">
			<h3 class="z-settings-heading">
				<Icon icon="mdi:application" />
				{i18nState.locale && m.prefs_window_title()}
			</h3>

			<div class="z-settings-row">
				<div class="z-settings-label">
					<span>{i18nState.locale && m.prefs_window_nativeTitlebar()}</span>
					<span class="z-settings-desc"
						>{i18nState.locale && m.prefs_window_nativeTitlebar_desc()}</span
					>
				</div>
				<Toggle bind:checked={useNativeTitlebar.current} />
			</div>
		</section>

		<!-- Display / DPI Scaling -->
		<section class="z-settings-section">
			<h3 class="z-settings-heading">
				<Icon icon="mdi:monitor" />
				{i18nState.locale && m.prefs_display_title()}
			</h3>

			<div class="z-settings-row">
				<div class="z-settings-label">
					<span>{i18nState.locale && m.prefs_display_dpiScale_title()}</span>
					<span class="z-settings-desc">{i18nState.locale && m.prefs_display_dpiScale_desc()}</span>
				</div>
				{#if prefs}
					<Dropdown
						options={dpiScaleOptions}
						value={String(prefs.dpiScale)}
						onchange={changeDpiScale}
						placeholder="100%"
					/>
				{/if}
			</div>
		</section>

		<!-- Gamepad -->
		<section class="z-settings-section">
			<h3 class="z-settings-heading">
				<Icon icon="mdi:controller" />
				{i18nState.locale && m.prefs_gamepad_title()}
			</h3>

			<div class="z-settings-row">
				<div class="z-settings-label">
					<span>{i18nState.locale && m.prefs_gamepad_enabled_title()}</span>
					<span class="z-settings-desc">{i18nState.locale && m.prefs_gamepad_enabled_desc()}</span>
				</div>
				{#if prefs}
					<Toggle checked={prefs.gamepadEnabled} onchange={toggleGamepad} />
				{/if}
			</div>

			<div class="z-settings-row z-gamepad-status">
				<div class="z-settings-label">
					{#if gamepadName}
						<span class="z-gamepad-connected">
							<Icon icon="mdi:controller" />
							{i18nState.locale && m.prefs_gamepad_connected({ name: gamepadName })}
						</span>
					{:else}
						<span class="z-gamepad-none">
							<Icon icon="mdi:controller-off" />
							{i18nState.locale && m.prefs_gamepad_none()}
						</span>
					{/if}
				</div>
			</div>
		</section>

		<!-- Sources -->
		<section class="z-settings-section">
			<h3 class="z-settings-heading">
				<Icon icon="mdi:store-search" />
				{i18nState.locale && m.prefs_sources_title()}
			</h3>

			<div class="z-settings-row">
				<div class="z-settings-label">
					<span>CurseForge <span class="z-beta-badge">Beta</span></span>
					<span class="z-settings-desc"
						>{i18nState.locale && m.prefs_sources_curseforge_desc()}</span
					>
				</div>
				<Toggle
					checked={curseForgeEnabled.current}
					onchange={() => {
						if (curseForgeEnabled.current) {
							curseForgeEnabled.current = false;
						} else {
							showCurseForgeModal = true;
						}
					}}
				/>
			</div>
		</section>

		{#if prefs}
			<!-- Behavior -->
			<section class="z-settings-section">
				<h3 class="z-settings-heading">
					<Icon icon="mdi:cog" />
					{i18nState.locale && m.prefs_miscellaneous_title()}
				</h3>

				<div class="z-settings-row">
					<div class="z-settings-label">
						<span>{i18nState.locale && m.prefs_miscellaneous_fetchMods_title()}</span>
						<span class="z-settings-desc"
							>{i18nState.locale && m.prefs_miscellaneous_fetchMods_content_1()}</span
						>
					</div>
					<Toggle bind:checked={prefs.fetchModsAutomatically} onchange={savePrefs} />
				</div>

				<div class="z-settings-row">
					<div class="z-settings-label">
						<span>{i18nState.locale && m.prefs_miscellaneous_pullBeforeLaunch_title()}</span>
						<span class="z-settings-desc"
							>{i18nState.locale && m.prefs_miscellaneous_pullBeforeLaunch_content()}</span
						>
					</div>
					<Toggle bind:checked={prefs.pullBeforeLaunch} onchange={savePrefs} />
				</div>
			</section>

			<!-- Paths -->
			<section class="z-settings-section">
				<h3 class="z-settings-heading">
					<Icon icon="mdi:folder" />
					{i18nState.locale && m.prefs_locations_title()}
				</h3>
				<div class="z-settings-path">
					<div class="z-settings-path-header">
						<span class="z-settings-path-label"
							>{i18nState.locale && m.prefs_locations_dataFolder()}</span
						>
						<div class="z-settings-path-actions">
							<button class="z-path-action" onclick={() => copyPath(prefs!.dataDir)} title="Copy">
								<Icon icon="mdi:content-copy" />
							</button>
							<button class="z-path-action" onclick={() => openPath(prefs!.dataDir)} title="Open">
								<Icon icon="mdi:folder-open" />
							</button>
							<button class="z-path-action" onclick={changeDataDir} title="Change">
								<Icon icon="mdi:folder-edit" />
							</button>
						</div>
					</div>
					<code>{prefs.dataDir}</code>
				</div>
				<div class="z-settings-path">
					<div class="z-settings-path-header">
						<span class="z-settings-path-label">{m.prefs_locations_cacheFolder()}</span>
						<div class="z-settings-path-actions">
							<button class="z-path-action" onclick={() => copyPath(prefs!.cacheDir)} title="Copy">
								<Icon icon="mdi:content-copy" />
							</button>
							<button class="z-path-action" onclick={() => openPath(prefs!.cacheDir)} title="Open">
								<Icon icon="mdi:folder-open" />
							</button>
							<button class="z-path-action" onclick={changeCacheDir} title="Change">
								<Icon icon="mdi:folder-edit" />
							</button>
						</div>
					</div>
					<code>{prefs.cacheDir}</code>
				</div>
			</section>
		{/if}

		<!-- Actions -->
		{#key i18nState.locale}
			<section class="z-settings-section">
				<h3 class="z-settings-heading">
					<Icon icon="mdi:wrench" />
					{m.dashboard_quickActions_title()}
				</h3>
				<div class="z-settings-actions">
					<Button variant="secondary" size="sm" onclick={() => clearCache(true)}>
						{#snippet icon()}<Icon icon="mdi:broom" />{/snippet}
						{#snippet children()}{m.menuBar_file_item_6()}{/snippet}
					</Button>
					<Button variant="secondary" size="sm" onclick={() => clearCache(false)}>
						{#snippet icon()}<Icon icon="mdi:delete-sweep" />{/snippet}
						{#snippet children()}{m.menuBar_file_item_5()}{/snippet}
					</Button>
					<Button variant="ghost" size="sm" onclick={openLog}>
						{#snippet icon()}<Icon icon="mdi:file-document" />{/snippet}
						{#snippet children()}{m.menuBar_file_item_4()}{/snippet}
					</Button>
				</div>
			</section>
		{/key}
		<section class="z-settings-section z-about">
			<div class="z-about-brand">
				<span class="z-about-name text-gradient">Zephyr</span>
				<span class="z-about-version">v{appVersion}</span>
			</div>
			<p class="z-about-desc">{m.prefs_aboutDesc()}</p>
			<div class="z-about-actions">
				<Button
					variant="secondary"
					size="sm"
					onclick={checkForUpdates}
					disabled={updates.isChecking}
				>
					{#snippet icon()}
						<Icon
							icon={updates.isChecking ? 'mdi:loading' : 'mdi:update'}
							class={updates.isChecking ? 'z-spin' : ''}
						/>
					{/snippet}
					{i18nState.locale && m.aboutDialog_checkUpdate()}
				</Button>
			</div>
		</section>
	</div>
</div>

{#if showCurseForgeModal}
	<Modal
		bind:open={showCurseForgeModal}
		title="CurseForge"
		onclose={() => {
			showCurseForgeModal = false;
		}}
	>
		{#snippet children()}
			<div class="z-cf-modal">
				<div class="z-cf-modal-header">
					<img src="/logos/curseforge.png" alt="CurseForge" class="z-cf-modal-logo" />
					<div>
						<p class="z-cf-modal-title">{i18nState.locale && m.prefs_curseforge_modal_title()}</p>
						<p class="z-cf-modal-sub">{i18nState.locale && m.prefs_curseforge_modal_desc()}</p>
					</div>
				</div>
				<p class="z-cf-modal-warning">{i18nState.locale && m.prefs_curseforge_modal_warning()}</p>
			</div>
		{/snippet}
		{#snippet actions()}
			<Button
				variant="primary"
				onclick={() => {
					curseForgeEnabled.current = true;
					showCurseForgeModal = false;
				}}
			>
				{#snippet icon()}<Icon icon="mdi:check" />{/snippet}
				{i18nState.locale && m.prefs_curseforge_modal_confirm()}
			</Button>
		{/snippet}
	</Modal>
{/if}

<style>
	.z-settings-page {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.z-settings-content {
		flex: 1;
		overflow-y: auto;
		width: 100%;
		padding-bottom: var(--space-3xl);
	}

	.z-settings-header-wrapper {
		width: 100%;
		max-width: 720px;
		margin: 0 auto;
		padding: var(--space-xl) var(--space-xl) 0;
	}

	.appearance_margin {
		margin-top: var(--space-xl);
	}

	.z-settings-content > * {
		max-width: 720px;
		margin: 0 auto;
		width: 100%;
		padding: 0 var(--space-xl);
	}

	.z-settings-section {
		margin-bottom: var(--space-3xl);
	}

	.z-settings-header-wrapper :global(.z-header-title) {
		font-size: 28px; /* High-DPI bump from 20px */
	}

	.z-settings-heading {
		font-family: var(--font-display);
		font-size: 16px; /* High-DPI bump */
		font-weight: 700;
		color: var(--text-primary);
		display: flex;
		align-items: center;
		gap: var(--space-md);
		margin-bottom: var(--space-xl);
		padding-bottom: var(--space-sm);
		border-bottom: 1px solid var(--border-subtle);
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

	/* Settings rows */
	.z-settings-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-lg) 0;
		border-bottom: 1px solid var(--border-subtle);
	}

	.z-settings-label {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.z-settings-label > span:first-child {
		font-size: 15px; /* High-DPI bump */
		font-weight: 500;
		color: var(--text-primary);
	}

	.z-settings-desc {
		font-size: 12px; /* High-DPI bump */
		color: var(--text-muted);
	}

	/* Paths */
	.z-settings-path {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin-bottom: var(--space-md);
	}

	.z-settings-path-label {
		font-size: 12px;
		color: var(--text-muted);
	}

	.z-settings-path code {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-secondary);
		padding: var(--space-sm) var(--space-md);
		background: var(--bg-elevated);
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-subtle);
		word-break: break-all;
	}

	.z-settings-path-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 4px;
	}

	.z-settings-path-actions {
		display: flex;
		gap: var(--space-xs);
	}

	.z-path-action {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--text-muted);
		border: none;
		cursor: pointer;
		transition: all var(--transition-fast);
		font-size: 14px;
	}

	.z-path-action:hover {
		background: var(--bg-hover);
		color: var(--text-accent);
	}

	/* Actions */
	.z-settings-actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-sm);
	}

	/* About */
	.z-about {
		text-align: center;
		padding-top: var(--space-xl);
	}

	.z-about-brand {
		display: flex;
		align-items: baseline;
		justify-content: center;
		gap: var(--space-sm);
	}

	.z-about-name {
		font-family: var(--font-display);
		font-size: 28px;
		font-weight: 800;
	}

	.z-about-version {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
	}

	.z-about-desc {
		font-size: 13px;
		color: var(--text-muted);
		margin-top: var(--space-sm);
	}

	.z-settings-disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.z-cf-modal {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.z-cf-modal-header {
		display: flex;
		align-items: center;
		gap: var(--space-md);
	}

	.z-cf-modal-logo {
		width: 40px;
		height: 40px;
		border-radius: var(--radius-md);
		flex-shrink: 0;
	}

	.z-cf-modal-title {
		font-size: 14px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.z-cf-modal-sub {
		font-size: 12px;
		color: var(--text-muted);
		margin-top: 2px;
	}

	.z-cf-modal-warning {
		font-size: 13px;
		color: var(--text-secondary);
		line-height: 1.6;
		padding: var(--space-md);
		border-radius: var(--radius-md);
		background: rgba(255, 179, 71, 0.06);
		border: 1px solid rgba(255, 179, 71, 0.15);
	}

	.z-settings-input {
		background: var(--bg-secondary, #1a1a2e);
		border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
		border-radius: var(--radius-md, 6px);
		color: var(--text-primary, #fff);
		padding: 6px 10px;
		font-size: 13px;
		min-width: 220px;
		outline: none;
		transition: border-color 0.15s;
	}

	.z-settings-input:focus {
		border-color: var(--accent-400, #2d8cf0);
	}

	.z-settings-input::placeholder {
		color: var(--text-muted, rgba(255, 255, 255, 0.35));
	}

	.z-beta-badge {
		display: inline-block;
		font-size: 10px;
		font-weight: 700;
		letter-spacing: 0.05em;
		text-transform: uppercase;
		padding: 1px 6px;
		border-radius: 999px;
		background: color-mix(in srgb, var(--accent-400) 20%, transparent);
		color: var(--accent-400);
		border: 1px solid color-mix(in srgb, var(--accent-400) 40%, transparent);
		vertical-align: middle;
		margin-left: 4px;
	}

	/* About actions */
	.z-about-actions {
		display: flex;
		justify-content: center;
		margin-top: var(--space-lg);
	}

	/* Game settings */
	.z-settings-subheading {
		font-size: 12px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-muted);
		margin-bottom: var(--space-sm);
		margin-top: var(--space-xl);
	}

	.z-settings-subheading:first-of-type {
		margin-top: 0;
	}

	.z-path-value {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-secondary);
		padding: var(--space-sm) var(--space-md);
		background: var(--bg-elevated);
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-subtle);
		word-break: break-all;
	}

	/* Number inputs for launch mode */
	.z-settings-number-input {
		width: 72px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-sm);
		color: var(--text-primary);
		padding: 4px 8px;
		font-size: 13px;
		text-align: center;
		outline: none;
		transition: border-color var(--transition-fast);
	}

	.z-settings-number-input:focus {
		border-color: var(--accent-400);
	}

	/* Update modal */
	.z-update-modal {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.z-update-version {
		font-size: 14px;
		color: var(--text-primary);
		font-weight: 500;
	}

	.z-update-desc {
		font-size: 13px;
		color: var(--text-muted);
		line-height: 1.6;
	}

	/* Spin animation for loading icons */
	:global(.z-spin) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}

	/* Gamepad status */
	.z-gamepad-status {
		border-bottom: none;
	}

	.z-gamepad-connected {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-size: 13px;
		color: var(--text-accent);
	}

	.z-gamepad-none {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-size: 13px;
		color: var(--text-muted);
	}
</style>
