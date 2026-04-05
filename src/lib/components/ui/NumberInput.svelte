<script lang="ts">
	type Props = {
		value: number;
		min?: number;
		max?: number;
		step?: number;
		onchange?: (value: number) => void;
	};

	let { value, min, max, step = 1, onchange }: Props = $props();

	function decrement() {
		let newVal = value - step;
		if (min != null) newVal = Math.max(newVal, min);
		if (step < 1) newVal = Math.round(newVal * 100) / 100;
		onchange?.(newVal);
	}

	function increment() {
		let newVal = value + step;
		if (max != null) newVal = Math.min(newVal, max);
		if (step < 1) newVal = Math.round(newVal * 100) / 100;
		onchange?.(newVal);
	}

	function handleInput(e: Event) {
		const target = e.currentTarget as HTMLInputElement;
		const numVal = step < 1 ? parseFloat(target.value) : parseInt(target.value);
		if (!isNaN(numVal)) onchange?.(numVal);
	}
</script>

<div class="z-number-control">
	<button class="z-number-btn" onclick={decrement}>−</button>
	<input
		class="z-number-value"
		type="number"
		{value}
		{min}
		{max}
		{step}
		onchange={handleInput}
	/>
	<button class="z-number-btn" onclick={increment}>+</button>
</div>

<style>
	.z-number-control {
		display: flex;
		align-items: center;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-default);
		background: var(--bg-elevated);
		overflow: hidden;
	}

	.z-number-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		background: var(--bg-overlay);
		color: var(--text-muted);
		border: none;
		font-size: 16px;
		font-weight: 600;
		cursor: pointer;
		transition: all var(--transition-fast);
		user-select: none;
	}

	.z-number-btn:hover {
		background: var(--bg-active);
		color: var(--text-accent);
	}

	.z-number-value {
		width: 80px;
		text-align: center;
		border: none;
		background: transparent;
		color: var(--text-primary);
		font-family: var(--font-mono);
		font-size: 12px;
		outline: none;
		padding: 6px 4px;
		-moz-appearance: textfield;
	}

	.z-number-value::-webkit-inner-spin-button,
	.z-number-value::-webkit-outer-spin-button {
		-webkit-appearance: none;
		margin: 0;
	}
</style>
