<script lang="ts">
	import Icon from '@iconify/svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
	import type { CommandRegistry, CommandDef, CommandGroup } from '../core/command-registry';

	type Props = {
		open: boolean;
		registry: CommandRegistry;
		prefix: '/' | '!';
		onclose: () => void;
		ontry: (line: string) => void;
	};
	let { open = $bindable(false), registry, prefix, onclose, ontry }: Props = $props();

	let search = $state('');
	let searchEl: HTMLInputElement | undefined = $state();

	const allCommands = $derived(registry.all(prefix));

	const filtered = $derived.by(() => {
		const q = search.trim().toLowerCase();
		if (!q) return allCommands;
		return allCommands.filter(
			(c) =>
				c.name.toLowerCase().includes(q) ||
				c.summary.toLowerCase().includes(q) ||
				(c.description ?? '').toLowerCase().includes(q)
		);
	});

	const GROUPS = $derived<{ id: CommandGroup; label: string; icon: string }[]>([
		{ id: 'info', label: m.console_palette_group_info(), icon: 'mdi:information-outline' },
		{ id: 'action', label: m.console_palette_group_action(), icon: 'mdi:flash' },
		{ id: 'social', label: m.console_palette_group_social(), icon: 'mdi:forum' },
		{ id: 'admin', label: m.console_palette_group_admin(), icon: 'mdi:shield-account' },
		{ id: 'zephyr-ext', label: m.console_palette_group_zephyr(), icon: 'mdi:sparkles' }
	]);

	const grouped = $derived.by(() => {
		const out: Record<CommandGroup, CommandDef[]> = {
			info: [],
			action: [],
			social: [],
			admin: [],
			'zephyr-ext': []
		};
		for (const c of filtered) out[c.group].push(c);
		for (const g of Object.values(out)) g.sort((a, b) => a.name.localeCompare(b.name));
		return out;
	});

	function handleKey(e: KeyboardEvent) {
		if (!open) return;
		if (e.key === 'Escape') {
			e.preventDefault();
			onclose();
		}
	}

	$effect(() => {
		if (open) {
			search = '';
			setTimeout(() => searchEl?.focus(), 20);
		}
	});

	function fillTemplate(c: CommandDef) {
		const placeholders = c.args.map((a) => (a.optional ? `[${a.name}]` : `<${a.name}>`)).join(' ');
		const line = `${c.prefix}${c.name}${placeholders ? ' ' + placeholders : ''}`;
		ontry(line);
		onclose();
	}
</script>

