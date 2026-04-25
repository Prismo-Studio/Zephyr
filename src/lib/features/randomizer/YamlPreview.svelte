<script lang="ts">
	import Icon from '@iconify/svelte';
	import { randomizerStore } from './randomizer.store.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	let codeEl: HTMLPreElement | null = $state(null);
	let copied = $state(false);

	function copy() {
		navigator.clipboard
			.writeText(randomizerStore.generatedYaml)
			.then(() => {
				copied = true;
				setTimeout(() => (copied = false), 1500);
			})
			.catch(() => {});
	}

	const lintErrors = $derived(randomizerStore.lintIssues.filter((i) => i.level === 'error'));
	const lintWarnings = $derived(randomizerStore.lintIssues.filter((i) => i.level === 'warning'));
	const lintStatus = $derived.by(() => {
		if (!randomizerStore.generatedYaml) return 'idle';
		if (lintErrors.length > 0) return 'error';
		if (lintWarnings.length > 0) return 'warning';
		return 'ok';
	});

	// naive YAML highlight: keys, strings, numbers, booleans
	function highlight(line: string): string {
		const escape = (s: string) =>
			s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
		const m = line.match(/^(\s*)([\w\-]+)(:)(.*)$/);
		if (m) {
			const [, indent, key, colon, rest] = m;
			let valHtml = escape(rest);
			valHtml = valHtml.replace(
				/(\b(?:true|false|null)\b|-?\d+(?:\.\d+)?|".*?"|'.*?')/g,
				'<span class="yh-val">$1</span>'
			);
			return `${indent}<span class="yh-key">${escape(key)}</span><span class="yh-punct">${colon}</span>${valHtml}`;
		}
		return escape(line);
	}

	const lines = $derived(randomizerStore.generatedYaml.split('\n'));

	/** Find the line index in the YAML for the given option id (matches `  <id>:`). */
	const highlightedLineIndex = $derived.by(() => {
		const id = randomizerStore.lastChangedId;
		if (!id || lines.length === 0) return -1;
		const re = new RegExp(`^\\s*${id.replace(/[.*+?^${}()|[\\]\\\\]/g, '\\$&')}:`);
		return lines.findIndex((l) => re.test(l));
	});

	// Auto-scroll to the highlighted line if it's outside the visible area.
	$effect(() => {
		const idx = highlightedLineIndex;
		if (idx < 0 || !codeEl) return;
		// queueMicrotask so the DOM has the new <span> rendered before we query
		queueMicrotask(() => {
			if (!codeEl) return;
			const target = codeEl.querySelector<HTMLElement>(`[data-line-idx="${idx}"]`);
			if (!target) return;
			const cRect = codeEl.getBoundingClientRect();
			const tRect = target.getBoundingClientRect();
			const above = tRect.top < cRect.top + 8;
			const below = tRect.bottom > cRect.bottom - 8;
			if (above || below) {
				const offset =
					tRect.top - cRect.top + codeEl.scrollTop - cRect.height / 2 + tRect.height / 2;
				codeEl.scrollTo({ top: offset, behavior: 'smooth' });
			}
		});
	});
</script>

