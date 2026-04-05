<script lang="ts">
	import Header from '$lib/components/layout/Header.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Dropdown from '$lib/components/ui/Dropdown.svelte';
	import NumberInput from '$lib/components/ui/NumberInput.svelte';
	import Icon from '@iconify/svelte';
	import Input from '$lib/components/ui/Input.svelte';

	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import * as api from '$lib/api';
	import type { ConfigFile, ConfigEntry, ConfigSection, ConfigValue } from '$lib/types';
	import { onMount } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import games from '$lib/state/game.svelte';

	let configFiles: ConfigFile[] = $state([]);
	let selectedFile: ConfigFile | null = $state(null);
	let searchTerm = $state('');
	let noConfigForMod: string | null = $state(null);

	onMount(async () => {
		await refresh();

		const params = new URLSearchParams(window.location.search);

		// Match by exact file path (e.g. /config?file=BepInEx/config/MyMod.cfg)
		const targetFile = params.get('file');
		if (targetFile) {
			const match = configFiles.find(
				(f) => f.relativePath === targetFile || displayName(f) === targetFile
			);
			if (match) selectedFile = match;
		}

		// Match by mod name — finds the config file whose name contains the mod name
		const targetMod = params.get('mod');
		if (targetMod) {
			const lowerMod = targetMod.toLowerCase().replace(/_/g, ' ');
			const match = configFiles.find((f) => {
				const name = displayName(f).toLowerCase().replace(/_/g, ' ');
				return name.includes(lowerMod) || lowerMod.includes(name);
			});
			if (match) {
				selectedFile = match;
			} else {
				noConfigForMod = targetMod.replace(/_/g, ' ');
			}
		}
	});

	// Re-fetch configs when the active game changes
	let lastGameSlug: string | null = null;
	$effect(() => {
		const slug = games.active?.slug ?? null;
		if (slug !== lastGameSlug && lastGameSlug !== null) {
			selectedFile = null;
			refresh();
		}
		lastGameSlug = slug;
	});

	async function refresh() {
		configFiles = await api.config.getFiles();
		if (configFiles.length > 0 && !selectedFile) {
			selectedFile = configFiles[0];
		}
	}

	function displayName(file: ConfigFile): string {
		return file.displayName ?? file.relativePath;
	}

	function configValueStr(value: ConfigValue): string {
		switch (value.type) {
			case 'bool':
				return value.content ? 'true' : 'false';
			case 'string':
				return value.content;
			case 'int':
			case 'float':
				return value.content.value.toString();
			case 'enum':
				return value.content.options[value.content.index];
			case 'flags':
				return value.content.indicies.map((i) => value.content.options[i]).join(', ');
			default:
				return '???';
		}
	}

	async function setEntry(
		file: ConfigFile,
		section: ConfigSection,
		entry: ConfigEntry,
		value: ConfigValue
	) {
		await api.config.setEntry({ file: { relativePath: file.relativePath }, section, entry }, value);
	}

	async function resetEntry(file: ConfigFile, section: ConfigSection, entry: ConfigEntry) {
		const newVal = await api.config.resetEntry({
			file: { relativePath: file.relativePath },
			section,
			entry
		});
		entry.value = newVal;
	}

	async function openFile(file: ConfigFile) {
		await api.config.openFile(file);
	}

	let filteredFiles = $derived(
		searchTerm
			? configFiles.filter((f) => displayName(f).toLowerCase().includes(searchTerm.toLowerCase()))
			: configFiles
	);
</script>

