<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import PythonRuntimeSection from './PythonRuntimeSection.svelte';
	import PlayerSlotsSection from './PlayerSlotsSection.svelte';
	import SeedsSection from './SeedsSection.svelte';
	import HostSection from './HostSection.svelte';
	import { pushToast, pushInfoToast } from '$lib/toast';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';
	import * as api from './api';
	import { randomizerStore } from './randomizer.store.svelte';
	import type { GenerateOutcome, PlayerFile, PythonStatus, SeedFile, ServerStatus } from './types';
	import { onDestroy, onMount, tick } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	export function refresh() {
		refreshAll();
	}

	let openBlocks = $state<Record<string, boolean>>({
		python: true,
		players: true,
		seeds: true,
		host: true
	});
	function toggleBlock(key: string) {
		openBlocks[key] = !openBlocks[key];
	}

	type ConnTarget = 'local' | 'lan' | 'public';
	let connTarget: ConnTarget = $state('local');

	let python: PythonStatus | null = $state(null);
	let players: PlayerFile[] = $state([]);
	let server: ServerStatus | null = $state(null);
	let seeds: SeedFile[] = $state([]);
	let selectedSeed: string | null = $state(null);
	let generateLog: string = $state('');
	let generating = $state(false);
	let starting = $state(false);
	let initialLoading = $state(true);

	let hostMode: 'local' | 'remote' = $state('remote');
	let remoteRoom: api.ArchipelagoGgRoom | null = $state(null);
	let uploading = $state(false);
	let remoteStarting = $state(false);

	let portStr = $state('38281');
	const port = $derived.by(() => {
		const v = parseInt(portStr, 10);
		return isNaN(v) ? 0 : v;
	});
	const portValid = $derived(port >= 1 && port <= 65535);
	let password = $state('');
	let pollHandle: ReturnType<typeof setInterval> | null = null;

	const PRESET_PORTS = [
		{ p: 38281, label: 'AP default' },
		{ p: 443, label: 'HTTPS' },
		{ p: 80, label: 'HTTP' },
		{ p: 25565, label: 'Minecraft' },
		{ p: 7777, label: 'Games' }
	];

	let quickActionsOpen = $state(false);
	let quickActionsEl: HTMLDivElement | null = $state(null);
	let quickActionsBusy = $state(false);
	let schemasRefreshing = $state(false);
	let qaMenuPos = $state<{ top: number; right: number } | null>(null);

	function updateQaMenuPos() {
		if (!quickActionsEl) return;
		const r = quickActionsEl.getBoundingClientRect();
		qaMenuPos = {
			top: r.bottom + 6,
			right: window.innerWidth - r.right
		};
	}

	function closeQuickActions(e: MouseEvent) {
		if (!quickActionsOpen) return;
		const target = e.target as Node;
		const inWrapper = quickActionsEl?.contains(target);
		const menu = document.getElementById('rdz-qa-menu');
		const inMenu = menu?.contains(target);
		if (!inWrapper && !inMenu) {
			quickActionsOpen = false;
		}
	}

	$effect(() => {
		if (quickActionsOpen) {
			updateQaMenuPos();
			document.addEventListener('mousedown', closeQuickActions, true);
			window.addEventListener('resize', updateQaMenuPos);
			window.addEventListener('scroll', updateQaMenuPos, true);
		} else {
			qaMenuPos = null;
			document.removeEventListener('mousedown', closeQuickActions, true);
			window.removeEventListener('resize', updateQaMenuPos);
			window.removeEventListener('scroll', updateQaMenuPos, true);
		}
		return () => {
			document.removeEventListener('mousedown', closeQuickActions, true);
			window.removeEventListener('resize', updateQaMenuPos);
			window.removeEventListener('scroll', updateQaMenuPos, true);
		};
	});

	async function quickPatchGame() {
		quickActionsOpen = false;
		const picked = await openDialog({
			filters: [
				{
					name: 'Archipelago patch',
					extensions: [
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
					]
				},
				{ name: 'All files', extensions: ['*'] }
			],
			multiple: false
		});
		if (!picked || Array.isArray(picked)) return;
		quickActionsBusy = true;
		try {
			try {
				await api.openConsoleWindow();
				await new Promise((r) => setTimeout(r, 500));
			} catch {
				// noop
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
			quickActionsBusy = false;
		}
	}

	async function quickOpenConsole() {
		quickActionsOpen = false;
		quickActionsBusy = true;
		try {
			await api.openConsoleWindow();
		} catch (err) {
			pushToast({
				type: 'error',
				name: m.randomizer_quickActions_launchFailed(),
				message: (err as any)?.message ?? String(err)
			});
		} finally {
			quickActionsBusy = false;
		}
	}

	async function quickRefreshSchemas() {
		quickActionsOpen = false;
		quickActionsBusy = true;
		schemasRefreshing = true;
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
			const elapsed = performance.now() - shownAt;
			if (elapsed < 400) {
				await new Promise((r) => setTimeout(r, 400 - elapsed));
			}
			quickActionsBusy = false;
			schemasRefreshing = false;
		}
	}

	async function refreshAll() {
		try {
			[python, players, server, seeds] = await Promise.all([
				api.checkPython(),
				api.listPlayerYamls(),
				api.serverStatus(),
				api.listSeeds()
			]);
			if (!selectedSeed && seeds.length > 0) {
				selectedSeed = seeds[0].path;
			}
		} finally {
			initialLoading = false;
		}
	}

	onMount(() => {
		refreshAll();
		pollHandle = setInterval(() => {
			api.serverStatus().then((s) => (server = s));
		}, 5000);
	});

	onDestroy(() => {
		if (pollHandle) clearInterval(pollHandle);
		stopPortPoll();
	});

	async function generate() {
		generating = true;
		generateLog = '';
		try {
			const outcome: GenerateOutcome = await api.generateSeed();
			generateLog = (outcome.stdout + '\n' + outcome.stderr).trim();
			if (outcome.success && outcome.zip_path) {
				const playerNames = players.map((p) => p.name).join('_') || 'seed';
				const now = new Date();
				const date = now.toISOString().slice(0, 10).replace(/-/g, '');
				const time = now.toTimeString().slice(0, 8).replace(/:/g, '');
				const friendlyName = `${playerNames}_${date}_${time}`;
				try {
					const newPath = await api.renameSeed(outcome.zip_path, friendlyName);
					selectedSeed = newPath;
				} catch {
					selectedSeed = outcome.zip_path;
				}
			}
			seeds = await api.listSeeds();

			if (outcome.success && selectedSeed && hostMode === 'remote') {
				await uploadAndStartRemote();
			}
		} finally {
			generating = false;
		}
	}

	async function regenerateAndRestart() {
		if (players.length === 0) return;
		await generate();
	}

	let remoteLog: string[] = $state([]);

	let portPollHandle: ReturnType<typeof setInterval> | null = null;

	function stopPortPoll() {
		if (portPollHandle) {
			clearInterval(portPollHandle);
			portPollHandle = null;
		}
	}

	function startPortPoll(roomId: string) {
		stopPortPoll();
		let attempts = 0;
		const maxAttempts = 30;
		portPollHandle = setInterval(async () => {
			attempts += 1;
			try {
				const info = await api.archipelagoGgRoomInfo(roomId);
				if (remoteRoom && remoteRoom.room_id === roomId) {
					let changed = false;
					if (info.port > 0 && remoteRoom.port !== info.port) {
						remoteRoom = { ...remoteRoom, port: info.port };
						remoteLog = [...remoteLog, `Connect at: ${remoteRoom.host}:${info.port}`];
						changed = true;
					}
					if (info.tracker_url && remoteRoom.tracker_url !== info.tracker_url) {
						remoteRoom = { ...remoteRoom, tracker_url: info.tracker_url };
						changed = true;
					}
					if (changed && remoteRoom.port > 0 && remoteRoom.tracker_url) {
						stopPortPoll();
						return;
					}
				}
			} catch {
				// noop
			}
			if (attempts >= maxAttempts) {
				stopPortPoll();
			}
		}, 2000);
	}

	async function uploadAndStartRemote() {
		if (!selectedSeed) return;
		stopPortPoll();
		uploading = true;
		remoteStarting = true;
		remoteLog = [];
		try {
			remoteLog.push(`Uploading ${selectedSeed.split(/[/\\]/).pop()} to archipelago.gg...`);
			const room = await api.archipelagoGgHost(selectedSeed);
			remoteRoom = room;
			remoteLog.push(`Room created: ${room.room_url}`);
			if (room.port > 0) {
				remoteLog.push(`Connect at: ${room.host}:${room.port}`);
			} else {
				remoteLog.push('Waiting for port assignment...');
				startPortPoll(room.room_id);
			}
		} catch (e: any) {
			remoteLog.push(`Error: ${e?.message || e}`);
		} finally {
			uploading = false;
			remoteStarting = false;
		}
	}

	function clearRemoteRoom() {
		stopPortPoll();
		remoteRoom = null;
		remoteLog = [];
	}

	async function startHost() {
		if (!selectedSeed || !portValid) return;
		starting = true;
		try {
			server = await api.startServer(selectedSeed, port, password.trim() || null);
		} catch (e: any) {
			pushToast({
				type: 'error',
				name: m.randomizer_serverError(),
				message: e?.message || String(e)
			});
		} finally {
			starting = false;
		}
	}

	async function stop() {
		await api.stopServer();
		server = await api.serverStatus();
	}

	let renameModal = $state<{ open: boolean; type: 'player' | 'seed'; path: string; value: string }>(
		{
			open: false,
			type: 'player',
			path: '',
			value: ''
		}
	);

	function openRenamePlayer(p: PlayerFile) {
		renameModal = { open: true, type: 'player', path: p.path, value: p.name };
	}

	function openRenameSeed(s: SeedFile) {
		renameModal = {
			open: true,
			type: 'seed',
			path: s.path,
			value: s.name.replace('.archipelago', '')
		};
	}

	async function confirmRename() {
		const { type, path, value } = renameModal;
		if (!value.trim()) return;
		renameModal.open = false;
		if (type === 'player') {
			await api.renamePlayerYaml(path, value.trim());
			players = await api.listPlayerYamls();
			await regenerateAndRestart();
		} else {
			const newPath = await api.renameSeed(path, value.trim());
			if (selectedSeed === path) selectedSeed = newPath;
			seeds = await api.listSeeds();
		}
	}

	async function deletePlayer(p: PlayerFile) {
		await api.deletePlayerYaml(p.path);
		players = await api.listPlayerYamls();
		if (players.length > 0) {
			await regenerateAndRestart();
		}
	}

	async function deleteOneSeed(s: SeedFile) {
		await api.deleteSeed(s.path);
		if (selectedSeed === s.path) selectedSeed = null;
		seeds = await api.listSeeds();
		if (!selectedSeed && seeds.length > 0) selectedSeed = seeds[0].path;
	}

	let confirmClearSeeds = $state(false);

	async function clearAllSeeds() {
		confirmClearSeeds = true;
	}

	async function doConfirmClearSeeds() {
		confirmClearSeeds = false;
		await api.clearSeeds();
		selectedSeed = null;
		seeds = await api.listSeeds();
	}

	let copiedKey: string | null = $state(null);
	let copiedTimer: ReturnType<typeof setTimeout> | null = null;

	function copyText(text: string, key: string) {
		navigator.clipboard.writeText(text).then(() => {
			copiedKey = key;
			if (copiedTimer) clearTimeout(copiedTimer);
			copiedTimer = setTimeout(() => (copiedKey = null), 1500);
		});
	}
