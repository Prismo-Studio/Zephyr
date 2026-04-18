<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { CommandRegistry, CommandDef } from '../core/command-registry';
	import { parseLine } from '../core/command-parser';

	type Props = {
		registry: CommandRegistry;
		history: string[];
		prefix: '/' | '!';
		onsubmit: (line: string) => void | Promise<void>;
		onhelp?: () => void;
	};
	let { registry, history, prefix, onsubmit, onhelp }: Props = $props();

	let value = $state('');
	let inputEl: HTMLInputElement | undefined = $state();
	let historyIndex = $state(-1);
	let acOpen = $state(false);
	let acIndex = $state(0);

	const parsed = $derived(parseLine(value));

	/** Autocomplete candidates: only when the first char is the prefix and
	 *  we haven't typed a space yet (i.e. still editing the command name). */
	const candidates = $derived.by((): CommandDef[] => {
		const trimmed = value.trimStart();
		if (!trimmed.startsWith(prefix)) return [];
		if (trimmed.includes(' ')) return [];
		const partial = trimmed.slice(1);
		return registry.autocomplete(prefix, partial, /* includeComingSoon = */ false).slice(0, 8);
	});

	const currentDef = $derived.by((): CommandDef | undefined => {
		if (parsed.kind !== 'command') return undefined;
		if (parsed.prefix !== prefix) return undefined;
		return registry.lookup(prefix, parsed.name);
	});

	/** Ghost hint shown under the input: shows the next expected arg name. */
	const ghostHint = $derived.by(() => {
		if (!currentDef || parsed.kind !== 'command') return '';
		const typedArgs = parsed.args.length;
		const nextArg = currentDef.args[typedArgs];
		if (!nextArg) return '';
		const wrapped = nextArg.optional ? `[${nextArg.name}]` : `<${nextArg.name}>`;
		return `next: ${wrapped}`;
	});

	function acceptCandidate(i: number) {
		const c = candidates[i];
		if (!c) return;
		value = `${c.prefix}${c.name} `;
		acOpen = false;
		acIndex = 0;
		inputEl?.focus();
	}

	function onInput() {
		acOpen = candidates.length > 0 && value.length > 0;
		acIndex = 0;
		historyIndex = -1;
	}

	function onKey(e: KeyboardEvent) {
		// Help palette shortcut
		if (e.key === '/' && value === '' && e.ctrlKey) {
			e.preventDefault();
			onhelp?.();
			return;
		}

		if (acOpen && candidates.length > 0) {
			if (e.key === 'ArrowDown') {
				e.preventDefault();
				acIndex = (acIndex + 1) % candidates.length;
				return;
			}
			if (e.key === 'ArrowUp') {
				e.preventDefault();
				acIndex = (acIndex - 1 + candidates.length) % candidates.length;
				return;
			}
			if (e.key === 'Tab') {
				e.preventDefault();
				acceptCandidate(acIndex);
				return;
			}
			if (e.key === 'Escape') {
				acOpen = false;
				return;
			}
		}

		if (e.key === 'ArrowUp' && !acOpen) {
			e.preventDefault();
			if (history.length === 0) return;
			historyIndex = Math.min(historyIndex + 1, history.length - 1);
			value = history[historyIndex] ?? '';
			return;
		}
		if (e.key === 'ArrowDown' && !acOpen) {
			e.preventDefault();
			if (historyIndex <= 0) {
				historyIndex = -1;
				value = '';
			} else {
				historyIndex -= 1;
				value = history[historyIndex] ?? '';
			}
			return;
		}

		if (e.key === 'Enter') {
			e.preventDefault();
			if (acOpen && candidates.length > 0) {
				acceptCandidate(acIndex);
				return;
			}
			const line = value;
			if (!line.trim()) return;
			value = '';
			acOpen = false;
			historyIndex = -1;
			void onsubmit(line);
		}
	}

	export function focus() {
		inputEl?.focus();
	}
