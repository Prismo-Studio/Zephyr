<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Dropdown from '$lib/components/ui/Dropdown.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import { pushToast } from '$lib/toast';
	import * as api from './api';
	import type { GenerateOutcome, PlayerFile, PythonStatus, SeedFile, ServerStatus } from './types';
	import { onDestroy, onMount } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	export function refresh() {
		refreshAll();
	}

	// Collapsible sections — mirror the randomizer options accordion behaviour.
	// All open by default; the narrow sidebar is the default layout so users can
	// close what they don't need to focus on the part they're working on.
	let openBlocks = $state<Record<string, boolean>>({
		python: true,
		players: true,
		seeds: true,
		host: true
	});
	function toggleBlock(key: string) {
		openBlocks[key] = !openBlocks[key];
	}

	// Which connection address to display. The old UI piled 3 cards on top of
	// each other, eating all the vertical space. Dropdown-picker is tighter.
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

	// --- Remote server state ---
	let hostMode: 'local' | 'remote' = $state('remote');
	let remote: api.RemoteStatus | null = $state(null);
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

	// Suggested alt ports when the default 38281 is blocked by the ISP / CGNAT.
	const PRESET_PORTS = [
		{ p: 38281, label: 'AP default' },
		{ p: 443, label: 'HTTPS' },
		{ p: 80, label: 'HTTP' },
		{ p: 25565, label: 'Minecraft' },
		{ p: 7777, label: 'Games' }
	];

	async function refreshAll() {
		[python, players, server, seeds] = await Promise.all([
			api.checkPython(),
			api.listPlayerYamls(),
			api.serverStatus(),
			api.listSeeds()
		]);
		// Auto-select most recent if nothing chosen yet
		if (!selectedSeed && seeds.length > 0) {
			selectedSeed = seeds[0].path;
		}
	}

	onMount(() => {
		refreshAll();
		refreshRemote();
		pollHandle = setInterval(() => {
			api.serverStatus().then((s) => (server = s));
			refreshRemote();
		}, 5000);
	});

	onDestroy(() => {
		if (pollHandle) clearInterval(pollHandle);
	});

	async function generate() {
		generating = true;
		generateLog = '';
		try {
			const outcome: GenerateOutcome = await api.generateSeed();
			generateLog = (outcome.stdout + '\n' + outcome.stderr).trim();
			// Auto-rename the seed with a friendly name based on player names
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

			// Auto-upload and restart remote server if in remote mode
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

	async function uploadAndStartRemote() {
		if (!selectedSeed) return;
		uploading = true;
		remoteLog = [];
		try {
			remoteLog.push(`Uploading ${selectedSeed.split(/[/\\]/).pop()}...`);
			const result = await api.remoteUploadSeed(selectedSeed);
			remoteLog.push(`Uploaded: ${result.uploaded}`);
			remoteStarting = true;
			remoteLog.push('Starting remote server...');
			remote = await api.remoteStart(result.uploaded);
			remoteLog.push(remote.running ? 'Server started!' : 'Server failed to start');
		} catch (e: any) {
			remoteLog.push(`Error: ${e?.message || e}`);
		} finally {
			uploading = false;
			remoteStarting = false;
		}
	}

	async function stopRemote() {
		remote = await api.remoteStop();
	}

	async function refreshRemote() {
		try {
			const status = await api.remoteStatus();
			remote = status;
		} catch {
			// Don't reset remote state on network error — keep last known state
		}
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

	// ── Rename modal state ──
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
		// Player removed → regenerate seed and restart server
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

	function fmtBytes(b: number): string {
		if (b < 1024) return `${b} B`;
		if (b < 1024 * 1024) return `${(b / 1024).toFixed(1)} KB`;
		return `${(b / 1024 / 1024).toFixed(2)} MB`;
	}

	function fmtTime(epoch: number): string {
		if (!epoch) return '';
		const d = new Date(epoch * 1000);
		return d.toLocaleString();
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
	<header class="rdz-server-header">
		<Icon icon="mdi:server-network" />
		<h2>{i18nState.locale && m.randomizer_serverTitle()}</h2>
		<Button size="sm" variant="ghost" onclick={refreshAll}>
			{#snippet icon()}<Icon icon="mdi:refresh" />{/snippet}
			{i18nState.locale && m.randomizer_refresh()}
		</Button>
	</header>

	<!-- Python check (greyed out in remote mode — python runs on the Fly server, not locally) -->
	<Tooltip
		text={hostMode === 'remote' ? i18nState.locale && m.randomizer_pythonRemoteTooltip() : ''}
		position="top"
		delay={300}
		block
	>
		<div class="rdz-block" class:rdz-block-disabled={hostMode === 'remote'}>
			<button class="rdz-block-header" onclick={() => toggleBlock('python')}>
				<Icon icon="mdi:language-python" />
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
				<div class="rdz-block-body">
					{#if python === null}
						<p class="rdz-muted">{i18nState.locale && m.randomizer_checking()}</p>
					{:else if python.available}
						<p class="rdz-ok">
							<Icon icon="mdi:check-circle" />
							{i18nState.locale && m.randomizer_pythonFound({ version: python.version ?? '' })} ({python.executable})
						</p>
						{#if !python.ap_present}
							<p class="rdz-warn">
								<Icon icon="mdi:alert" />
								{i18nState.locale && m.randomizer_apNotFound()} <code>{python.ap_dir}</code>
							</p>
						{:else}
							<p class="rdz-muted"><code>{python.ap_dir}</code></p>
						{/if}
					{:else}
						<p class="rdz-err">
							<Icon icon="mdi:close-circle" />
							{i18nState.locale && m.randomizer_pythonNotDetected()}
						</p>
					{/if}
				</div>
			{/if}
		</div>
	</Tooltip>

	<!-- Player files -->
	<div class="rdz-block">
		<div class="rdz-block-row">
			<button class="rdz-block-header" onclick={() => toggleBlock('players')}>
				<Icon icon="mdi:account-multiple" />
				<span class="rdz-block-name">
					{i18nState.locale && m.randomizer_playerSlots()} <small>({players.length})</small>
				</span>
				<Icon
					icon="mdi:chevron-down"
					class={openBlocks.players ? 'rdz-chev' : 'rdz-chev rdz-chev-closed'}
				/>
			</button>
			<Tooltip
				text={i18nState.locale && m.randomizer_openFolder()}
				position="top"
				delay={200}
			>
				<button class="rdz-icon-btn" onclick={() => api.openWorkspaceDir()} aria-label="Open folder">
					<Icon icon="mdi:folder-open" />
				</button>
			</Tooltip>
		</div>
		{#if openBlocks.players}
			<div class="rdz-block-body">
				{#if players.length === 0}
					<p class="rdz-muted">
						{i18nState.locale && m.randomizer_noPlayerYamls()}
					</p>
				{:else}
					<ul class="rdz-player-list">
						{#each players as p (p.path)}
							<li>
								<Icon icon="mdi:file-document" />
								<span class="rdz-player-name">{p.name}</span>
								<span class="rdz-player-size">{Math.ceil(p.size / 102.4) / 10} KB</span>
								<div class="rdz-seed-actions">
									<Tooltip text={i18nState.locale && m.randomizer_rename()} position="top" delay={200}>
										<button class="rdz-icon-btn" onclick={() => openRenamePlayer(p)}>
											<Icon icon="mdi:pencil" />
										</button>
									</Tooltip>
									<Tooltip text={i18nState.locale && m.randomizer_delete()} position="top" delay={200}>
										<button class="rdz-icon-btn" onclick={() => deletePlayer(p)}>
											<Icon icon="mdi:delete" />
										</button>
									</Tooltip>
								</div>
							</li>
						{/each}
					</ul>
				{/if}
			</div>
		{/if}
	</div>

	<!-- Seeds -->
	<div class="rdz-block">
		<div class="rdz-block-row">
			<button class="rdz-block-header" onclick={() => toggleBlock('seeds')}>
				<Icon icon="mdi:dice-multiple" />
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
					<button class="rdz-icon-btn rdz-clear-all" onclick={clearAllSeeds} aria-label="Clear all seeds">
						<Icon icon="mdi:delete-sweep" />
					</button>
				</Tooltip>
			{/if}
		</div>

		{#if openBlocks.seeds}
		<div class="rdz-block-body">
		<Button
			variant="primary"
			disabled={generating || players.length === 0 || !python?.available}
			onclick={generate}
			loading={generating}
		>
			{#snippet icon()}<Icon icon="mdi:cog-play" />{/snippet}
			{generating
				? i18nState.locale && m.randomizer_generating()
				: i18nState.locale &&
					m.randomizer_generateSeed({
						count: players.length.toString(),
						s: players.length > 1 ? 's' : ''
					})}
		</Button>

		{#if seeds.length > 0}
			<ul class="rdz-seed-list">
				{#each seeds as s (s.path)}
					{@const isHosted = server?.running && server?.multidata === s.path}
					<li
						class="rdz-seed-item"
						class:selected={selectedSeed === s.path}
						class:hosted={isHosted}
					>
						<button class="rdz-seed-pick" onclick={() => (selectedSeed = s.path)}>
							<Icon icon={selectedSeed === s.path ? 'mdi:radiobox-marked' : 'mdi:radiobox-blank'} />
							<div class="rdz-seed-info">
								<span class="rdz-seed-name">
									{s.name}
									{#if isHosted}
										<span class="rdz-hosted-tag">{i18nState.locale && m.randomizer_hosted()}</span>
									{/if}
								</span>
								<small>{fmtTime(s.modified)} - {fmtBytes(s.size)}</small>
							</div>
						</button>
						<div class="rdz-seed-actions">
							<Tooltip text={i18nState.locale && m.randomizer_rename()} position="top" delay={200}>
								<button class="rdz-icon-btn" disabled={isHosted} onclick={() => openRenameSeed(s)}>
									<Icon icon="mdi:pencil" />
								</button>
							</Tooltip>
							<Tooltip text={i18nState.locale && m.randomizer_delete()} position="top" delay={200}>
								<button class="rdz-icon-btn" disabled={isHosted} onclick={() => deleteOneSeed(s)}>
									<Icon icon="mdi:delete" />
								</button>
							</Tooltip>
						</div>
					</li>
				{/each}
			</ul>
		{/if}

		{#if generateLog}
			<details class="rdz-log-details">
				<summary>
					<span>{i18nState.locale && m.randomizer_generateLog()}</span>
					<button
						class="rdz-log-copy"
						title={i18nState.locale && m.randomizer_copy()}
						onclick={(e) => {
							e.preventDefault();
							copyText(generateLog, 'gen');
						}}
					>
						<Icon icon={copiedKey === 'gen' ? 'mdi:check' : 'mdi:content-copy'} />
						{copiedKey === 'gen'
							? i18nState.locale && m.randomizer_copied()
							: i18nState.locale && m.randomizer_copy()}
					</button>
				</summary>
				<pre class="rdz-log">{generateLog}</pre>
			</details>
		{/if}
		</div>
		{/if}
	</div>

	<!-- Host -->
	<div class="rdz-block">
		<button class="rdz-block-header" onclick={() => toggleBlock('host')}>
			<Icon icon="mdi:broadcast" />
			<span class="rdz-block-name">{i18nState.locale && m.randomizer_host()}</span>
			<Icon
				icon="mdi:chevron-down"
				class={openBlocks.host ? 'rdz-chev' : 'rdz-chev rdz-chev-closed'}
			/>
		</button>

		{#if openBlocks.host}
		<div class="rdz-block-body">
		<div class="rdz-host-toggle">
			<button
				class="rdz-toggle-btn"
				class:active={hostMode === 'remote'}
				onclick={() => (hostMode = 'remote')}
			>
				<Icon icon="mdi:cloud" />
				{i18nState.locale && m.randomizer_remote()}
			</button>
			<button
				class="rdz-toggle-btn"
				class:active={hostMode === 'local'}
				onclick={() => (hostMode = 'local')}
			>
				<Icon icon="mdi:laptop" />
				{i18nState.locale && m.randomizer_local()}
			</button>
		</div>

		{#if hostMode === 'remote'}
			<div class="rdz-remote-host">
				{#if remote?.running}
					<div class="rdz-running-line">
						<span class="rdz-live-dot"></span>
						<span>{i18nState.locale && m.randomizer_running()}</span>
						<code>{remote.seed}</code>
					</div>
					<button
						class="rdz-conn-card"
						onclick={() => copyText('nozomi.proxy.rlwy.net:45465', 'addr')}
					>
						<span class="rdz-label"
							><Icon icon="mdi:cloud" /> {i18nState.locale && m.randomizer_connectAddress()}</span
						>
						<code>nozomi.proxy.rlwy.net:45465</code>
						<small
							>{copiedKey === 'addr'
								? i18nState.locale && m.randomizer_copiedExcl()
								: i18nState.locale && m.randomizer_clickToCopy()}</small
						>
					</button>
				{:else}
					<p class="rdz-muted">
						{#if !selectedSeed}
							{i18nState.locale && m.randomizer_selectSeed()}
						{:else}
							{i18nState.locale && m.randomizer_readyToUpload()}
						{/if}
					</p>
					<Button
						variant="primary"
						disabled={!selectedSeed || uploading || remoteStarting}
						loading={uploading || remoteStarting}
						onclick={uploadAndStartRemote}
					>
						{#snippet icon()}<Icon icon="mdi:cloud-upload" />{/snippet}
						{uploading
							? i18nState.locale && m.randomizer_uploading()
							: remoteStarting
								? i18nState.locale && m.randomizer_starting()
								: i18nState.locale && m.randomizer_uploadAndStart()}
					</Button>
				{/if}
				{#if remoteLog.length > 0}
					<details class="rdz-log-details" open>
						<summary>
							<span>{i18nState.locale && m.randomizer_remoteLog()}</span>
							<button
								class="rdz-log-copy"
								onclick={(e) => {
									e.preventDefault();
									copyText(remoteLog.join('\n'), 'remote');
								}}
							>
								<Icon icon={copiedKey === 'remote' ? 'mdi:check' : 'mdi:content-copy'} />
								{copiedKey === 'remote'
									? i18nState.locale && m.randomizer_copied()
									: i18nState.locale && m.randomizer_copy()}
							</button>
						</summary>
						<pre class="rdz-log">{remoteLog.join('\n')}</pre>
					</details>
				{/if}
			</div>
		{:else}
			{#if server?.running}
				{@const selectedIp =
					connTarget === 'local'
						? '127.0.0.1'
						: connTarget === 'lan'
							? server.local_ip ?? '127.0.0.1'
							: server.public_ip ?? '127.0.0.1'}
				{@const selectedAddr = `${selectedIp}:${server.port}`}
				{@const selectedIcon =
					connTarget === 'local' ? 'mdi:laptop' : connTarget === 'lan' ? 'mdi:lan' : 'mdi:earth'}
				{@const selectedLabel =
					connTarget === 'local'
						? (i18nState.locale && m.randomizer_thisMachine()) || 'Cette machine'
						: connTarget === 'lan'
							? (i18nState.locale && m.randomizer_lanTailscale()) || 'LAN / Tailscale'
							: (i18nState.locale && m.randomizer_publicInternet()) || 'Internet public'}
				{@const selectedDesc =
					connTarget === 'local'
						? (i18nState.locale && m.randomizer_thisMachineDesc()) || ''
						: connTarget === 'lan'
							? (i18nState.locale && m.randomizer_lanDesc()) || ''
							: (i18nState.locale &&
									m.randomizer_publicDesc({ port: (server.port ?? 0).toString() })) ||
								''}
				{@const connOptions = [
					{ value: 'local', label: (i18nState.locale && m.randomizer_thisMachine()) || 'Cette machine' },
					...(server.local_ip
						? [{ value: 'lan', label: (i18nState.locale && m.randomizer_lanTailscale()) || 'LAN / Tailscale' }]
						: []),
					...(server.public_ip
						? [{ value: 'public', label: (i18nState.locale && m.randomizer_publicInternet()) || 'Internet public' }]
						: [])
				]}
				<div class="rdz-server-running">
					<div class="rdz-running-line">
						<span class="rdz-live-dot"></span>
						<span>{i18nState.locale &&
							m.randomizer_runningOnPort({ port: (server.port ?? 0).toString() })}</span>
					</div>

					<div class="rdz-conn-picker">
						<Dropdown
							options={connOptions}
							value={connTarget}
							onchange={(v) => (connTarget = v as ConnTarget)}
						/>
						<button
							class="rdz-conn-card rdz-conn-recommended"
							title={i18nState.locale && m.randomizer_clickToCopy()}
							onclick={() => copyText(selectedAddr, 'addr')}
						>
							<span class="rdz-label">
								<Icon icon={selectedIcon} />
								{selectedLabel}
								<span class="rdz-conn-copy-hint">
									<Icon icon={copiedKey === 'addr' ? 'mdi:check' : 'mdi:content-copy'} />
								</span>
							</span>
							<code>{selectedAddr}</code>
							{#if selectedDesc}
								<small>{selectedDesc}</small>
							{/if}
						</button>
					</div>

					{#if server.password}
						<div class="rdz-conn-line">
							<span class="rdz-label">{i18nState.locale && m.randomizer_password()}</span>
							<code>{server.password}</code>
						</div>
					{/if}
					<div class="rdz-conn-line">
						<span class="rdz-label">{i18nState.locale && m.randomizer_pid()}</span>
						<code>{server.pid}</code>
					</div>

					<div class="rdz-inline-field rdz-running-port" aria-disabled="true">
						<span>{i18nState.locale && m.randomizer_port()}</span>
						<div class="rdz-inline-control">
							<Input value={String(server.port ?? portStr)} placeholder="38281" />
						</div>
						<div class="rdz-port-presets">
							{#each PRESET_PORTS as preset}
								<button
									type="button"
									class="rdz-preset-chip"
									class:active={server.port === preset.p}
									disabled
								>
									<span class="rdz-preset-port">{preset.p}</span>
									<span class="rdz-preset-label">{preset.label}</span>
								</button>
							{/each}
						</div>
						<p class="rdz-muted rdz-running-hint">
							<Icon icon="mdi:information-outline" />
							Arrêtez le serveur pour changer le port.
						</p>
					</div>

					<div class="rdz-server-actions">
						<Button variant="primary" onclick={() => api.openConsoleWindow()}>
							{#snippet icon()}<Icon icon="mdi:console" />{/snippet}
							Open console
						</Button>
						<Button variant="danger" onclick={stop}>
							{#snippet icon()}<Icon icon="mdi:stop" />{/snippet}
							{i18nState.locale && m.randomizer_stopServer()}
						</Button>
					</div>
				</div>
			{:else}
				<div class="rdz-host-form">
					<div class="rdz-inline-field">
						<span>{i18nState.locale && m.randomizer_port()}</span>
						<div class="rdz-inline-control">
							<Input bind:value={portStr} placeholder="38281" />
						</div>
						<div class="rdz-port-presets">
							{#each PRESET_PORTS as preset}
								<button
									type="button"
									class="rdz-preset-chip"
									class:active={port === preset.p}
									onclick={() => (portStr = String(preset.p))}
								>
									<span class="rdz-preset-port">{preset.p}</span>
									<span class="rdz-preset-label">{preset.label}</span>
								</button>
							{/each}
						</div>
					</div>
					<div class="rdz-inline-field">
						<span>{i18nState.locale && m.randomizer_passwordOptional()}</span>
						<div class="rdz-inline-control">
							<Input bind:value={password} placeholder={i18nState.locale && m.randomizer_none()} />
						</div>
					</div>
					<Button
						variant="primary"
						disabled={!selectedSeed || starting || !python?.available || !portValid}
						onclick={startHost}
						loading={starting}
					>
						{#snippet icon()}<Icon icon="mdi:play" />{/snippet}
						{i18nState.locale && m.randomizer_startServer()}
					</Button>
				</div>
				{#if !portValid}
					<p class="rdz-warn-text">{i18nState.locale && m.randomizer_portInvalid()}</p>
				{/if}
				{#if !selectedSeed}
					<p class="rdz-muted">{i18nState.locale && m.randomizer_selectSeedFirst()}</p>
				{:else}
					<p class="rdz-muted">
						{i18nState.locale && m.randomizer_willHost()}
						<strong>{selectedSeed.split(/[/\\]/).pop()}</strong>
					</p>
				{/if}

				<div class="rdz-cgnat-note">
					<Icon icon="mdi:information-outline" />
					<div>
						<strong>{i18nState.locale && m.randomizer_cantConnect()}</strong><br />
						{i18nState.locale && m.randomizer_cgnatExplanation()}
					</div>
				</div>
			{/if}

			{#if server?.recent_log && server.recent_log.length > 0}
				<details class="rdz-log-details" open>
					<summary>
						<span
							>{i18nState.locale && m.randomizer_serverLog()} ({server.recent_log.length} lines)</span
						>
						<button
							class="rdz-log-copy"
							title={i18nState.locale && m.randomizer_copy()}
							onclick={(e) => {
								e.preventDefault();
								copyText(server?.recent_log.join('\n') ?? '', 'srv');
							}}
						>
							<Icon icon={copiedKey === 'srv' ? 'mdi:check' : 'mdi:content-copy'} />
							{copiedKey === 'srv'
								? i18nState.locale && m.randomizer_copied()
								: i18nState.locale && m.randomizer_copy()}
						</button>
					</summary>
					<pre class="rdz-log">{server.recent_log.join('\n')}</pre>
				</details>
			{/if}
		{/if}
		<!-- end hostMode if/else -->
		</div>
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
		gap: var(--space-md);
	}

	.rdz-host-toggle {
		display: flex;
		gap: 2px;
		padding: 2px;
		background: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
	}

	.rdz-toggle-btn {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 6px 12px;
		border: none;
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--text-muted);
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.rdz-toggle-btn:hover {
		color: var(--text-secondary);
	}

	.rdz-toggle-btn.active {
		background: var(--bg-elevated);
		color: var(--text-primary);
		box-shadow: var(--shadow-sm);
	}

	.rdz-remote-host {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.rdz-server-header {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding-bottom: var(--space-sm);
		border-bottom: 1px solid var(--border-subtle);
	}

	.rdz-server-header h2 {
		margin: 0;
		flex: 1;
		font-size: 16px;
		color: var(--text-primary);
	}

	.rdz-server-header :global(svg) {
		font-size: 20px;
		color: var(--accent-400);
	}

	.rdz-block {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		padding: var(--space-md);
		background: var(--bg-surface);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-lg);
		transition: opacity var(--transition-fast);
		position: relative;
		overflow: hidden;
	}

	.rdz-block-disabled {
		pointer-events: none;
	}

	.rdz-block-disabled > *:not(.rdz-block-title) {
		opacity: 0.4;
	}

	.rdz-block-disabled .rdz-block-title :global(svg:first-child),
	.rdz-block-disabled .rdz-block-title {
		color: var(--text-muted);
	}

	.rdz-block-disabled::before {
		content: '';
		position: absolute;
		inset: 0;
		background: repeating-linear-gradient(
			135deg,
			transparent,
			transparent 10px,
			var(--bg-active) 10px,
			var(--bg-active) 11px
		);
		pointer-events: none;
		z-index: 1;
		opacity: 0.6;
	}

	.rdz-block-badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		margin-left: auto;
		padding: 2px 8px;
		border-radius: var(--radius-full);
		background: var(--bg-active);
		border: 1px solid var(--border-accent);
		color: var(--text-accent);
		font-size: 10px;
		font-weight: 700;
		text-transform: none;
		letter-spacing: 0;
		position: relative;
		z-index: 2;
		opacity: 2; /* Compensate the parent's 0.5 opacity so badge stays at full opacity */
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
		padding: 8px 10px;
		border: none;
		background: transparent;
		color: var(--text-primary);
		font-family: var(--font-body);
		font-size: 13px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		cursor: pointer;
		border-radius: var(--radius-sm);
		text-align: left;
		transition: background var(--transition-fast);
	}

	.rdz-block-header:hover {
		background: var(--bg-hover);
	}

	.rdz-block-header :global(svg:first-child) {
		font-size: 16px;
		color: var(--accent-400);
		flex-shrink: 0;
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

	.rdz-block-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		padding: 0 4px 4px;
		animation: rdz-block-in 180ms ease;
	}

	@keyframes rdz-block-in {
		from {
			opacity: 0;
			transform: translateY(-2px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.rdz-running-line {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 12px;
		color: var(--text-secondary);
		padding: 2px 2px 6px;
	}

	.rdz-running-line code {
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		color: var(--text-primary);
		background: transparent;
		padding: 0;
		font-size: 12px;
	}

	.rdz-live-dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: var(--success, var(--accent-400));
		box-shadow: 0 0 0 0 color-mix(in srgb, var(--success, var(--accent-400)) 60%, transparent);
		animation: rdz-live-pulse 2s ease-in-out infinite;
		flex-shrink: 0;
	}

	@keyframes rdz-live-pulse {
		0%, 100% {
			box-shadow: 0 0 0 0 color-mix(in srgb, var(--success, var(--accent-400)) 50%, transparent);
		}
		50% {
			box-shadow: 0 0 0 5px transparent;
		}
	}

	.rdz-conn-picker {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.rdz-conn-copy-hint {
		margin-left: auto;
		display: inline-flex;
		align-items: center;
	}

	.rdz-conn-copy-hint :global(svg) {
		font-size: 14px;
		color: var(--text-muted);
		transition: color var(--transition-fast);
	}

	.rdz-conn-card:hover .rdz-conn-copy-hint :global(svg) {
		color: var(--accent-400);
	}

	.rdz-block-title {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		font-size: 12px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-secondary);
	}

	.rdz-block-title :global(svg) {
		font-size: 16px;
		color: var(--accent-400);
	}

	.rdz-block-title :global(.z-btn) {
		margin-left: auto;
	}

	.rdz-muted {
		margin: 0;
		color: var(--text-muted);
		font-size: 12px;
	}

	.rdz-ok {
		display: flex;
		align-items: center;
		gap: 6px;
		margin: 0;
		color: #66d9b0;
		font-size: 12px;
	}

	.rdz-warn {
		display: flex;
		align-items: center;
		gap: 6px;
		margin: 0;
		color: #ffcc80;
		font-size: 12px;
	}

	.rdz-err {
		display: flex;
		align-items: center;
		gap: 6px;
		margin: 0;
		color: #ef9a9a;
		font-size: 12px;
	}

	.rdz-player-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.rdz-player-list li {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: 6px 10px;
		border-radius: var(--radius-sm);
		background: var(--bg-base);
		border: 1px solid var(--border-subtle);
	}

	.rdz-player-list :global(svg) {
		font-size: 16px;
		color: var(--text-muted);
	}

	.rdz-player-name {
		flex: 1;
		font-size: 13px;
		color: var(--text-primary);
		font-weight: 600;
	}

	.rdz-player-size {
		font-size: 11px;
		color: var(--text-muted);
		font-family: var(--font-mono, monospace);
	}

	.rdz-seed-actions {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		margin-left: auto;
		flex-shrink: 0;
	}

	.rdz-icon-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 26px;
		height: 26px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.rdz-icon-btn:hover {
		color: var(--error);
		border-color: var(--error);
	}

	.rdz-icon-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.rdz-icon-btn:disabled:hover {
		color: var(--text-muted);
		border-color: var(--border-default);
	}

	.rdz-clear-all {
		margin-left: 0;
	}

	.rdz-block-title > :global(.z-tooltip-trigger:last-child) {
		margin-left: auto;
	}

	.rdz-seed-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.rdz-seed-item {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		padding: 4px;
		border-radius: var(--radius-sm);
		background: var(--bg-base);
		border: 1px solid var(--border-subtle);
		transition: all var(--transition-fast);
	}

	.rdz-seed-item.selected {
		border-color: var(--accent-400);
		background: var(--bg-active);
	}

	.rdz-seed-item.hosted {
		border-color: #66d9b0;
		background: rgba(102, 217, 176, 0.06);
	}

	.rdz-seed-pick {
		flex: 1;
		min-width: 0;
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: 6px 8px;
		border: none;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		text-align: left;
	}

	.rdz-seed-pick :global(svg) {
		font-size: 18px;
		color: var(--accent-400);
		flex-shrink: 0;
	}

	.rdz-seed-info {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.rdz-seed-name {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
		font-family: var(--font-mono, monospace);
		color: var(--text-primary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.rdz-seed-info small {
		font-size: 10px;
		color: var(--text-muted);
	}

	.rdz-hosted-tag {
		font-size: 9px;
		font-weight: 700;
		letter-spacing: 0.05em;
		padding: 1px 6px;
		border-radius: var(--radius-full);
		background: rgba(102, 217, 176, 0.15);
		color: #66d9b0;
		font-family: var(--font-body);
	}

	.rdz-host-form {
		display: flex;
		gap: var(--space-md);
		align-items: flex-end;
		flex-wrap: wrap;
	}

	.rdz-inline-field {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.rdz-inline-field span {
		font-size: 9px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-muted);
	}

	.rdz-inline-control {
		width: 160px;
	}

	.rdz-port-presets {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		margin-top: 4px;
	}

	.rdz-preset-chip {
		display: inline-flex;
		align-items: baseline;
		gap: 6px;
		padding: 4px 10px;
		border-radius: var(--radius-full);
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.rdz-preset-port {
		font-family: var(--font-mono, 'JetBrains Mono', monospace);
		font-size: 11px;
		font-weight: 700;
		color: var(--text-primary);
	}

	.rdz-preset-label {
		font-size: 10px;
		color: var(--text-muted);
	}

	.rdz-preset-chip:hover {
		border-color: var(--accent-400);
	}
	.rdz-preset-chip:hover .rdz-preset-port,
	.rdz-preset-chip:hover .rdz-preset-label {
		color: var(--accent-400);
	}

	.rdz-preset-chip.active {
		background: var(--bg-active);
		border-color: var(--accent-400);
	}
	.rdz-preset-chip.active .rdz-preset-port,
	.rdz-preset-chip.active .rdz-preset-label {
		color: var(--accent-400);
	}

	.rdz-preset-chip:disabled {
		cursor: not-allowed;
	}

	.rdz-running-port {
		opacity: 0.65;
		pointer-events: none;
	}
	.rdz-running-port .rdz-preset-chip.active {
		pointer-events: none;
	}

	.rdz-running-hint {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		margin-top: 6px !important;
		opacity: 0.85;
	}
	.rdz-running-hint :global(svg) {
		font-size: 13px;
		color: var(--accent-400);
	}

	.rdz-warn-text {
		margin: 0;
		color: #ef9a9a;
		font-size: 11px;
	}

	.rdz-cgnat-note {
		display: flex;
		align-items: flex-start;
		gap: var(--space-sm);
		margin-top: var(--space-sm);
		padding: var(--space-sm) var(--space-md);
		background: var(--bg-elevated);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		color: var(--text-secondary);
		font-size: 11px;
		line-height: 1.5;
	}

	.rdz-cgnat-note :global(svg) {
		font-size: 16px;
		color: var(--accent-400);
		flex-shrink: 0;
		margin-top: 2px;
	}

	.rdz-cgnat-note strong {
		color: var(--text-primary);
	}

	.rdz-cgnat-note em {
		color: var(--accent-400);
		font-style: normal;
		font-weight: 600;
	}

	.rdz-server-running {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.rdz-server-actions {
		display: flex;
		gap: var(--space-sm);
		flex-wrap: wrap;
		align-items: center;
	}

	.rdz-status-pill {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 4px 10px;
		border-radius: var(--radius-full);
		font-size: 11px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		width: fit-content;
		border: 1px solid;
	}

	.rdz-status-row {
		display: flex;
		gap: var(--space-xs);
		flex-wrap: wrap;
	}

	.rdz-status-on {
		background: rgba(102, 217, 176, 0.1);
		color: #66d9b0;
		border-color: rgba(102, 217, 176, 0.4);
	}

	.rdz-status-on :global(svg) {
		color: #66d9b0;
		animation: pulse 1.6s ease infinite;
	}

	.rdz-status-warn {
		background: rgba(255, 204, 128, 0.1);
		color: #ffcc80;
		border-color: rgba(255, 204, 128, 0.4);
	}

	.rdz-status-warn :global(svg) {
		color: #ffcc80;
	}

	@keyframes pulse {
		50% {
			opacity: 0.4;
		}
	}

	.rdz-conn-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-sm);
	}

	.rdz-conn-card {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: var(--space-sm) var(--space-md);
		background: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		text-align: left;
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.rdz-conn-card:hover {
		border-color: var(--accent-400);
		background: var(--bg-elevated);
	}

	.rdz-conn-recommended {
		border-color: var(--accent-400);
		background: var(--bg-active);
	}

	.rdz-conn-card small {
		font-size: 10px;
		color: var(--text-muted);
		line-height: 1.3;
	}

	.rdz-conn-line {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: 4px 8px;
	}

	.rdz-label {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 9px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-muted);
	}

	.rdz-label :global(svg) {
		font-size: 11px;
	}

	code {
		font-family: var(--font-mono, monospace);
		font-size: 12px;
		color: var(--text-accent);
		background: var(--bg-active);
		padding: 2px 6px;
		border-radius: var(--radius-sm);
	}

	.rdz-log {
		margin: 0;
		padding: var(--space-sm);
		background: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-sm);
		font-family: var(--font-mono, monospace);
		font-size: 11px;
		color: var(--text-secondary);
		max-height: 240px;
		overflow: auto;
		white-space: pre-wrap;
		word-break: break-all;
		user-select: text !important;
		-webkit-user-select: text !important;
		cursor: text !important;
	}

	.rdz-log::selection {
		background: var(--accent-400);
		color: var(--text-primary);
	}

	.rdz-log-details summary {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		cursor: pointer;
		font-size: 11px;
		color: var(--text-muted);
		padding: 4px 0;
		list-style: none;
	}

	.rdz-log-details summary::-webkit-details-marker {
		display: none;
	}

	.rdz-log-details summary::before {
		content: '▶';
		display: inline-block;
		font-size: 9px;
		transition: transform var(--transition-fast);
		color: var(--text-muted);
	}

	.rdz-log-details[open] summary::before {
		transform: rotate(90deg);
	}

	.rdz-log-details summary span {
		flex: 1;
	}

	.rdz-log-copy {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 3px 8px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		color: var(--text-secondary);
		font-size: 10px;
		font-weight: 600;
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.rdz-log-copy :global(svg) {
		font-size: 12px;
	}

	.rdz-log-copy:hover {
		border-color: var(--accent-400);
		color: var(--accent-400);
	}
</style>
