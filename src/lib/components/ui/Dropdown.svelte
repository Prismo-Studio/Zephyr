<script lang="ts">
	import Icon from '@iconify/svelte';

	type Option = {
		value: string;
		label: string;
	};

	type Props = {
		options: Option[];
		value: string;
		onchange: (value: string) => void;
		placeholder?: string;
		/** Also accept a value that isn't in the list. The options become
		 *  suggestions, filtered as the user types. */
		editable?: boolean;
		invalid?: boolean;
	};

	let {
		options,
		value,
		onchange,
		placeholder = 'Select...',
		editable = false,
		invalid = false
	}: Props = $props();

	let open = $state(false);
	let wrapperEl: HTMLDivElement | undefined = $state();
	let dropdownEl: HTMLDivElement | undefined = $state();

	let selectedLabel = $derived(options.find((o) => o.value === value)?.label ?? placeholder);

	/** Narrow the suggestions to what has been typed so far, but never show an
	 *  empty menu - a value of the user's own making still deserves the list. */
	let visibleOptions = $derived.by(() => {
		if (!editable || !value) return options;

		const needle = value.toLowerCase();
		const matches = options.filter((option) => option.label.toLowerCase().includes(needle));
		return matches.length > 0 ? matches : options;
	});

	function select(option: Option) {
		onchange(option.value);
		open = false;
	}

	function toggle() {
		open = !open;
		if (open) {
			requestAnimationFrame(clampToViewport);
		}
	}

	function reveal() {
		if (open) return;
		open = true;
		requestAnimationFrame(clampToViewport);
	}

	function handleClickOutside(e: MouseEvent) {
		if (wrapperEl && !wrapperEl.contains(e.target as Node)) {
			open = false;
		}
	}

	let dropdownStyle = $state('');

	function clampToViewport() {
		if (!dropdownEl || !wrapperEl) return;
		const rect = wrapperEl.getBoundingClientRect();
		const spaceBelow = window.innerHeight - rect.bottom - 8;
		const maxH = Math.min(280, spaceBelow);

		if (spaceBelow < 120) {
			// open upward
			dropdownStyle = `bottom: calc(100% + 4px); max-height: 280px;`;
		} else {
			dropdownStyle = `top: calc(100% + 4px); max-height: ${maxH}px;`;
		}
	}

	$effect(() => {
		if (open) {
			document.addEventListener('click', handleClickOutside, true);
		} else {
			document.removeEventListener('click', handleClickOutside, true);
		}

		return () => {
			document.removeEventListener('click', handleClickOutside, true);
		};
	});
</script>

<div class="z-dropdown-wrapper" bind:this={wrapperEl}>
	{#if editable}
		<div class="z-dropdown-trigger z-dropdown-combo" class:invalid>
			<input
				class="z-dropdown-input"
				type="text"
				{value}
				{placeholder}
				spellcheck="false"
				autocomplete="off"
				oninput={(e) => {
					reveal();
					onchange(e.currentTarget.value);
				}}
				onfocus={reveal}
				onkeydown={(e) => e.key === 'Escape' && (open = false)}
			/>
			<Icon
				icon="mdi:chevron-down"
				class="z-dropdown-chevron {open ? 'open' : ''}"
				aria-hidden="true"
			/>
		</div>
	{:else}
		<button class="z-dropdown-trigger" onclick={toggle}>
			<span class="z-dropdown-label">{selectedLabel}</span>
			<Icon icon="mdi:chevron-down" class="z-dropdown-chevron {open ? 'open' : ''}" />
		</button>
	{/if}

	{#if open}
		<div class="z-dropdown-menu" style={dropdownStyle} bind:this={dropdownEl}>
			{#each visibleOptions as option}
				<button
					class="z-dropdown-option"
					class:active={value === option.value}
					onclick={() => select(option)}
				>
					{#if value === option.value}
						<Icon icon="mdi:check" class="z-dropdown-check" />
					{/if}
					<span>{option.label}</span>
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	.z-dropdown-wrapper {
		position: relative;
		width: 100%;
	}

	.z-dropdown-trigger {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 6px;
		width: 100%;
		height: 36px;
		padding: 0 12px;
		border-radius: var(--radius-md);
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		color: var(--text-primary);
		font-family: var(--font-body);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.z-dropdown-trigger:hover {
		border-color: var(--border-strong);
		background: var(--bg-overlay);
	}

	.z-dropdown-label {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		text-align: left;
	}

	/* Same shell as the plain trigger, with the label swapped for a text field
	   so the value can be typed as well as picked. */
	.z-dropdown-combo {
		cursor: text;
	}

	.z-dropdown-combo:focus-within {
		border-color: var(--accent-400);
		background: var(--bg-overlay);
	}

	.z-dropdown-combo.invalid {
		border-color: var(--color-error, #e05252);
	}

	.z-dropdown-input {
		flex: 1;
		min-width: 0;
		border: none;
		background: transparent;
		color: inherit;
		font-family: var(--font-mono, monospace);
		font-size: 12px;
		outline: none;
		padding: 0;
	}

	.z-dropdown-input::placeholder {
		color: var(--text-muted);
		font-family: var(--font-body);
	}

	:global(.z-dropdown-chevron) {
		font-size: 16px;
		color: var(--text-muted);
		transition: transform var(--transition-fast);
		flex-shrink: 0;
	}

	:global(.z-dropdown-chevron.open) {
		transform: rotate(180deg);
	}

	.z-dropdown-menu {
		position: absolute;
		left: 0;
		right: 0;
		min-width: 160px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		padding: 4px;
		z-index: var(--z-dropdown);
		box-shadow: var(--shadow-lg);
		animation: scaleIn 100ms ease;
		overflow-y: auto;
	}

	.z-dropdown-option {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 8px 12px;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		font-family: var(--font-body);
		font-size: 13px;
		cursor: pointer;
		transition: all var(--transition-fast);
		text-align: left;
	}

	.z-dropdown-option:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.z-dropdown-option.active {
		color: var(--text-accent);
		background: var(--bg-active);
	}

	:global(.z-dropdown-check) {
		font-size: 14px;
		color: var(--text-accent);
	}
</style>
