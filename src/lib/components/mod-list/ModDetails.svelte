<script lang="ts">
	import type { Mod, ModId } from '$lib/types';
	import Icon from '@iconify/svelte';
	import {
		formatModName,
		modIconSrc,
		shortenNum,
		shortenFileSize,
		timeSince,
		getMarkdown
	} from '$lib/util';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Tabs from '$lib/components/ui/Tabs.svelte';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import * as api from '$lib/api';
	import type { Snippet } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import { togglePin, isModPinned } from '$lib/state/misc.svelte';
	import { pushToast } from '$lib/toast';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { open } from '@tauri-apps/plugin-shell';
	import { PersistedState } from 'runed';

	type Props = {
		mod: Mod;
		locked?: boolean;
		showVersionSelector?: boolean;
		onclose?: () => void;
		ontoggle?: () => void;
		onremove?: () => void;
		children?: Snippet;
	};

	let {
		mod,
		locked = false,
		showVersionSelector = true,
		onclose,
		ontoggle,
		onremove,
		children
	}: Props = $props();

	let activeTab = $state('readme');
	let versionDropdownOpen = $state(false);
	let changingVersion = $state(false);
	let confirmVersionChange: {
		open: boolean;
		targetVersion: { name: string; uuid: string } | null;
	} = $state({ open: false, targetVersion: null });
	let dontAskAgain = $state(false);
	const skipVersionConfirm = new PersistedState('skipVersionConfirm', false);

	async function selectVersion(version: { name: string; uuid: string }) {
		if (changingVersion) return;
		if (version.uuid === mod.versionUuid) {
			versionDropdownOpen = false;
			return;
		}

		versionDropdownOpen = false;

		if (skipVersionConfirm.current) {
			await doChangeVersion(version);
		} else {
			confirmVersionChange = { open: true, targetVersion: version };
			dontAskAgain = false;
		}
	}

	async function doChangeVersion(version: { name: string; uuid: string }) {
		if (changingVersion) return;
		changingVersion = true;
		confirmVersionChange = { open: false, targetVersion: null };
		try {
			if (mod.isInstalled && !isExternalMod(mod)) {
				await api.profile.forceRemoveMods([mod.uuid]);
			}
			await api.profile.install.mod({
				packageUuid: mod.uuid,
				versionUuid: version.uuid
			});
			pushToast({
				type: 'info',
				message:
					(i18nState.locale &&
						m.modDetails_versionChange_success({
							mod: formatModName(mod.name),
							version: version.name
						})) ||
					`${formatModName(mod.name)} changed to ${version.name}`
			});
		} catch {
			// Error toast shown by invoke wrapper
		} finally {
			changingVersion = false;
		}
	}

	function confirmAndChange() {
		if (!confirmVersionChange.targetVersion || changingVersion) return;
		if (dontAskAgain) skipVersionConfirm.current = true;
		doChangeVersion(confirmVersionChange.targetVersion);
	}
	let markdown = $state('');
	let loadingMarkdown = $state(false);
	let copied = $state(false);
	let copyTimeoutId: number | null = null;

	let tabs = $derived([
		{ id: 'readme', label: i18nState.locale && m.modpack_readme_title() },
		{ id: 'changelog', label: i18nState.locale && m.modpack_changeLog_title() }
	]);

	function isExternalMod(m: Mod): boolean {
		return m.uuid.includes(':');
	}

	async function loadMarkdown(type: 'readme' | 'changelog') {
		loadingMarkdown = true;
		try {
			if (isExternalMod(mod)) {
				markdown = type === 'readme' ? (mod.description ?? '') : '';
			} else {
				const result = await getMarkdown(mod, type);
				markdown = result ?? '';
			}
		} catch {
			markdown = '';
		}
		loadingMarkdown = false;
	}

	const copyContent = async () => {
		try {
			await writeText(markdown);
			copied = true;
			if (copyTimeoutId !== null) clearTimeout(copyTimeoutId);
			copyTimeoutId = window.setTimeout(() => {
				copied = false;
			}, 2000);
		} catch (err) {
			console.error('Failed to copy:', err);
		}
	};

	$effect(() => {
		if (mod) {
			loadMarkdown(activeTab as 'readme' | 'changelog');
		}
	});
</script>