<div class="z-config-page">
	<Header title={i18nState.locale && m.dashboard_quickActions_config()}>
		{#snippet actions()}
			<Tooltip text={i18nState.locale && m.config_launchHint()} position="left" delay={200}>
				<span class="z-config-hint">
					<Icon icon="mdi:information-outline" />
				</span>
			</Tooltip>
			{#if selectedFile}
				<Button variant="ghost" size="sm" onclick={() => openFile(selectedFile!)}>
					{#snippet icon()}<Icon icon="mdi:open-in-new" />{/snippet}
					{i18nState.locale && m.config_openFile()}
				</Button>
			{/if}
		{/snippet}
	</Header>

	<div class="z-config-layout">
		<!-- File list -->
		<div class="z-config-sidebar">
			<div class="z-config-search">
				<Input
					bind:value={searchTerm}
					placeholder={i18nState.locale && m.config_searchPlaceholder()}
				>
					{#snippet iconLeft()}<Icon icon="mdi:magnify" />{/snippet}
				</Input>
			</div>

			<div class="z-config-files">
				{#each filteredFiles as file}
					<button
						class="z-config-file"
						class:active={selectedFile?.relativePath === file.relativePath}
						onclick={() => (selectedFile = file)}
					>
						<Icon
							icon={file.type === 'ok'
								? 'mdi:file-cog'
								: file.type === 'err'
									? 'mdi:file-alert'
									: 'mdi:file-question'}
							class={file.type === 'err' ? 'text-error' : ''}
						/>
						<span>{displayName(file)}</span>
					</button>
				{/each}

				{#if filteredFiles.length === 0}
					<div class="z-config-empty">
						<div class="z-config-empty-icon">
							<Icon icon="mdi:file-search" />
						</div>
						<span class="z-config-empty-title"
							>{i18nState.locale && m.configFileList_noFiles()}</span
						>
						<span class="z-config-empty-hint">{i18nState.locale && m.config_launchHint()}</span>
					</div>
				{/if}
			</div>
		</div>

		<!-- Config editor -->
		<div class="z-config-editor">
			{#if selectedFile}
				{#if selectedFile.type === 'ok'}
					{#each selectedFile.sections as section}
						<div class="z-config-section">
							<h3 class="z-section-title">{section.name}</h3>
							{#each section.entries as entry}
								<div class="z-config-entry">
									<div class="z-entry-header">
										<span class="z-entry-name">{entry.name}</span>
										{#if entry.default}
											<button
												class="z-entry-reset"
												onclick={() => resetEntry(selectedFile!, section, entry)}
												title={i18nState.locale && m.config_resetDefault()}
											>
												<Icon icon="mdi:undo" />
											</button>
										{/if}
									</div>
									{#if entry.description}
										<p class="z-entry-desc">{entry.description}</p>
									{/if}

									<div class="z-entry-value">
										{#if entry.value.type === 'bool'}
											<button
												class="z-bool-toggle"
												class:active={entry.value.content}
												onclick={() => {
													const newVal = { type: 'bool' as const, content: !entry.value.content };
													entry.value = newVal;
													setEntry(selectedFile!, section, entry, newVal);
												}}
											>
												{entry.value.content ? 'ON' : 'OFF'}
											</button>
										{:else if entry.value.type === 'string'}
											<input
												class="z-entry-input"
												type="text"
												value={entry.value.content}
												onchange={(e) => {
													const newVal = {
														type: 'string' as const,
														content: e.currentTarget.value
													};
													entry.value = newVal;
													setEntry(selectedFile!, section, entry, newVal);
												}}
											/>
										{:else if entry.value.type === 'int' || entry.value.type === 'float'}
											{@const range = entry.value.content.range}
											<NumberInput
												value={entry.value.content.value}
												min={range?.start}
												max={range?.end}
												step={entry.value.type === 'float' ? 0.1 : 1}
												onchange={(numVal) => {
													const c = entry.value.content as { value: number; range: { start: number; end: number } | null };
													const newVal = { type: entry.value.type as 'int' | 'float', content: { value: numVal, range: c.range } };
													entry.value = newVal;
													setEntry(selectedFile!, section, entry, newVal);
												}}
											/>
											{#if range}
												<span class="z-entry-range">
													{range.start} — {range.end}
												</span>
											{/if}
										{:else if entry.value.type === 'enum'}
											<Dropdown
												options={entry.value.content.options.map((opt, i) => ({
													value: String(i),
													label: opt
												}))}
												value={String(entry.value.content.index)}
												onchange={(val) => {
													const idx = parseInt(val);
													const c = entry.value.content as { index: number; options: string[] };
													const newVal = {
														type: 'enum' as const,
														content: { index: idx, options: c.options }
													};
													entry.value = newVal;
													setEntry(selectedFile!, section, entry, newVal);
												}}
											/>
										{:else}
											<span class="z-entry-raw">{configValueStr(entry.value)}</span>
										{/if}
									</div>
								</div>
							{/each}
						</div>
					{/each}
				{:else if selectedFile.type === 'err'}
					<div class="z-config-error">
						<Icon icon="mdi:alert-circle" />
						<span>{i18nState.locale && m.config_errorReading({ error: selectedFile.error })}</span>
					</div>
				{:else}
					<div class="z-config-unsupported">
						<Icon icon="mdi:file-question" />
						<span>{i18nState.locale && m.config_unsupported()}</span>
					</div>
				{/if}
			{:else}
				<div class="z-config-placeholder">
					<div class="z-config-placeholder-icon">
						<Icon icon="mdi:file-cog" />
					</div>
					<span class="z-config-placeholder-title">{i18nState.locale && m.config_selectFile()}</span
					>
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.z-config-page {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.z-config-layout {
		display: flex;
		flex: 1;
		overflow: hidden;
	}

	/* Sidebar */
	.z-config-sidebar {
		width: 260px;
		min-width: 260px;
		display: flex;
		flex-direction: column;
		border-right: 1px solid var(--border-subtle);
		background: var(--bg-surface);
	}

	.z-config-search {
		padding: var(--space-md);
		border-bottom: 1px solid var(--border-subtle);
	}

	.z-config-files {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-xs);
	}

	.z-config-file {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		width: 100%;
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-md);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		transition: all var(--transition-fast);
		font-family: var(--font-body);
		font-size: 12px;
		text-align: left;
	}

	.z-config-file:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}
	.z-config-file.active {
		background: var(--bg-active);
		color: var(--text-accent);
	}

	.z-config-file span {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.z-config-hint {
		display: flex;
		align-items: center;
		color: var(--text-muted);
		font-size: 16px;
		cursor: help;
		opacity: 0.6;
		transition: opacity var(--transition-fast);
	}

	.z-config-hint:hover {
		opacity: 1;
		color: var(--warning);
	}

	.z-config-no-mod-banner {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm) var(--space-md);
		margin: 0 var(--space-xl);
		border-radius: var(--radius-md);
		background: rgba(255, 179, 71, 0.08);
		border: 1px solid rgba(255, 179, 71, 0.2);
		color: var(--warning);
		font-size: 12px;
	}

	.z-config-empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-2xl) var(--space-lg);
		text-align: center;
	}

	.z-config-empty-icon {
		width: 48px;
		height: 48px;
		border-radius: var(--radius-xl);
		background: var(--bg-elevated);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 22px;
		color: var(--text-muted);
		margin-bottom: var(--space-xs);
	}

	.z-config-empty-title {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-secondary);
	}

	.z-config-empty-hint {
		font-size: 11px;
		color: var(--text-muted);
		line-height: 1.4;
	}

	/* Editor */
	.z-config-editor {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-xl);
	}

	.z-config-section {
		margin-bottom: var(--space-2xl);
	}

	.z-section-title {
		font-family: var(--font-display);
		font-size: 14px;
		font-weight: 700;
		color: var(--text-primary);
		margin-bottom: var(--space-md);
		padding-bottom: var(--space-sm);
		border-bottom: 1px solid var(--border-subtle);
	}

	.z-config-entry {
		padding: var(--space-md);
		border-radius: var(--radius-md);
		margin-bottom: var(--space-xs);
		transition: background var(--transition-fast);
	}

	.z-config-entry:hover {
		background: var(--bg-hover);
	}

	.z-entry-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.z-entry-name {
		font-weight: 600;
		font-size: 13px;
		color: var(--text-primary);
	}

	.z-entry-reset {
		display: flex;
		align-items: center;
		width: 24px;
		height: 24px;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
		justify-content: center;
	}

	.z-entry-reset:hover {
		background: var(--bg-hover);
		color: var(--text-accent);
	}

	.z-entry-desc {
		font-size: 11px;
		color: var(--text-muted);
		margin-top: 2px;
		line-height: 1.4;
	}

	.z-entry-value {
		margin-top: var(--space-sm);
		display: flex;
		align-items: center;
		gap: var(--space-sm);
	}

	.z-entry-value :global(.z-dropdown-wrapper) {
		width: 200px;
	}

	.z-bool-toggle {
		padding: 4px 16px;
		border-radius: var(--radius-full);
		border: 1px solid var(--border-default);
		background: var(--bg-overlay);
		color: var(--text-muted);
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
		transition: all var(--transition-fast);
		font-family: var(--font-mono);
	}

	.z-bool-toggle.active {
		background: var(--bg-active);
		border-color: var(--border-accent);
		color: var(--text-accent);
	}

	.z-entry-input,
	.z-entry-select {
		padding: 6px 10px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		color: var(--text-primary);
		font-family: var(--font-mono);
		font-size: 12px;
		outline: none;
		transition: border-color var(--transition-fast);
		min-width: 200px;
	}

	.z-entry-input:focus,
	.z-entry-select:focus {
		border-color: var(--accent-400);
	}

	.z-entry-select option {
		background: var(--bg-elevated);
		color: var(--text-primary);
	}

	.z-entry-range {
		font-size: 11px;
		color: var(--text-muted);
		font-family: var(--font-mono);
	}

	.z-entry-raw {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-secondary);
	}

	.z-config-error,
	.z-config-unsupported,
	.z-config-placeholder {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-md);
		padding: var(--space-3xl);
		height: 100%;
	}

	.z-config-placeholder-icon {
		width: 64px;
		height: 64px;
		border-radius: var(--radius-xl);
		background: var(--bg-elevated);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 28px;
		color: var(--text-muted);
	}

	.z-config-placeholder-title {
		font-size: 14px;
		font-weight: 500;
		color: var(--text-muted);
	}

	.z-config-error {
		color: var(--error);
	}

	:global(.text-error) {
		color: var(--error);
	}
</style>