</script>

<div class="zc-input-wrap">
	{#if acOpen && candidates.length > 0}
		<div class="zc-ac" role="listbox">
			{#each candidates as c, i (c.prefix + c.name)}
				<button
					class="zc-ac-row"
					class:active={i === acIndex}
					onclick={() => acceptCandidate(i)}
				>
					<span class="zc-ac-sig">{registry.signature(c)}</span>
					<span class="zc-ac-sum">{c.summary}</span>
				</button>
			{/each}
		</div>
	{/if}

	<div class="zc-input-row">
		<span class="zc-prompt">{prefix}</span>
		<input
			bind:this={inputEl}
			bind:value
			oninput={onInput}
			onkeydown={onKey}
			spellcheck="false"
			autocomplete="off"
			placeholder={prefix === '/' ? 'type /help for commands' : 'type !help, or chat…'}
		/>
		<button class="zc-help-btn" onclick={() => onhelp?.()} title="Help (Ctrl+/)">
			<Icon icon="mdi:help-circle-outline" />
		</button>
	</div>

	{#if ghostHint || currentDef}
		<div class="zc-ghost">
			{#if currentDef}
				<span class="zc-ghost-sig">{registry.signature(currentDef)}</span>
				<span class="zc-ghost-sep">·</span>
				<span>{currentDef.summary}</span>
			{/if}
			{#if ghostHint}
				<span class="zc-ghost-next">{ghostHint}</span>
			{/if}
		</div>
	{/if}
</div>

<style>
	.zc-input-wrap {
		position: relative;
		border-top: 1px solid var(--border-subtle);
		background: var(--bg-surface);
		flex-shrink: 0;
	}

	.zc-input-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 16px;
	}

	.zc-prompt {
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		font-size: 16px;
		font-weight: 700;
		color: var(--accent-400);
		width: 14px;
		text-align: center;
	}

	input {
		flex: 1;
		background: transparent;
		border: none;
		outline: none;
		color: var(--text-primary);
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		font-size: 14px;
		caret-color: var(--accent-400);
	}

	input::placeholder {
		color: var(--text-muted);
		font-style: italic;
	}

	.zc-help-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: 1px solid var(--border-default);
		background: transparent;
		border-radius: var(--radius-sm);
		color: var(--text-muted);
		cursor: pointer;
	}
	.zc-help-btn:hover {
		border-color: var(--accent-400);
		color: var(--accent-400);
	}

	.zc-ghost {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 0 16px 6px 38px;
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		font-size: 11px;
		color: var(--text-muted);
	}
	.zc-ghost-sig {
		color: var(--accent-400);
	}
	.zc-ghost-sep {
		opacity: 0.5;
	}
	.zc-ghost-next {
		margin-left: auto;
		color: var(--text-secondary);
	}

	.zc-ac {
		position: absolute;
		bottom: 100%;
		left: 0;
		right: 0;
		background: var(--bg-surface);
		border-top: 1px solid var(--border-subtle);
		border-bottom: 1px solid var(--border-subtle);
		max-height: 240px;
		overflow-y: auto;
		box-shadow: 0 -8px 24px rgba(0, 0, 0, 0.25);
	}
	.zc-ac-row {
		display: flex;
		align-items: baseline;
		gap: 12px;
		width: 100%;
		padding: 6px 16px;
		border: none;
		background: transparent;
		color: var(--text-secondary);
		text-align: left;
		cursor: pointer;
		font-family: inherit;
	}
	.zc-ac-row:hover,
	.zc-ac-row.active {
		background: var(--bg-hover);
		color: var(--text-primary);
	}
	.zc-ac-sig {
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		font-size: 12.5px;
		font-weight: 600;
		color: var(--accent-400);
		min-width: 180px;
	}
	.zc-ac-sum {
		font-size: 11px;
		color: var(--text-muted);
	}
</style>
