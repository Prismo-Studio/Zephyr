<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import type { PlayerFile, PythonStatus, SeedFile, ServerStatus } from './types';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	let {
		seeds,
		players,
		python,
		server,
		selectedSeed = $bindable(),
		generating,
		generateLog,
		initialLoading,
		copiedKey,
		onGenerate,
		onRenameSeed,
		onDeleteSeed,
		onCopyText
	}: {
		seeds: SeedFile[];
		players: PlayerFile[];
		python: PythonStatus | null;
		server: ServerStatus | null;
		selectedSeed: string | null;
		generating: boolean;
		generateLog: string;
		initialLoading: boolean;
		copiedKey: string | null;
		onGenerate: () => void;
		onRenameSeed: (s: SeedFile) => void;
		onDeleteSeed: (s: SeedFile) => void;
		onCopyText: (text: string, key: string) => void;
	} = $props();

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
</script>

<div class="rdz-block-body">
	{#if initialLoading}
		<p class="rdz-muted rdz-loading-row">
			<Icon icon="mdi:loading" class="rdz-spin" />
			<span>{i18nState.locale && m.randomizer_loadingData()}</span>
		</p>
	{/if}
	<Button
		variant="primary"
		disabled={generating || players.length === 0 || !python?.available}
		onclick={onGenerate}
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
				<li class="rdz-seed-item" class:selected={selectedSeed === s.path} class:hosted={isHosted}>
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
							<button class="rdz-icon-btn" disabled={isHosted} onclick={() => onRenameSeed(s)}>
								<Icon icon="mdi:pencil" />
							</button>
						</Tooltip>
						<Tooltip text={i18nState.locale && m.randomizer_delete()} position="top" delay={200}>
							<button
								class="rdz-icon-btn rdz-icon-danger"
								disabled={isHosted}
								onclick={() => onDeleteSeed(s)}
							>
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
						onCopyText(generateLog, 'gen');
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

<style>
	.rdz-block-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		padding: 0;
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

	.rdz-muted {
		margin: 0;
		color: var(--text-muted);
		font-size: 12px;
	}

	.rdz-loading-row {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}

	.rdz-seed-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 0;
	}

	.rdz-seed-item {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		padding: 6px 8px;
		border-radius: 0;
		border-bottom: 1px solid var(--border-subtle);
		transition: background var(--transition-fast);
	}

	.rdz-seed-item:hover {
		background: var(--bg-hover);
	}

	.rdz-seed-item:last-child {
		border-bottom: none;
	}

	.rdz-seed-item.selected {
		background: var(--bg-hover);
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

	.rdz-icon-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.rdz-icon-btn:disabled:hover {
		color: var(--text-muted);
		background: transparent;
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
