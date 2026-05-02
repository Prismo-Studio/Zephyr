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
	import RandomizerQuickActions from './RandomizerQuickActions.svelte';
	import CollapsibleBlock from './CollapsibleBlock.svelte';

	import { pushToast } from '$lib/toast.svelte';
	import * as api from './api';
	import type { GenerateOutcome, PlayerFile, PythonStatus, SeedFile, ServerStatus } from './types';
	import { onDestroy, onMount } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	export function refresh() {
		refreshAll();
	}

	const PRESET_PORTS = [
		{ p: 38281, label: 'AP default' },
		{ p: 443, label: 'HTTPS' },
		{ p: 80, label: 'HTTP' },
		{ p: 25565, label: 'Minecraft' },
		{ p: 7777, label: 'Games' }
	];

	const SERVER_POLL_MS = 5000;
	const PORT_POLL_MS = 2000;
	const PORT_POLL_MAX_ATTEMPTS = 30;

	type ConnTarget = 'local' | 'lan' | 'public';

	let openBlocks = $state<Record<string, boolean>>({
		python: true,
		players: true,
		seeds: true,
		host: true
	});
	function toggleBlock(key: string) {
		openBlocks[key] = !openBlocks[key];
	}

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
	let schemasRefreshing = $state(false);

	let hostMode: 'local' | 'remote' = $state('remote');
	let remoteRoom: api.ArchipelagoGgRoom | null = $state(null);
	let uploading = $state(false);
	let remoteStarting = $state(false);
	let remoteLog: string[] = $state([]);

	let portStr = $state('38281');
	const port = $derived.by(() => {
		const v = parseInt(portStr, 10);
		return isNaN(v) ? 0 : v;
	});
	const portValid = $derived(port >= 1 && port <= 65535);
	let password = $state('');

	let pollHandle: ReturnType<typeof setInterval> | null = null;
	let portPollHandle: ReturnType<typeof setInterval> | null = null;

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
		}, SERVER_POLL_MS);
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

	function stopPortPoll() {
		if (portPollHandle) {
			clearInterval(portPollHandle);
			portPollHandle = null;
		}
	}

	function startPortPoll(roomId: string) {
		stopPortPoll();
		let attempts = 0;
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
				// archipelago.gg may not have provisioned the room yet — keep polling
			}
			if (attempts >= PORT_POLL_MAX_ATTEMPTS) {
				stopPortPoll();
			}
		}, PORT_POLL_MS);
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
		{ open: false, type: 'player', path: '', value: '' }
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
		<RandomizerQuickActions onschemaRefreshing={(active) => (schemasRefreshing = active)} />
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
		<CollapsibleBlock
			title={(i18nState.locale && m.randomizer_pythonRuntime()) ?? ''}
			open={openBlocks.python}
			disabled={hostMode === 'remote'}
			disabledLabel={(i18nState.locale && m.randomizer_localOnly()) ?? ''}
			ontoggle={() => toggleBlock('python')}
		>
			<PythonRuntimeSection {python} />
		</CollapsibleBlock>
	</Tooltip>

	<CollapsibleBlock
		title={(i18nState.locale && m.randomizer_playerSlots()) ?? ''}
		open={openBlocks.players}
		count={players.length}
		ontoggle={() => toggleBlock('players')}
	>
		{#snippet extra()}
			<Tooltip text={i18nState.locale && m.randomizer_openFolder()} position="top" delay={200}>
				<button
					class="rdz-icon-btn"
					onclick={() => api.openWorkspaceDir()}
					aria-label="Open folder"
				>
					<Icon icon="mdi:folder-open" />
				</button>
			</Tooltip>
		{/snippet}

		<PlayerSlotsSection
			{players}
			{initialLoading}
			onRenamePlayer={openRenamePlayer}
			onDeletePlayer={deletePlayer}
		/>
	</CollapsibleBlock>

	<CollapsibleBlock
		title={(i18nState.locale && m.randomizer_seeds()) ?? ''}
		open={openBlocks.seeds}
		count={seeds.length}
		ontoggle={() => toggleBlock('seeds')}
	>
		{#snippet extra()}
			{#if seeds.length > 0}
				<Tooltip
					text={i18nState.locale && m.randomizer_deleteAllSeeds()}
					position="top"
					delay={200}
				>
					<button
						class="rdz-icon-btn rdz-icon-danger rdz-clear-all"
						onclick={() => (confirmClearSeeds = true)}
						aria-label="Clear all seeds"
					>
						<Icon icon="mdi:delete-sweep" />
					</button>
				</Tooltip>
			{/if}
		{/snippet}

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
	</CollapsibleBlock>

	<CollapsibleBlock
		title={(i18nState.locale && m.randomizer_host()) ?? ''}
		open={openBlocks.host}
		ontoggle={() => toggleBlock('host')}
	>
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
	</CollapsibleBlock>
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
		<Button variant="ghost" onclick={() => (renameModal.open = false)}>
			{i18nState.locale && m.randomizer_cancel()}
		</Button>
		<Button variant="primary" onclick={confirmRename}>
			{i18nState.locale && m.randomizer_rename()}
		</Button>
	{/snippet}
</Modal>

<Modal
	bind:open={confirmClearSeeds}
	title={i18nState.locale && m.randomizer_deleteAllTitle()}
	onclose={() => (confirmClearSeeds = false)}
>
	<p>{i18nState.locale && m.randomizer_deleteAllConfirm()}</p>
	{#snippet actions()}
		<Button variant="ghost" onclick={() => (confirmClearSeeds = false)}>
			{i18nState.locale && m.randomizer_cancel()}
		</Button>
		<Button variant="primary" onclick={doConfirmClearSeeds}>
			{i18nState.locale && m.randomizer_deleteAll()}
		</Button>
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

	.rdz-server-header :global(svg) {
		font-size: 20px;
		color: var(--accent-400);
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