</script>

<section class="rdz-server">
	{#if schemasRefreshing}
		<div class="rdz-schema-loading">
			<Icon icon="mdi:loading" class="rdz-schema-spin" />
			<span>{i18nState.locale && m.randomizer_quickActions_refreshSchemas()}…</span>
		</div>
	{/if}
	<header class="rdz-server-header">
		<h2>{i18nState.locale && m.randomizer_serverTitle()}</h2>
		<div class="rdz-qa-wrapper" bind:this={quickActionsEl}>
			<Button
				size="sm"
				variant="ghost"
				onclick={() => (quickActionsOpen = !quickActionsOpen)}
				disabled={quickActionsBusy}
			>
				{#snippet icon()}<Icon icon="mdi:menu" />{/snippet}
				{i18nState.locale && m.randomizer_quickActions_title()}
			</Button>
			{#if quickActionsOpen && qaMenuPos}
				<div
					id="rdz-qa-menu"
					class="rdz-qa-menu"
					role="menu"
					style="top: {qaMenuPos.top}px; right: {qaMenuPos.right}px;"
				>
					<button
						class="rdz-qa-item"
						role="menuitem"
						onclick={quickPatchGame}
						disabled={quickActionsBusy}
					>
						<Icon icon="mdi:puzzle" />
						<div class="rdz-qa-item-body">
							<span class="rdz-qa-item-title"
								>{i18nState.locale && m.randomizer_quickActions_patchGame()}</span
							>
							<span class="rdz-qa-item-desc"
								>{i18nState.locale && m.randomizer_quickActions_patchGameDesc()}</span
							>
						</div>
					</button>
					<button
						class="rdz-qa-item"
						role="menuitem"
						onclick={quickOpenConsole}
						disabled={quickActionsBusy}
					>
						<Icon icon="mdi:console-line" />
						<div class="rdz-qa-item-body">
							<span class="rdz-qa-item-title"
								>{i18nState.locale && m.randomizer_quickActions_console()}</span
							>
							<span class="rdz-qa-item-desc"
								>{i18nState.locale && m.randomizer_quickActions_consoleDesc()}</span
							>
						</div>
					</button>
					<button
						class="rdz-qa-item"
						role="menuitem"
						onclick={quickRefreshSchemas}
						disabled={quickActionsBusy}
					>
						<Icon icon="mdi:database-refresh" />
						<div class="rdz-qa-item-body">
							<span class="rdz-qa-item-title"
								>{i18nState.locale && m.randomizer_quickActions_refreshSchemas()}</span
							>
							<span class="rdz-qa-item-desc"
								>{i18nState.locale && m.randomizer_quickActions_refreshSchemasDesc()}</span
							>
						</div>
					</button>
				</div>
			{/if}
		</div>
		<Button size="sm" variant="ghost" onclick={refreshAll}>
			{#snippet icon()}<Icon icon="mdi:refresh" />{/snippet}
			{i18nState.locale && m.randomizer_refresh()}
		</Button>
	</header>

	<Tooltip
		text={hostMode === 'remote' ? i18nState.locale && m.randomizer_pythonRemoteTooltip() : ''}
		position="top"
		delay={300}
		block
	>
		<div class="rdz-block" class:rdz-block-disabled={hostMode === 'remote'}>
			<button class="rdz-block-header" onclick={() => toggleBlock('python')}>
				<span class="rdz-block-name">{i18nState.locale && m.randomizer_pythonRuntime()}</span>
				{#if hostMode === 'remote'}
					<span class="rdz-block-badge">
						<Icon icon="mdi:laptop" />
						{i18nState.locale && m.randomizer_localOnly()}
					</span>
				{/if}
				<Icon
					icon="mdi:chevron-down"
					class={openBlocks.python ? 'rdz-chev' : 'rdz-chev rdz-chev-closed'}
				/>
			</button>
			{#if openBlocks.python}
				<PythonRuntimeSection {python} />
			{/if}
		</div>
	</Tooltip>

	<div class="rdz-block">
		<div class="rdz-block-row">
			<button class="rdz-block-header" onclick={() => toggleBlock('players')}>
				<span class="rdz-block-name">
					{i18nState.locale && m.randomizer_playerSlots()} <small>({players.length})</small>
				</span>
				<Icon
					icon="mdi:chevron-down"
					class={openBlocks.players ? 'rdz-chev' : 'rdz-chev rdz-chev-closed'}
				/>
			</button>
			<Tooltip text={i18nState.locale && m.randomizer_openFolder()} position="top" delay={200}>
				<button
					class="rdz-icon-btn"
					onclick={() => api.openWorkspaceDir()}
					aria-label="Open folder"
				>
					<Icon icon="mdi:folder-open" />
				</button>
			</Tooltip>
		</div>
		{#if openBlocks.players}
			<PlayerSlotsSection
				{players}
				{initialLoading}
				onRenamePlayer={openRenamePlayer}
				onDeletePlayer={deletePlayer}
			/>
		{/if}
	</div>

	<div class="rdz-block">
		<div class="rdz-block-row">
			<button class="rdz-block-header" onclick={() => toggleBlock('seeds')}>
				<span class="rdz-block-name">
					{i18nState.locale && m.randomizer_seeds()} <small>({seeds.length})</small>
				</span>
				<Icon
					icon="mdi:chevron-down"
					class={openBlocks.seeds ? 'rdz-chev' : 'rdz-chev rdz-chev-closed'}
				/>
			</button>
			{#if seeds.length > 0}
				<Tooltip
					text={i18nState.locale && m.randomizer_deleteAllSeeds()}
					position="top"
					delay={200}
				>
					<button
						class="rdz-icon-btn rdz-icon-danger rdz-clear-all"
						onclick={clearAllSeeds}
						aria-label="Clear all seeds"
					>
						<Icon icon="mdi:delete-sweep" />
					</button>
				</Tooltip>
			{/if}
		</div>

		{#if openBlocks.seeds}
			<SeedsSection
				{seeds}
				{players}
				{python}
				{server}
				bind:selectedSeed
				{generating}
				{generateLog}
				{initialLoading}
				{copiedKey}
				onGenerate={generate}
				onRenameSeed={openRenameSeed}
				onDeleteSeed={deleteOneSeed}
				onCopyText={copyText}
			/>
		{/if}
	</div>

	<div class="rdz-block">
		<button class="rdz-block-header" onclick={() => toggleBlock('host')}>
			<span class="rdz-block-name">{i18nState.locale && m.randomizer_host()}</span>
			<Icon
				icon="mdi:chevron-down"
				class={openBlocks.host ? 'rdz-chev' : 'rdz-chev rdz-chev-closed'}
			/>
		</button>

		{#if openBlocks.host}
			<HostSection
				bind:hostMode
				bind:connTarget
				bind:portStr
				bind:password
				{port}
				{portValid}
				{python}
				{server}
				{selectedSeed}
				{starting}
				{copiedKey}
				{remoteRoom}
				{uploading}
				{remoteStarting}
				{remoteLog}
				{PRESET_PORTS}
				onCopyText={copyText}
				onCopyRemoteLog={() => copyText(remoteLog.join('\n'), 'remote')}
				onUploadAndStartRemote={uploadAndStartRemote}
				onClearRemoteRoom={clearRemoteRoom}
				onStartHost={startHost}
				onStop={stop}
			/>
		{/if}
	</div>
</section>

<Modal
	bind:open={renameModal.open}
	title={renameModal.type === 'player'
		? i18nState.locale && m.randomizer_renamePlayerSlot()
		: i18nState.locale && m.randomizer_renameSeed()}
	onclose={() => (renameModal.open = false)}
>
	<Input
		bind:value={renameModal.value}
		placeholder={i18nState.locale && m.randomizer_newName()}
		onkeydown={(e) => {
			if (e.key === 'Enter') {
				e.preventDefault();
				confirmRename();
			}
		}}
	/>
	{#snippet actions()}
		<Button variant="ghost" onclick={() => (renameModal.open = false)}
			>{i18nState.locale && m.randomizer_cancel()}</Button
		>
		<Button variant="primary" onclick={confirmRename}
			>{i18nState.locale && m.randomizer_rename()}</Button
		>
	{/snippet}
</Modal>

<Modal
	bind:open={confirmClearSeeds}
	title={i18nState.locale && m.randomizer_deleteAllTitle()}
	onclose={() => (confirmClearSeeds = false)}
>
	<p>{i18nState.locale && m.randomizer_deleteAllConfirm()}</p>
	{#snippet actions()}
		<Button variant="ghost" onclick={() => (confirmClearSeeds = false)}
			>{i18nState.locale && m.randomizer_cancel()}</Button
		>
		<Button variant="primary" onclick={doConfirmClearSeeds}
			>{i18nState.locale && m.randomizer_deleteAll()}</Button
		>
	{/snippet}
</Modal>

<style>
	.rdz-server {
		display: flex;
		flex-direction: column;
		gap: 0;
		position: relative;
	}

	.rdz-server :global(.z-btn-primary) {
		background: transparent;
		color: var(--accent-400);
		border: 1px solid var(--accent-400);
		font-weight: 600;
		box-shadow: none;
	}
	.rdz-server :global(.z-btn-primary:hover:not(:disabled)) {
		background: var(--bg-hover);
		color: var(--accent-300);
		border-color: var(--accent-300);
		box-shadow: none;
	}
	.rdz-server :global(.z-btn-primary:disabled) {
		opacity: 0.4;
	}

	.rdz-server-header {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding-bottom: var(--space-sm);
		margin-bottom: var(--space-sm);
		border-bottom: 1px solid var(--border-subtle);
		flex-wrap: wrap;
	}

	.rdz-server-header h2 {
		margin: 0;
		flex: 1 1 auto;
		min-width: 0;
		font-size: 14px;
		font-weight: 600;
		color: var(--text-primary);
		letter-spacing: 0.02em;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

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

	.rdz-schema-loading {
		position: fixed;
		inset: 0;
		z-index: 9999;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-md);
		background: rgba(0, 0, 0, 0.75);
		color: var(--text-primary);
		font-size: 14px;
		font-weight: 600;
	}

	.rdz-schema-loading :global(.rdz-schema-spin) {
		font-size: 48px;
		color: var(--accent-400);
		animation: rdz-spin 0.8s linear infinite;
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

	.rdz-server-header :global(svg) {
		font-size: 20px;
		color: var(--accent-400);
	}

	.rdz-block {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
		padding: var(--space-md) 0;
		border-top: 1px solid var(--border-subtle);
		transition: opacity var(--transition-fast);
		position: relative;
	}

	.rdz-block:first-child {
		border-top: none;
		padding-top: 0;
	}

	.rdz-block-disabled {
		pointer-events: none;
		overflow: hidden;
	}

	.rdz-block-disabled > :not(.rdz-block-row) {
		opacity: 0.45;
	}

	.rdz-block-disabled .rdz-block-badge {
		color: var(--text-primary);
		position: relative;
		z-index: 6;
	}

	.rdz-block-disabled::before {
		content: '';
		position: absolute;
		inset: 0;
		background-image: repeating-linear-gradient(
			135deg,
			transparent,
			transparent 6px,
			rgba(255, 255, 255, 0.04) 6px,
			rgba(255, 255, 255, 0.04) 12px
		);
		pointer-events: none;
		z-index: 5;
	}

	.rdz-block-disabled .rdz-block-title :global(svg:first-child),
	.rdz-block-disabled .rdz-block-title {
		color: var(--text-muted);
	}

	.rdz-block-badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		margin-left: auto;
		padding: 2px 8px;
		font-size: 10px;
		font-weight: 700;
		text-transform: none;
		letter-spacing: 0;
		color: var(--text-muted);
	}

	.rdz-block-row {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		width: 100%;
	}

	.rdz-block-row > :global(.rdz-icon-btn) {
		flex-shrink: 0;
	}

	.rdz-block-header {
		flex: 1;
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		width: 100%;
		padding: 4px 0;
		border: none;
		background: transparent;
		color: var(--text-secondary);
		font-family: var(--font-body);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		cursor: pointer;
		border-radius: 0;
		text-align: left;
		transition: color var(--transition-fast);
	}

	.rdz-block-header:hover {
		color: var(--text-primary);
		background: transparent;
	}

	.rdz-block-name {
		flex: 1;
		color: var(--text-primary);
	}

	.rdz-block-name small {
		color: var(--text-muted);
		font-weight: 600;
		margin-left: 4px;
	}

	:global(.rdz-chev) {
		font-size: 16px;
		color: var(--text-muted);
		transition: transform 180ms ease;
		flex-shrink: 0;
	}

	:global(.rdz-chev-closed) {
		transform: rotate(-90deg);
	}

	:global(.rdz-spin) {
		animation: rdz-spin 1s linear infinite;
	}

	@keyframes rdz-spin {
		to {
			transform: rotate(360deg);
		}
	}

	.rdz-icon-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition:
			color var(--transition-fast),
			background var(--transition-fast);
	}

	.rdz-icon-btn:hover {
		color: var(--text-primary);
		background: var(--bg-hover);
	}

	.rdz-icon-btn.rdz-icon-danger:hover {
		color: var(--error);
		background: transparent;
	}

	.rdz-clear-all {
		margin-left: 0;
	}
</style>