<aside class="rdz-yaml">
	<header class="rdz-yaml-header">
		<div class="rdz-yaml-title">
			<Icon icon="mdi:code-braces" />
			<span>{i18nState.locale && m.randomizer_generatedYaml()}</span>
		</div>
		<div class="rdz-yaml-actions">
			<span class="rdz-lint-pill rdz-lint-{lintStatus}">
				{#if lintStatus === 'ok'}
					<Icon icon="mdi:check-circle" /> {i18nState.locale && m.randomizer_lintOk()}
				{:else if lintStatus === 'warning'}
					<Icon icon="mdi:alert" />
					{lintWarnings.length} warning{lintWarnings.length > 1 ? 's' : ''}
				{:else if lintStatus === 'error'}
					<Icon icon="mdi:close-circle" />
					{lintErrors.length} error{lintErrors.length > 1 ? 's' : ''}
				{:else}
					<Icon icon="mdi:dots-horizontal" /> {i18nState.locale && m.randomizer_idle()}
				{/if}
			</span>
			<button
				class="rdz-yaml-copy"
				onclick={copy}
				disabled={!randomizerStore.generatedYaml}
				aria-label={i18nState.locale && m.randomizer_copyYaml()}
			>
				<Icon icon={copied ? 'mdi:check' : 'mdi:content-copy'} />
			</button>
		</div>
	</header>

	{#if randomizerStore.validationErrors.length > 0}
		<div class="rdz-yaml-errors">
			<div class="rdz-yaml-errors-title">
				<Icon icon="mdi:alert-circle" />
				{randomizerStore.validationErrors.length} validation issue{randomizerStore.validationErrors
					.length > 1
					? 's'
					: ''}
			</div>
			<ul>
				{#each randomizerStore.validationErrors as err}
					<li><strong>{err.option_id}</strong>: {err.message}</li>
				{/each}
			</ul>
		</div>
	{/if}

	{#if randomizerStore.lintIssues.length > 0}
		<div class="rdz-yaml-lint">
			<div class="rdz-yaml-lint-title">
				<Icon icon="mdi:format-list-checks" />
				{i18nState.locale && m.randomizer_yamlLint()}
			</div>
			<ul>
				{#each randomizerStore.lintIssues as issue}
					<li class="rdz-lint-item rdz-lint-item-{issue.level}">
						<Icon
							icon={issue.level === 'error'
								? 'mdi:close-circle'
								: issue.level === 'warning'
									? 'mdi:alert'
									: 'mdi:information'}
						/>
						<span>{issue.message}</span>
					</li>
				{/each}
			</ul>
		</div>
	{/if}

	<pre class="rdz-yaml-code" bind:this={codeEl}><code
			>{#each lines as line, i}<span
					class="yh-line"
					class:highlighted={i === highlightedLineIndex}
					data-line-idx={i}>{@html highlight(line)}</span
				>{#if i < lines.length - 1}{'\n'}{/if}{/each}</code
		></pre>
</aside>

<style>
	.rdz-yaml {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-width: 0;
		min-height: 0;
		background: var(--bg-surface);
		overflow: hidden;
	}

	.rdz-yaml-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-md);
		border-bottom: 1px solid var(--border-subtle);
		background: var(--bg-elevated);
	}

	.rdz-yaml-title {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		color: var(--text-accent);
		font-size: 12px;
		font-weight: 700;
		letter-spacing: 0.05em;
		text-transform: uppercase;
	}

	.rdz-yaml-actions {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
	}

	.rdz-lint-pill {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 3px 8px;
		border-radius: var(--radius-full);
		font-size: 10px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		border: 1px solid;
	}

	.rdz-lint-pill :global(svg) {
		font-size: 12px;
	}

	.rdz-lint-ok {
		color: #66d9b0;
		border-color: rgba(102, 217, 176, 0.4);
		background: rgba(102, 217, 176, 0.08);
	}

	.rdz-lint-warning {
		color: #ffcc80;
		border-color: rgba(255, 204, 128, 0.4);
		background: rgba(255, 204, 128, 0.08);
	}

	.rdz-lint-error {
		color: #ef9a9a;
		border-color: rgba(239, 154, 154, 0.4);
		background: rgba(239, 154, 154, 0.08);
	}

	.rdz-lint-idle {
		color: var(--text-muted);
		border-color: var(--border-default);
		background: var(--bg-base);
	}

	.rdz-yaml-copy {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border-radius: var(--radius-md);
		border: 1px solid var(--border-default);
		background: var(--bg-base);
		color: var(--text-secondary);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.rdz-yaml-copy:hover:not(:disabled) {
		border-color: var(--accent-400);
		color: var(--accent-400);
	}

	.rdz-yaml-copy:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.rdz-yaml-errors {
		padding: var(--space-md);
		background: rgba(244, 67, 54, 0.08);
		border-bottom: 1px solid rgba(244, 67, 54, 0.2);
		color: #ef9a9a;
		font-size: 12px;
	}

	.rdz-yaml-errors-title {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		font-weight: 700;
		margin-bottom: var(--space-xs);
	}

	.rdz-yaml-errors ul {
		margin: 0;
		padding-left: var(--space-lg);
	}

	.rdz-yaml-lint {
		padding: var(--space-md);
		background: var(--bg-elevated);
		border-bottom: 1px solid var(--border-subtle);
		font-size: 11px;
	}

	.rdz-yaml-lint-title {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		font-size: 10px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-muted);
		margin-bottom: var(--space-xs);
	}

	.rdz-yaml-lint ul {
		margin: 0;
		padding: 0;
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.rdz-lint-item {
		display: flex;
		align-items: flex-start;
		gap: 6px;
		padding: 4px 6px;
		border-radius: var(--radius-sm);
		line-height: 1.4;
	}

	.rdz-lint-item :global(svg) {
		font-size: 13px;
		flex-shrink: 0;
		margin-top: 1px;
	}

	.rdz-lint-item-error {
		color: #ef9a9a;
		background: rgba(239, 154, 154, 0.06);
	}

	.rdz-lint-item-warning {
		color: #ffcc80;
		background: rgba(255, 204, 128, 0.06);
	}

	.rdz-lint-item-info {
		color: var(--text-secondary);
		background: var(--bg-base);
	}

	.rdz-yaml-code {
		flex: 1;
		overflow: auto;
		margin: 0;
		padding: var(--space-md);
		background: var(--bg-base);
		font-family: var(--font-mono, 'JetBrains Mono', monospace);
		font-size: 12px;
		line-height: 1.6;
		color: var(--text-secondary);
		white-space: pre;
		user-select: text !important;
		-webkit-user-select: text !important;
		cursor: text !important;
	}

	.rdz-yaml-code :global(*) {
		user-select: text !important;
		-webkit-user-select: text !important;
		cursor: text !important;
	}

	.rdz-yaml-code::selection {
		background: var(--accent-400);
		color: var(--text-primary);
	}

	.rdz-yaml-code :global(.yh-key) {
		color: #80cbc4;
	}

	.rdz-yaml-code :global(.yh-val) {
		color: #ffcc80;
	}

	.rdz-yaml-code :global(.yh-punct) {
		color: var(--text-muted);
	}

	.rdz-yaml-code :global(.yh-line) {
		display: inline-block;
		width: 100%;
		padding: 0 var(--space-md);
		margin: 0 calc(-1 * var(--space-md));
		border-left: 2px solid transparent;
		transition:
			background 200ms ease,
			border-color 200ms ease;
	}

	.rdz-yaml-code :global(.yh-line.highlighted) {
		background: var(--bg-active);
		border-left-color: var(--accent-400);
		box-shadow: 0 0 16px var(--bg-active);
	}
</style>
