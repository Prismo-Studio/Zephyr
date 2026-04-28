<script lang="ts">
	type Props = {
		checked?: boolean;
		disabled?: boolean;
		onchange?: (checked: boolean) => void;
	};

	let { checked = false, disabled = false, onchange }: Props = $props();
</script>

<button
	class="z-toggle"
	class:active={checked}
	{disabled}
	role="switch"
	aria-checked={checked}
	onclick={() => onchange?.(!checked)}
>
	<span class="z-toggle-thumb"></span>
</button>

<style>
	.z-toggle {
		position: relative;
		width: 40px;
		height: 22px;
		border-radius: var(--radius-full);
		background: var(--bg-overlay);
		border: 1px solid var(--border-default);
		cursor: pointer;
		transition: all var(--transition-fast);
		flex-shrink: 0;
	}

	.z-toggle:hover:not(:disabled) {
		border-color: var(--border-strong);
	}

	.z-toggle.active {
		background: var(--accent-400);
		border-color: var(--accent-400);
	}

	.z-toggle:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.z-toggle-thumb {
		position: absolute;
		top: 2px;
		left: 2px;
		width: 16px;
		height: 16px;
		border-radius: 50%;
		background: var(--text-secondary);
		transition: all var(--transition-fast);
	}

	.z-toggle.active .z-toggle-thumb {
		transform: translateX(18px);
		background: var(--text-inverse);
	}
</style>
