<script lang="ts">
	import Icon from '@iconify/svelte';
	import { gamepadState } from '$lib/gamepad.svelte';
	import { onMount } from 'svelte';

	type Props = {
		open: boolean;
		value: string;
		onsubmit: (value: string) => void;
		oncancel: () => void;
	};

	let { open, value = $bindable(''), onsubmit, oncancel }: Props = $props();

	const rows = [
		['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'],
		['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
		['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'],
		['z', 'x', 'c', 'v', 'b', 'n', 'm']
	];
	const specialRow = ['shift', 'space', 'backspace', 'confirm'];

	let shifted = $state(false);
	let cursorRow = $state(0);
	let cursorCol = $state(0);

	function currentRowKeys(r: number): string[] {
		return r < rows.length ? rows[r] : specialRow;
	}

	function typeKey(key: string) {
		value += shifted ? key.toUpperCase() : key;
		shifted = false;
	}

	function pressCurrentKey() {
		const rowKeys = currentRowKeys(cursorRow);
		const key = rowKeys[cursorCol];
		if (!key) return;
		if (cursorRow < rows.length) {
			typeKey(key);
		} else {
			switch (key) {
				case 'shift':
					shifted = !shifted;
					break;
				case 'space':
					value += ' ';
					break;
				case 'backspace':
					value = value.slice(0, -1);
					break;
				case 'confirm':
					onsubmit(value);
					break;
			}
		}
	}

	function move(dir: 'up' | 'down' | 'left' | 'right') {
		const totalRows = rows.length + 1;
		switch (dir) {
			case 'up':
				cursorRow = Math.max(0, cursorRow - 1);
				cursorCol = Math.min(cursorCol, currentRowKeys(cursorRow).length - 1);
				break;
			case 'down':
				cursorRow = Math.min(totalRows - 1, cursorRow + 1);
				cursorCol = Math.min(cursorCol, currentRowKeys(cursorRow).length - 1);
				break;
			case 'left':
				cursorCol = Math.max(0, cursorCol - 1);
				break;
			case 'right':
				cursorCol = Math.min(currentRowKeys(cursorRow).length - 1, cursorCol + 1);
				break;
		}
	}

	function handleGamepadEvent(e: Event) {
		const btn = (e as CustomEvent).detail.button as number;
		const dirMap: Record<number, 'up' | 'down' | 'left' | 'right'> = {
			12: 'up',
			13: 'down',
			14: 'left',
			15: 'right'
		};
		if (btn in dirMap) move(dirMap[btn]);
		else if (btn === 0) pressCurrentKey();
		else if (btn === 1) oncancel();
		else if (btn === 2) value = value.slice(0, -1);
		else if (btn === 9) onsubmit(value);
		else if (btn === 10) shifted = !shifted;
	}

	$effect(() => {
		if (open) {
			cursorRow = 0;
			cursorCol = 0;
			window.addEventListener('gamepad-kb', handleGamepadEvent);
		} else {
			window.removeEventListener('gamepad-kb', handleGamepadEvent);
		}
		return () => window.removeEventListener('gamepad-kb', handleGamepadEvent);
	});

	let type = $derived(gamepadState.controllerType);
</script>

{#if open}
	<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="z-kb-overlay">
		<div class="z-kb-panel">
			<div class="z-kb-header">
				<div class="z-kb-input">{value}<span class="z-kb-cursor">|</span></div>
				<div class="z-kb-hints">
					<span><kbd>{type === 'playstation' ? '✕' : 'A'}</kbd> Type</span>
					<span><kbd>{type === 'playstation' ? '□' : 'X'}</kbd> Delete</span>
					<span><kbd>L3</kbd> Shift</span>
					<span><kbd>{type === 'playstation' ? '○' : 'B'}</kbd> Close</span>
				</div>
			</div>

			<div class="z-kb-rows">
				{#each rows as row, ri}
					<div class="z-kb-row">
						{#each row as key, ci}
							<button
								class="z-kb-key"
								class:focused={cursorRow === ri && cursorCol === ci}
								onclick={() => {
									cursorRow = ri;
									cursorCol = ci;
									typeKey(key);
								}}
							>
								{shifted ? key.toUpperCase() : key}
							</button>
						{/each}
					</div>
				{/each}

				<div class="z-kb-row z-kb-bottom">
					<button
						class="z-kb-key z-kb-special"
						class:active={shifted}
						class:focused={cursorRow === rows.length && cursorCol === 0}
						onclick={() => {
							cursorRow = rows.length;
							cursorCol = 0;
							shifted = !shifted;
						}}
					>
						<Icon icon="mdi:arrow-up-bold" />
					</button>
					<button
						class="z-kb-key z-kb-space"
						class:focused={cursorRow === rows.length && cursorCol === 1}
						onclick={() => {
							cursorRow = rows.length;
							cursorCol = 1;
							value += ' ';
						}}
					>
						Space
					</button>
					<button
						class="z-kb-key z-kb-special"
						class:focused={cursorRow === rows.length && cursorCol === 2}
						onclick={() => {
							cursorRow = rows.length;
							cursorCol = 2;
							value = value.slice(0, -1);
						}}
					>
						<Icon icon="mdi:backspace" />
					</button>
					<button
						class="z-kb-key z-kb-confirm"
						class:focused={cursorRow === rows.length && cursorCol === 3}
						onclick={() => {
							cursorRow = rows.length;
							cursorCol = 3;
							onsubmit(value);
						}}
					>
						<Icon icon="mdi:check" />
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}

<style>
	.z-kb-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: flex-end;
		justify-content: center;
		z-index: 9999;
		padding-bottom: 32px;
		animation: fadeIn 150ms ease;
	}

	.z-kb-panel {
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		padding: 16px;
		width: min(600px, 90vw);
		box-shadow: 0 -8px 32px rgba(0, 0, 0, 0.4);
		animation: slideUp 200ms cubic-bezier(0.16, 1, 0.3, 1);
	}

	.z-kb-header {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-bottom: 12px;
	}

	.z-kb-input {
		width: 100%;
		padding: 10px 14px;
		border-radius: var(--radius-md);
		border: 1px solid var(--border-accent);
		background: var(--bg-base);
		color: var(--text-primary);
		font-size: 16px;
		font-family: var(--font-body);
		min-height: 42px;
	}

	.z-kb-cursor {
		animation: blink 1s step-end infinite;
		color: var(--text-accent);
	}

	@keyframes blink {
		50% {
			opacity: 0;
		}
	}

	.z-kb-hints {
		display: flex;
		gap: 16px;
		justify-content: center;
		font-size: 11px;
		color: var(--text-muted);
	}

	.z-kb-hints kbd {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 20px;
		height: 18px;
		padding: 0 4px;
		border-radius: 4px;
		background: var(--bg-overlay);
		border: 1px solid var(--border-subtle);
		font-size: 10px;
		font-weight: 700;
		color: var(--text-secondary);
		margin-right: 4px;
	}

	.z-kb-rows {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.z-kb-row {
		display: flex;
		justify-content: center;
		gap: 4px;
	}

	.z-kb-key {
		min-width: 40px;
		height: 40px;
		border-radius: var(--radius-md);
		border: 2px solid transparent;
		background: var(--bg-overlay);
		color: var(--text-primary);
		font-size: 15px;
		font-weight: 600;
		font-family: var(--font-body);
		cursor: pointer;
		transition: all 80ms ease;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.z-kb-key:hover {
		background: var(--bg-hover);
	}

	.z-kb-key:active {
		background: var(--bg-active);
		transform: scale(0.95);
	}

	.z-kb-key.focused {
		border-color: var(--accent-400);
		background: var(--bg-active);
		box-shadow: 0 0 12px var(--border-accent);
	}

	.z-kb-special {
		min-width: 56px;
		font-size: 18px;
		color: var(--text-secondary);
	}

	.z-kb-special.active {
		background: var(--bg-active);
		color: var(--text-accent);
	}

	.z-kb-space {
		flex: 1;
		font-size: 12px;
		color: var(--text-muted);
		letter-spacing: 0.05em;
	}

	.z-kb-confirm {
		min-width: 56px;
		background: var(--accent-400);
		color: var(--text-inverse);
		font-size: 18px;
	}

	.z-kb-confirm:hover,
	.z-kb-confirm.focused {
		background: var(--accent-300);
	}

	.z-kb-bottom {
		margin-top: 4px;
	}

	@keyframes fadeIn {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}

	@keyframes slideUp {
		from {
			transform: translateY(20px);
			opacity: 0;
		}
		to {
			transform: translateY(0);
			opacity: 1;
		}
	}
</style>