<svelte:window onkeydown={handleKey} />

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div class="zc-hp-backdrop" onclick={onclose}>
		<div
			class="zc-hp"
			role="dialog"
			aria-label={i18nState.locale && m.console_palette_search()}
			onclick={(e) => e.stopPropagation()}
		>
			<div class="zc-hp-search">
				<Icon icon="mdi:magnify" />
				<input
					bind:this={searchEl}
					bind:value={search}
					placeholder={i18nState.locale && m.console_palette_search()}
					spellcheck="false"
					autocomplete="off"
				/>
				<span class="zc-hp-hint">{i18nState.locale && m.console_palette_escToClose()}</span>
			</div>

			<div class="zc-hp-body">
				{#each GROUPS as group (group.id)}
					{@const list = grouped[group.id]}
					{#if list.length > 0}
						<section class="zc-hp-group">
							<header>
								<Icon icon={group.icon} />
								<span>{group.label}</span>
								<small>{list.length}</small>
							</header>
							<div class="zc-hp-rows">
								{#each list as c (c.prefix + c.name)}
									<article class="zc-hp-row" class:coming-soon={c.status === 'coming-soon'}>
										<div class="zc-hp-row-head">
											<code class="zc-hp-sig">{registry.signature(c)}</code>
											{#if c.status === 'coming-soon'}
												<span class="zc-hp-tag">{i18nState.locale && m.console_palette_v2()}</span>
											{/if}
										</div>
										<p class="zc-hp-sum">{c.summary}</p>
										{#if c.description}
											<p class="zc-hp-desc">{c.description}</p>
										{/if}
										{#if c.examples && c.examples.length > 0}
											<div class="zc-hp-examples">
												{#each c.examples as ex}
													<code>{ex}</code>
												{/each}
											</div>
										{/if}
										{#if c.status === 'ready'}
											<button class="zc-hp-try" onclick={() => fillTemplate(c)}>
												<Icon icon="mdi:arrow-right-bold" />
												{i18nState.locale && m.console_palette_tryIt()}
											</button>
										{/if}
									</article>
								{/each}
							</div>
						</section>
					{/if}
				{/each}

				{#if filtered.length === 0}
					<div class="zc-hp-empty">
						<Icon icon="mdi:magnify-close" />
						<p>{i18nState.locale && m.console_palette_empty({ search })}</p>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}

<style>
	.zc-hp-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.55);
		backdrop-filter: blur(4px);
		z-index: var(--z-modal);
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding: 60px 24px;
		animation: fadeIn var(--transition-fast) ease;
	}

	.zc-hp {
		width: min(900px, 100%);
		max-height: calc(100vh - 120px);
		display: flex;
		flex-direction: column;
		background: var(--bg-surface);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
		box-shadow: var(--shadow-lg), var(--shadow-glow);
		overflow: hidden;
	}

	.zc-hp-search {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 14px 18px;
		border-bottom: 1px solid var(--border-subtle);
	}
	.zc-hp-search :global(svg) {
		font-size: 20px;
		color: var(--accent-400);
	}
	.zc-hp-search input {
		flex: 1;
		border: none;
		background: transparent;
		outline: none;
		color: var(--text-primary);
		font-size: 16px;
	}
	.zc-hp-search input::placeholder {
		color: var(--text-muted);
	}
	.zc-hp-hint {
		font-size: 10px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.08em;
	}

	.zc-hp-body {
		flex: 1;
		overflow-y: auto;
		padding: 8px 0 18px;
	}

	.zc-hp-group {
		padding: 8px 18px;
	}

	.zc-hp-group header {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 0;
		color: var(--text-muted);
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		font-weight: 700;
	}
	.zc-hp-group header :global(svg) {
		font-size: 14px;
		color: var(--accent-400);
	}
	.zc-hp-group header small {
		font-size: 10px;
		font-weight: 600;
		background: var(--bg-active);
		padding: 1px 6px;
		border-radius: var(--radius-full);
		color: var(--text-muted);
	}

	.zc-hp-rows {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
		gap: 8px;
		margin-top: 6px;
	}

	.zc-hp-row {
		padding: 10px 12px;
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		background: var(--bg-base);
		display: flex;
		flex-direction: column;
		gap: 4px;
		position: relative;
	}
	.zc-hp-row:hover {
		border-color: var(--accent-400);
	}
	.zc-hp-row.coming-soon {
		opacity: 0.6;
	}
	.zc-hp-row.coming-soon:hover {
		opacity: 0.85;
	}

	.zc-hp-row-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.zc-hp-sig {
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		font-size: 12.5px;
		font-weight: 700;
		color: var(--accent-400);
	}

	.zc-hp-tag {
		font-size: 9.5px;
		font-weight: 700;
		padding: 1px 6px;
		border-radius: var(--radius-full);
		background: color-mix(in srgb, var(--warning) 18%, transparent);
		color: var(--warning);
		text-transform: uppercase;
		letter-spacing: 0.08em;
	}

	.zc-hp-sum {
		margin: 0;
		font-size: 12px;
		color: var(--text-secondary);
	}

	.zc-hp-desc {
		margin: 0;
		font-size: 11px;
		color: var(--text-muted);
		line-height: 1.5;
	}

	.zc-hp-examples {
		display: flex;
		gap: 4px;
		flex-wrap: wrap;
		margin-top: 2px;
	}
	.zc-hp-examples code {
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		background: var(--bg-active);
		padding: 2px 6px;
		border-radius: var(--radius-sm);
		font-size: 10.5px;
		color: var(--text-secondary);
	}

	.zc-hp-try {
		margin-top: 6px;
		align-self: flex-start;
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 4px 10px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-accent);
		background: var(--bg-active);
		color: var(--accent-400);
		font-size: 11px;
		font-weight: 700;
		cursor: pointer;
	}
	.zc-hp-try:hover {
		background: var(--accent-400);
		color: var(--text-inverse);
	}

	.zc-hp-empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 60px 16px;
		color: var(--text-muted);
	}
	.zc-hp-empty :global(svg) {
		font-size: 36px;
		opacity: 0.5;
	}

	@keyframes fadeIn {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}
</style>
