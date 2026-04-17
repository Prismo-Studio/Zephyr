<script lang="ts">
	type Props = {
		checked?: boolean;
		disabled?: boolean;
		size?: number;
		onchange?: (checked: boolean) => void;
	};

	let { checked = $bindable(false), disabled = false, size = 16, onchange }: Props = $props();
</script>

<button
	class="z-checkbox"
	class:checked
	{disabled}
	role="checkbox"
	aria-checked={checked}
	style="width: {size}px; height: {size}px;"
	onclick={(e) => {
		e.stopPropagation();
		if (disabled) return;
		checked = !checked;
		onchange?.(checked);
	}}
>
	{#if checked}
		<svg viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
			<path
				d="M2.5 6L5 8.5L9.5 3.5"
				stroke="currentColor"
				stroke-width="1.8"
				stroke-linecap="round"
				stroke-linejoin="round"
			/>
		</svg>
	{/if}
</button>

<style>
	.z-checkbox {
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 4px;
		border: 1.5px solid var(--border-default);
		background: transparent;
		color: transparent;
		cursor: pointer;
		transition: all 120ms ease;
		padding: 0;
		flex-shrink: 0;
	}

	.z-checkbox:hover:not(:disabled) {
		border-color: var(--accent-400);
		background: var(--bg-active);
	}

	.z-checkbox.checked {
		background: var(--accent-400);
		border-color: var(--accent-400);
		color: var(--bg-base);
	}

	.z-checkbox.checked:hover:not(:disabled) {
		background: var(--accent-300);
		border-color: var(--accent-300);
	}

	.z-checkbox:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.z-checkbox svg {
		width: 70%;
		height: 70%;
	}
</style>