<div class="z-mod-details">
	<!-- Header -->
	<div class="z-details-header">
		<button class="z-details-close" onclick={onclose}>
			<Icon icon="mdi:close" />
		</button>

		<div class="z-details-hero">
			<img src={modIconSrc(mod)} alt={mod.name} class="z-details-icon" />
			<div class="z-details-title">
				<div class="z-details-name-row">
					<h2>{formatModName(mod.name)}</h2>
					{#if mod.isInstalled && !isExternalMod(mod)}
						<button
							class="z-pin-toggle"
							class:pinned={isModPinned(mod.uuid)}
							onclick={() => togglePin(mod.uuid)}
							title={isModPinned(mod.uuid) ? 'Désépingler' : 'Épingler'}
						>
							<Icon icon={isModPinned(mod.uuid) ? 'mdi:pin' : 'mdi:pin-outline'} />
						</button>
					{/if}
				</div>
				{#if mod.author}
					<span class="z-details-author">{i18nState.locale && m.modDetails_by()} {mod.author}</span>
				{/if}
			</div>
		</div>

		<!-- Badges -->
		<div class="z-details-badges">
			{#if mod.version && mod.versions.length > 1 && mod.isInstalled && showVersionSelector && !isExternalMod(mod)}
				<div class="z-version-selector">
					<button
						class="z-version-btn"
						disabled={changingVersion}
						onclick={() => (versionDropdownOpen = !versionDropdownOpen)}
					>
						<Icon icon="mdi:tag" />
						<span>{mod.version}</span>
						<Icon
							icon="mdi:chevron-down"
							class="z-version-chevron {versionDropdownOpen ? 'open' : ''}"
						/>
					</button>

					{#if versionDropdownOpen}
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div class="z-version-dropdown" onmouseleave={() => (versionDropdownOpen = false)}>
							{#each mod.versions as version}
								<button
									class="z-version-option"
									class:active={version.name === mod.version}
									onclick={() => selectVersion(version)}
								>
									{#if version.name === mod.version}
										<Icon icon="mdi:check" />
									{/if}
									<span>{version.name}</span>
									{#if version.name === mod.versions[0].name}
										<Badge variant="accent">{i18nState.locale && m.modDetails_latest()}</Badge>
									{/if}
								</button>
							{/each}
						</div>
					{/if}
				</div>
			{:else if mod.version}
				<Badge variant="accent">{mod.version}</Badge>
			{/if}
			{#if mod.isInstalled}
				<Badge variant="success">{i18nState.locale && m.modDetails_installed()}</Badge>
			{/if}
			{#if mod.isDeprecated}
				<Badge variant="error">{i18nState.locale && m.modDetails_deprecated()}</Badge>
			{/if}
		</div>

		<!-- Stats -->
		<div class="z-details-stats">
			{#if mod.downloads != null}
				<div class="z-stat">
					<Icon icon="mdi:download" />
					<span>{shortenNum(mod.downloads)}</span>
				</div>
			{/if}
			{#if mod.rating != null}
				<div class="z-stat">
					<Icon icon="mdi:thumb-up" />
					<span>{shortenNum(mod.rating)}</span>
				</div>
			{/if}
			<div class="z-stat">
				<Icon icon="mdi:file" />
				<span>{shortenFileSize(mod.fileSize)}</span>
			</div>
			{#if mod.lastUpdated}
				<div class="z-stat">
					<Icon icon="mdi:clock-outline" />
					<span>{timeSince(mod.lastUpdated)}</span>
				</div>
			{/if}
		</div>

		<!-- Action buttons -->
		{#if mod.isInstalled && !isExternalMod(mod)}
			<div class="z-details-actions">
				<Tooltip
					text={i18nState.locale &&
						(mod.enabled === false ? m.modDetails_enable() : m.modDetails_disable())}
					position="bottom"
					delay={300}
				>
					<button class="z-action-btn" class:disabled={locked} disabled={locked} onclick={ontoggle}>
						<Icon icon={mod.enabled === false ? 'mdi:eye' : 'mdi:eye-off'} />
						<span class="z-action-label"
							>{i18nState.locale &&
								(mod.enabled === false ? m.modDetails_enable() : m.modDetails_disable())}</span
						>
					</button>
				</Tooltip>

				<Tooltip text={i18nState.locale && m.modDetails_openFolder()} position="bottom" delay={300}>
					<button class="z-action-btn" onclick={() => api.profile.openModDir(mod.uuid)}>
						<Icon icon="mdi:folder-open" />
						<span class="z-action-label">{i18nState.locale && m.modDetails_openFolder()}</span>
					</button>
				</Tooltip>

				<Tooltip text={i18nState.locale && m.modDetails_uninstall()} position="bottom" delay={300}>
					<button
						class="z-action-btn danger"
						class:disabled={locked}
						disabled={locked}
						onclick={onremove}
					>
						<Icon icon="mdi:delete" />
						<span class="z-action-label">{i18nState.locale && m.modDetails_uninstall()}</span>
					</button>
				</Tooltip>
			</div>
		{/if}

		<!-- Install button slot -->
		{#if children}{@render children()}{/if}

		{#if isExternalMod(mod) && mod.websiteUrl}
			<a
				href={mod.websiteUrl}
				target="_blank"
				rel="noopener"
				class="z-external-link-btn"
				onclick={(e) => {
					e.preventDefault();
					open(mod.websiteUrl!);
				}}
			>
				<Icon icon="mdi:open-in-new" />
				<span>{i18nState.locale && m.mods_contextMenu_openThunderstore()}</span>
			</a>
		{/if}
	</div>

	<!-- Tabs + Content -->
	<div class="z-details-content">
		<div class="z-details-tabs-row">
			<Tabs
				{tabs}
				bind:active={activeTab}
				onchange={(id) => loadMarkdown(id as 'readme' | 'changelog')}
			/>
			{#if markdown && !loadingMarkdown}
				<button class="z-copy-btn" class:copied onclick={copyContent} title="Copy content">
					<Icon icon={copied ? 'mdi:check' : 'mdi:content-copy'} />
					<span class="z-copy-text">{copied ? 'Copied!' : 'Copy'}</span>
				</button>
			{/if}
		</div>

		<div class="z-details-body">
			{#if loadingMarkdown}
				<div class="z-details-loading">
					<Spinner size={20} />
				</div>
			{:else if markdown}
				<div class="markdown">
					{@html markdown}
				</div>
			{:else}
				<p class="z-details-empty">{i18nState.locale && m.modDetails_noContent()}</p>
			{/if}
		</div>
	</div>
</div>

{#if confirmVersionChange.open && confirmVersionChange.targetVersion}
	<Modal
		bind:open={confirmVersionChange.open}
		title={i18nState.locale && m.modDetails_versionChange_title()}
		onclose={() => (confirmVersionChange = { open: false, targetVersion: null })}
	>
		{#snippet children()}
			<div class="z-version-confirm-body">
				<p>
					{i18nState.locale &&
						m.modDetails_versionChange_desc({
							mod: formatModName(mod.name),
							from: mod.version ?? '',
							to: confirmVersionChange.targetVersion?.name ?? ''
						})}
				</p>
				<label class="z-version-dont-ask">
					<Checkbox bind:checked={dontAskAgain} />
					<span>{i18nState.locale && m.modDetails_versionChange_dontAsk()}</span>
				</label>
			</div>
		{/snippet}

		{#snippet actions()}
			<Button
				variant="ghost"
				onclick={() => (confirmVersionChange = { open: false, targetVersion: null })}
			>
				{i18nState.locale && m.modDetails_versionChange_cancel()}
			</Button>
			<Button variant="primary" onclick={confirmAndChange}>
				{#snippet icon()}<Icon icon="mdi:swap-vertical" />{/snippet}
				{i18nState.locale && m.modDetails_versionChange_confirm()}
			</Button>
		{/snippet}
	</Modal>
{/if}

<style>
	.z-mod-details {
		width: 40%;
		min-width: 320px;
		max-width: 480px;
		height: 100%;
		background: var(--bg-surface);
		border-left: 1px solid var(--border-subtle);
		display: flex;
		flex-direction: column;
		overflow: visible;
		animation: slideIn var(--transition-normal) ease;
		container-type: inline-size;
	}

	@keyframes slideIn {
		from {
			transform: translateX(20px);
			opacity: 0;
		}
		to {
			transform: translateX(0);
			opacity: 1;
		}
	}

	.z-details-header {
		padding: var(--space-xl);
		border-bottom: 1px solid var(--border-subtle);
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		position: relative;
		overflow: visible;
		z-index: 1;
	}

	.z-details-close {
		position: absolute;
		top: var(--space-md);
		right: var(--space-md);
		width: 28px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.z-details-close:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.z-details-hero {
		display: flex;
		align-items: center;
		gap: var(--space-lg);
	}

	.z-details-icon {
		width: 64px;
		height: 64px;
		border-radius: var(--radius-lg);
		object-fit: cover;
		background: var(--bg-overlay);
		border: 1px solid var(--border-subtle);
	}

	.z-details-name-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.z-details-title h2 {
		font-family: var(--font-display);
		font-size: 18px;
		font-weight: 800;
		color: var(--text-primary);
		letter-spacing: -0.02em;
	}

	.z-pin-toggle {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
		flex-shrink: 0;
		font-size: 16px;
	}

	.z-pin-toggle:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.z-pin-toggle.pinned {
		color: var(--text-accent);
	}

	.z-pin-toggle.pinned:hover {
		color: var(--error);
		background: rgba(255, 92, 92, 0.1);
	}

	.z-details-author {
		font-size: 13px;
		color: var(--text-muted);
	}

	.z-details-badges {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.z-details-stats {
		display: flex;
		gap: var(--space-lg);
		font-size: 12px;
		color: var(--text-muted);
	}

	.z-stat {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.z-details-actions {
		display: flex;
		gap: 6px;
	}

	.z-action-btn {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-md);
		border: 1px solid var(--border-subtle);
		background: var(--bg-elevated);
		color: var(--text-secondary);
		font-family: var(--font-body);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all var(--transition-fast);
		white-space: nowrap;
		overflow: hidden;
		min-width: 36px;
	}

	.z-action-label {
		overflow: hidden;
		text-overflow: ellipsis;
	}

	@container (max-width: 380px) {
		.z-action-label {
			display: none;
		}
	}

	.z-action-btn:hover:not(:disabled) {
		background: var(--bg-hover);
		border-color: var(--border-default);
		color: var(--text-primary);
	}

	.z-action-btn.danger:hover:not(:disabled) {
		background: rgba(255, 92, 92, 0.1);
		border-color: rgba(255, 92, 92, 0.3);
		color: var(--error);
	}

	.z-action-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.z-details-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		padding: var(--space-lg);
		gap: var(--space-md);
	}

	.z-details-tabs-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-md);
	}

	.z-copy-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-subtle);
		background: var(--bg-elevated);
		color: var(--text-secondary);
		font-family: var(--font-body);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all var(--transition-fast);
		white-space: nowrap;
	}

	.z-copy-btn:hover:not(.copied) {
		background: var(--bg-hover);
		border-color: var(--border-default);
		color: var(--text-primary);
	}

	.z-copy-btn.copied {
		background: rgba(0, 212, 170, 0.1);
		border-color: rgba(0, 212, 170, 0.3);
		color: var(--success);
	}

	.z-copy-text {
		display: none;
	}

	@media (min-width: 420px) {
		.z-copy-text {
			display: inline;
		}
	}

	.z-details-body {
		flex: 1;
		overflow-y: auto;
		padding-top: var(--space-sm);
	}

	.z-details-loading {
		display: flex;
		justify-content: center;
		padding: var(--space-2xl);
		color: var(--text-muted);
	}

	.z-details-empty {
		text-align: center;
		color: var(--text-muted);
		font-size: 13px;
		padding: var(--space-2xl);
	}

	.z-external-link-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-sm);
		padding: var(--space-sm) var(--space-lg);
		border-radius: var(--radius-md);
		background: var(--bg-active);
		color: var(--text-accent);
		font-size: 13px;
		font-weight: 600;
		text-decoration: none;
		cursor: pointer;
		transition: all var(--transition-fast);
		border: 1px solid var(--border-accent);
	}

	.z-external-link-btn:hover {
		background: rgba(26, 255, 250, 0.12);
	}

	/* Version selector */
	.z-version-selector {
		position: relative;
	}

	.z-version-btn {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 3px 8px;
		border-radius: var(--radius-full);
		border: 1px solid var(--accent-400);
		background: rgba(26, 255, 250, 0.08);
		color: var(--accent-400);
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.z-version-btn:hover {
		background: rgba(26, 255, 250, 0.15);
	}

	:global(.z-version-chevron) {
		font-size: 14px;
		transition: transform 150ms ease;
	}

	:global(.z-version-chevron.open) {
		transform: rotate(180deg);
	}

	.z-version-dropdown {
		position: absolute;
		top: 100%;
		left: 0;
		margin-top: 4px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		padding: var(--space-xs);
		min-width: 160px;
		max-height: 240px;
		overflow-y: auto;
		z-index: var(--z-dropdown);
		box-shadow: var(--shadow-lg);
	}

	.z-version-option {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		width: 100%;
		padding: var(--space-xs) var(--space-sm);
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		font-size: 12px;
		cursor: pointer;
		transition: all var(--transition-fast);
		font-family: var(--font-body);
	}

	.z-version-option:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.z-version-option.active {
		color: var(--text-accent);
	}

	/* Version change confirm */
	.z-version-confirm-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		font-size: 13px;
		color: var(--text-secondary);
		line-height: 1.5;
	}

	.z-version-dont-ask {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-size: 12px;
		color: var(--text-muted);
		cursor: pointer;
	}
</style>
