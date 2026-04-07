<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import * as api from './api';
	import type {
		GenerateOutcome,
		PlayerFile,
		PythonStatus,
		SeedFile,
		ServerStatus
	} from './types';
	import { onDestroy, onMount } from 'svelte';

	let python: PythonStatus | null = $state(null);
	let players: PlayerFile[] = $state([]);
	let server: ServerStatus | null = $state(null);
	let seeds: SeedFile[] = $state([]);
	let selectedSeed: string | null = $state(null);
	let generateLog: string = $state('');
	let generating = $state(false);
	let starting = $state(false);

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
		{ p: 443, label: '443 (HTTPS)' },
		{ p: 80, label: '80 (HTTP)' },
		{ p: 25565, label: '25565 (Minecraft)' },
		{ p: 7777, label: '7777 (games)' }
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
		pollHandle = setInterval(() => api.serverStatus().then((s) => (server = s)), 2500);
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
			seeds = await api.listSeeds();
			if (outcome.zip_path) {
				selectedSeed = outcome.zip_path;
			}
		} finally {
			generating = false;
		}
	}

	async function startHost() {
		if (!selectedSeed || !portValid) return;
		starting = true;
		try {
			server = await api.startServer(selectedSeed, port, password.trim() || null);
		} finally {
			starting = false;
		}
	}

	async function stop() {
		await api.stopServer();
		server = await api.serverStatus();
	}

	async function deletePlayer(p: PlayerFile) {
		await api.deletePlayerYaml(p.path);
		players = await api.listPlayerYamls();
	}

	async function deleteOneSeed(s: SeedFile) {
		await api.deleteSeed(s.path);
		if (selectedSeed === s.path) selectedSeed = null;
		seeds = await api.listSeeds();
		if (!selectedSeed && seeds.length > 0) selectedSeed = seeds[0].path;
	}

	async function clearAllSeeds() {
		if (!confirm('Delete all generated seeds?')) return;
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
		<h2>Multiplayer Server</h2>
		<Button size="sm" variant="ghost" onclick={refreshAll}>
			{#snippet icon()}<Icon icon="mdi:refresh" />{/snippet}
			Refresh
		</Button>
	</header>

	<!-- Python check -->
	<div class="rdz-block">
		<div class="rdz-block-title">
			<Icon icon="mdi:language-python" />
			Python runtime
		</div>
		{#if python === null}
			<p class="rdz-muted">Checking…</p>
		{:else if python.available}
			<p class="rdz-ok">
				<Icon icon="mdi:check-circle" />
				Python {python.version} found ({python.executable})
			</p>
			{#if !python.ap_present}
				<p class="rdz-warn">
					<Icon icon="mdi:alert" />
					Archipelago not found at <code>{python.ap_dir}</code>
				</p>
			{:else}
				<p class="rdz-muted"><code>{python.ap_dir}</code></p>
			{/if}
		{:else}
			<p class="rdz-err">
				<Icon icon="mdi:close-circle" />
				Python not detected. Install Python 3.11+ to generate seeds and host servers.
			</p>
		{/if}
	</div>

	<!-- Player files -->
	<div class="rdz-block">
		<div class="rdz-block-title">
			<Icon icon="mdi:account-multiple" />
			Player slots ({players.length})
			<Button size="sm" variant="ghost" onclick={() => api.openWorkspaceDir()}>
				{#snippet icon()}<Icon icon="mdi:folder-open" />{/snippet}
				Open folder
			</Button>
		</div>
		{#if players.length === 0}
			<p class="rdz-muted">
				No player YAMLs yet. Configure a game above and click <strong>Save player slot</strong>.
			</p>
		{:else}
			<ul class="rdz-player-list">
				{#each players as p (p.path)}
					<li>
						<Icon icon="mdi:file-document" />
						<span class="rdz-player-name">{p.name}</span>
						<span class="rdz-player-size">{Math.ceil(p.size / 102.4) / 10} KB</span>
						<button
							class="rdz-icon-btn"
							aria-label="Delete"
							onclick={() => deletePlayer(p)}
						>
							<Icon icon="mdi:delete" />
						</button>
					</li>
				{/each}
			</ul>
		{/if}
	</div>

	<!-- Seeds -->
	<div class="rdz-block">
		<div class="rdz-block-title">
			<Icon icon="mdi:dice-multiple" />
			Seeds ({seeds.length})
			{#if seeds.length > 0}
				<button class="rdz-icon-btn rdz-clear-all" onclick={clearAllSeeds} title="Delete all seeds">
					<Icon icon="mdi:delete-sweep" />
				</button>
			{/if}
		</div>

		<Button
			variant="primary"
			disabled={generating || players.length === 0 || !python?.available}
			onclick={generate}
			loading={generating}
		>
			{#snippet icon()}<Icon icon="mdi:cog-play" />{/snippet}
			{generating
				? 'Generating...'
				: `Generate new seed (${players.length} slot${players.length > 1 ? 's' : ''})`}
		</Button>

		{#if seeds.length > 0}
			<ul class="rdz-seed-list">
				{#each seeds as s (s.path)}
					{@const isHosted = server?.running && server?.multidata === s.path}
					<li class="rdz-seed-item" class:selected={selectedSeed === s.path} class:hosted={isHosted}>
						<button class="rdz-seed-pick" onclick={() => (selectedSeed = s.path)}>
							<Icon
								icon={selectedSeed === s.path ? 'mdi:radiobox-marked' : 'mdi:radiobox-blank'}
							/>
							<div class="rdz-seed-info">
								<span class="rdz-seed-name">
									{s.name}
									{#if isHosted}
										<span class="rdz-hosted-tag">HOSTED</span>
									{/if}
								</span>
								<small>{fmtTime(s.modified)} - {fmtBytes(s.size)}</small>
							</div>
						</button>
						<button
							class="rdz-icon-btn"
							aria-label="Delete seed"
							disabled={isHosted}
							onclick={() => deleteOneSeed(s)}
						>
							<Icon icon="mdi:delete" />
						</button>
					</li>
				{/each}
			</ul>
		{/if}

		{#if generateLog}
			<details class="rdz-log-details">
				<summary>
					<span>Generate log</span>
					<button
						class="rdz-log-copy"
						title="Copy"
						onclick={(e) => {
							e.preventDefault();
							copyText(generateLog, 'gen');
						}}
					>
						<Icon icon={copiedKey === 'gen' ? 'mdi:check' : 'mdi:content-copy'} />
						{copiedKey === 'gen' ? 'Copied' : 'Copy'}
					</button>
				</summary>
				<pre class="rdz-log">{generateLog}</pre>
			</details>
		{/if}
	</div>

	<!-- Host -->
	<div class="rdz-block">
		<div class="rdz-block-title">
			<Icon icon="mdi:broadcast" />
			Host
		</div>

		{#if server?.running}
			<div class="rdz-server-running">
				<div class="rdz-status-row">
					<div class="rdz-status-pill rdz-status-on">
						<Icon icon="mdi:circle" /> Running on port {server.port}
					</div>
					{#if server.port_reachable}
						<div class="rdz-status-pill rdz-status-on">
							<Icon icon="mdi:check-circle" /> Port reachable
						</div>
					{:else}
						<div class="rdz-status-pill rdz-status-warn">
							<Icon icon="mdi:alert" /> Port not responding
						</div>
					{/if}
				</div>

				<div class="rdz-conn-grid">
					<button
						class="rdz-conn-card rdz-conn-recommended"
						title="Click to copy"
						onclick={() => navigator.clipboard.writeText(`127.0.0.1:${server?.port}`)}
					>
						<span class="rdz-label">
							<Icon icon="mdi:laptop" /> This machine (recommended for testing)
						</span>
						<code>127.0.0.1:{server.port}</code>
						<small>Use this URL in your client when testing on the same PC. Always works.</small>
					</button>
					{#if server.local_ip}
						<button
							class="rdz-conn-card"
							title="Click to copy"
							onclick={() => navigator.clipboard.writeText(`${server?.local_ip}:${server?.port}`)}
						>
							<span class="rdz-label">
								<Icon icon="mdi:lan" /> LAN / Tailscale
							</span>
							<code>{server.local_ip}:{server.port}</code>
							<small>Same WiFi router, or via Tailscale / Hamachi / ZeroTier (recommended for friends).</small>
						</button>
					{/if}
					{#if server.public_ip}
						<button
							class="rdz-conn-card"
							title="Click to copy"
							onclick={() => navigator.clipboard.writeText(`${server?.public_ip}:${server?.port}`)}
						>
							<span class="rdz-label">
								<Icon icon="mdi:earth" /> Public Internet
							</span>
							<code>{server.public_ip}:{server.port}</code>
							<small>
								Requires port {server.port} forwarded on your router. <strong>Won't work from this machine</strong>
								(NAT loopback) — use Localhost above for self-testing.
							</small>
						</button>
					{/if}
				</div>

				{#if server.password}
					<div class="rdz-conn-line">
						<span class="rdz-label">Password</span>
						<code>{server.password}</code>
					</div>
				{/if}
				<div class="rdz-conn-line">
					<span class="rdz-label">PID</span>
					<code>{server.pid}</code>
				</div>

				<Button variant="danger" onclick={stop}>
					{#snippet icon()}<Icon icon="mdi:stop" />{/snippet}
					Stop server
				</Button>
			</div>
		{:else}
			<div class="rdz-host-form">
				<div class="rdz-inline-field">
					<span>Port</span>
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
								title={preset.label}
							>
								{preset.p}
							</button>
						{/each}
					</div>
				</div>
				<div class="rdz-inline-field">
					<span>Password (optional)</span>
					<div class="rdz-inline-control">
						<Input bind:value={password} placeholder="(none)" />
					</div>
				</div>
				<Button
					variant="primary"
					disabled={!selectedSeed || starting || !python?.available || !portValid}
					onclick={startHost}
					loading={starting}
				>
					{#snippet icon()}<Icon icon="mdi:play" />{/snippet}
					Start server
				</Button>
			</div>
			{#if !portValid}
				<p class="rdz-warn-text">Port must be between 1 and 65535.</p>
			{/if}
			{#if !selectedSeed}
				<p class="rdz-muted">Generate or select a seed first.</p>
			{:else}
				<p class="rdz-muted">
					Will host: <strong>{selectedSeed.split(/[/\\]/).pop()}</strong>
				</p>
			{/if}

			<div class="rdz-cgnat-note">
				<Icon icon="mdi:information-outline" />
				<div>
					<strong>Friend can't connect from outside?</strong><br />
					Many ISPs use <em>CGNAT</em> (shared IP) or block incoming ports — port-forwarding
					won't work. Easiest fix: install <strong>Tailscale</strong> on both machines (free, 2
					min setup), then share your Tailscale IP from the "LAN / Tailscale" card above. No
					firewall, no port forwarding needed.
				</div>
			</div>
		{/if}

		{#if server?.recent_log && server.recent_log.length > 0}
			<details class="rdz-log-details" open>
				<summary>
					<span>Server log ({server.recent_log.length} lines)</span>
					<button
						class="rdz-log-copy"
						title="Copy"
						onclick={(e) => {
							e.preventDefault();
							copyText(server?.recent_log.join('\n') ?? '', 'srv');
						}}
					>
						<Icon icon={copiedKey === 'srv' ? 'mdi:check' : 'mdi:content-copy'} />
						{copiedKey === 'srv' ? 'Copied' : 'Copy'}
					</button>
				</summary>
				<pre class="rdz-log">{server.recent_log.join('\n')}</pre>
			</details>
		{/if}
	</div>
</section>

<style>
	.rdz-server {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
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
		background: rgba(26, 255, 250, 0.04);
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
		padding: 2px 8px;
		border-radius: var(--radius-full);
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		color: var(--text-muted);
		font-size: 10px;
		font-family: var(--font-mono, monospace);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.rdz-preset-chip:hover {
		border-color: var(--accent-400);
		color: var(--accent-400);
	}

	.rdz-preset-chip.active {
		background: rgba(26, 255, 250, 0.12);
		border-color: var(--accent-400);
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
		background: rgba(26, 255, 250, 0.04);
		border-left: 3px solid var(--accent-400);
		border-radius: var(--radius-sm);
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
		border-color: rgba(26, 255, 250, 0.4);
		background: rgba(26, 255, 250, 0.04);
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
		background: rgba(26, 255, 250, 0.06);
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
		background: rgba(26, 255, 250, 0.3);
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
