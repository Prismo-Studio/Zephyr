<script lang="ts">
	import Header from '$lib/components/layout/Header.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Toggle from '$lib/components/ui/Toggle.svelte';
	import Dropdown from '$lib/components/ui/Dropdown.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import Icon from '@iconify/svelte';

	import * as api from '$lib/api';
	import type { Prefs } from '$lib/types';
	import { getTheme, setTheme, type ThemeId, getVisibleThemes } from '$lib/design-system/tokens';
	import { setFont, getFont } from '$lib/theme';
	import { setLanguage, languageTitle } from '$lib/i18n';
	import { getLocale, locales, type Locale } from '$lib/paraglide/runtime';
	import { onMount } from 'svelte';
	import { pushInfoToast } from '$lib/toast';
	import { shortenFileSize } from '$lib/util';
	import { m } from '$lib/paraglide/messages';

	let prefs: Prefs | null = $state(null);
	let currentTheme: ThemeId = $state(getTheme());
	let systemFonts: string[] = $state([]);
	let currentFont = $state(getFont());
	let currentLocale: Locale = $state(getLocale() as Locale);
	let visibleThemes = $state(getVisibleThemes());

	let fontOptions = $derived([
		{ value: 'Inter', label: 'Inter (default)' },
		{ value: 'Outfit', label: 'Outfit' },
		{ value: 'DM Sans', label: 'DM Sans' },
		...systemFonts.slice(0, 50).map((f) => ({ value: f, label: f }))
	]);

	let languageOptions = $derived(locales.map((l) => ({ value: l, label: languageTitle[l] })));

	onMount(async () => {
		prefs = await api.prefs.get();
		systemFonts = await api.prefs.getSystemFonts();

		window.addEventListener('hotdog-unlocked', () => {
			visibleThemes = getVisibleThemes();
			currentTheme = 'hotdog';
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
		// setLanguage doit être appelé APRÈS savePrefs
		// car setLocale() trigger un reload de page,
		// et refreshLanguage() au mount lira prefs.language
		setLanguage(locale);
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
</script>

<div class="z-settings-page">
	<Header title={m.navBar_label_settings()} />

	<div class="z-settings-content">
		<!-- Theme -->
		<section class="z-settings-section">
			<h3 class="z-settings-heading">
				<Icon icon="mdi:palette" />
				{m.prefs_appearance_title()}
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
				{m.fontFamilyPref_title()}
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
				{m.languagePref_title()}
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
					{m.prefs_miscellaneous_title()}
				</h3>

				<div class="z-settings-row">
					<div class="z-settings-label">
						<span>{m.prefs_miscellaneous_fetchMods_title()}</span>
						<span class="z-settings-desc">{m.prefs_miscellaneous_fetchMods_content_1()}</span>
					</div>
					<Toggle bind:checked={prefs.fetchModsAutomatically} onchange={savePrefs} />
				</div>

				<div class="z-settings-row">
					<div class="z-settings-label">
						<span>{m.prefs_miscellaneous_pullBeforeLaunch_title()}</span>
						<span class="z-settings-desc">{m.prefs_miscellaneous_pullBeforeLaunch_content()}</span>
					</div>
					<Toggle bind:checked={prefs.pullBeforeLaunch} onchange={savePrefs} />
				</div>
			</section>

			<!-- Paths -->
			<section class="z-settings-section">
				<h3 class="z-settings-heading">
					<Icon icon="mdi:folder" />
					{m.prefs_locations_title()}
				</h3>
				<div class="z-settings-path">
					<span class="z-settings-path-label">{m.prefs_locations_dataFolder()}</span>
					<code>{prefs.dataDir}</code>
				</div>
				<div class="z-settings-path">
					<span class="z-settings-path-label">{m.prefs_locations_cacheFolder()}</span>
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
					{m.menuBar_file_item_6()}
				</Button>
				<Button variant="secondary" size="sm" onclick={() => clearCache(false)}>
					{#snippet icon()}<Icon icon="mdi:delete-sweep" />{/snippet}
					{m.menuBar_file_item_5()}
				</Button>
				<Button variant="ghost" size="sm" onclick={openLog}>
					{#snippet icon()}<Icon icon="mdi:file-document" />{/snippet}
					{m.menuBar_file_item_4()}
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
		padding: 0 var(--space-xl) var(--space-2xl);
		max-width: 640px;
	}

	.z-settings-section {
		margin-bottom: var(--space-2xl);
	}

	.z-settings-heading {
		font-family: var(--font-display);
		font-size: 14px;
		font-weight: 700;
		color: var(--text-primary);
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		margin-bottom: var(--space-lg);
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
		padding: var(--space-md) 0;
		border-bottom: 1px solid var(--border-subtle);
	}

	.z-settings-label {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.z-settings-label > span:first-child {
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
	}

	.z-settings-desc {
		font-size: 11px;
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
