<script lang="ts">
	import Header from '$lib/components/layout/Header.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Toggle from '$lib/components/ui/Toggle.svelte';
	import Dropdown from '$lib/components/ui/Dropdown.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import Icon from '@iconify/svelte';

	import * as api from '$lib/api';
	import type { Prefs } from '$lib/types';
	import { updateAppLanguage, languageTitle, i18nState } from '$lib/i18nCore.svelte';
	import { refreshColor, refreshFont, setFont, getFont, useNativeMenu } from '$lib/themeSystem';
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

	let prefs: Prefs | null = $state(null);
	let currentTheme: ThemeId = $state(getTheme());
	let systemFonts: string[] = $state([]);
	let currentFont = $state(getFont());
	let currentLocale: Locale = $state(getLocale() as Locale);
	let visibleThemes = $state(getVisibleThemes());
	let wingdingsUnlocked = $state(isWingdingsUnlocked());

	let fontOptions = $derived([
		{ value: 'Inter', label: 'Inter (default)' },
		{ value: 'Outfit', label: 'Outfit' },
		{ value: 'DM Sans', label: 'DM Sans' },
		...systemFonts.slice(0, 50).map((f) => ({ value: f, label: f })),
		...(wingdingsUnlocked ? [{ value: 'Wingdings', label: 'Wingdings' }] : [])
	]);

	let languageOptions = $derived(locales.map((l) => ({ value: l, label: languageTitle[l] })));

	onMount(async () => {
		prefs = await api.prefs.get();
		systemFonts = await api.prefs.getSystemFonts();

		window.addEventListener('hotdog-unlocked', () => {
			visibleThemes = getVisibleThemes();
			currentTheme = 'hotdog';
		});

		// Wingdings easter egg — session only, silently adds to dropdown
		window.addEventListener('wingdings-unlocked', () => {
			wingdingsUnlocked = true;
		});
	});

	async function savePrefs() {
		if (prefs) await api.prefs.set(prefs);
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
</script>

<div class="z-settings-page">
	<div class="z-settings-header-wrapper">
		<Header title={i18nState.locale && m.navBar_label_settings()} />
	</div>

	<div class="z-settings-content">
		<!-- Theme -->
		<section class="z-settings-section">
			<h3 class="z-settings-heading">
				<Icon icon="mdi:palette" />
				{i18nState.locale && m.prefs_appearance_title()}
			</h3>
			<div class="z-theme-grid">
				{#each visibleThemes as theme}
					<button
						class="z-theme-option"
						class:active={currentTheme === theme.id}
						onclick={() => switchTheme(theme.id)}
					>
						<div class="z-theme-preview" data-theme={theme.id}>
							<div class="z-theme-dots">
								<span style="background: var(--accent-400)"></span>
								<span style="background: var(--bg-elevated)"></span>
								<span style="background: var(--text-primary)"></span>
							</div>
						</div>
						<span>{theme.label}</span>
					</button>
				{/each}
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
		<section class="z-settings-section">
			<h3 class="z-settings-heading">
				<Icon icon="mdi:wrench" />
				{m.dashboard_quickActions_title()}
			</h3>
			<div class="z-settings-actions">
				<Button variant="secondary" size="sm" onclick={() => clearCache(true)}>
					{#snippet icon()}<Icon icon="mdi:broom" />{/snippet}
					{i18nState.locale && m.menuBar_file_item_6()}
				</Button>
				<Button variant="secondary" size="sm" onclick={() => clearCache(false)}>
					{#snippet icon()}<Icon icon="mdi:delete-sweep" />{/snippet}
					{i18nState.locale && m.menuBar_file_item_5()}
				</Button>
				<Button variant="ghost" size="sm" onclick={openLog}>
					{#snippet icon()}<Icon icon="mdi:file-document" />{/snippet}
					{i18nState.locale && m.menuBar_file_item_4()}
				</Button>
			</div>
		</section>

		<!-- About -->
		<section class="z-settings-section z-about">
			<div class="z-about-brand">
				<span class="z-about-name text-gradient">Zephyr</span>
				<span class="z-about-version">v0.2.0</span>
			</div>
			<p class="z-about-desc">{m.prefs_aboutDesc()}</p>
		</section>
	</div>
</div>

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

	/* Theme grid */
	.z-theme-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: var(--space-md);
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
</style>
