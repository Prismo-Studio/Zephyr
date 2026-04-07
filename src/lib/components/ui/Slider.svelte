<script lang="ts">
	type Props = {
		value: number;
		min?: number;
		max?: number;
		step?: number;
		disabled?: boolean;
		showValue?: boolean;
		onchange?: (value: number) => void;
	};

	let {
		value,
		min = 0,
		max = 100,
		step = 1,
		disabled = false,
		showValue = true,
		onchange
	}: Props = $props();

	const pct = $derived.by(() => {
		if (max === min) return 0;
		return Math.max(0, Math.min(100, ((value - min) / (max - min)) * 100));
	});

	function handleInput(e: Event) {
		const v = Number((e.currentTarget as HTMLInputElement).value);
		if (!isNaN(v)) onchange?.(v);
	}
</script>

<div class="z-slider" class:disabled style="--pct: {pct}%">
	<input
		type="range"
		class="z-slider-input"
		{min}
		{max}
		{step}
		{value}
		{disabled}
		oninput={handleInput}
		onchange={handleInput}
	/>
	{#if showValue}
		<div class="z-slider-value">{value}</div>
	{/if}
</div>

<style>
	.z-slider {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		width: 100%;
	}

	.z-slider.disabled {
		opacity: 0.5;
		pointer-events: none;
	}

	.z-slider-input {
		flex: 1;
		min-width: 0;
		-webkit-appearance: none;
		appearance: none;
		height: 24px;
		background: transparent;
		cursor: pointer;
		margin: 0;
		padding: 0;
	}

	.z-slider-input:focus {
		outline: none;
	}

	/* Track — WebKit */
	.z-slider-input::-webkit-slider-runnable-track {
		height: 4px;
		border-radius: var(--radius-full);
		background: linear-gradient(
			to right,
			var(--accent-400) 0%,
			var(--accent-400) var(--pct),
			var(--bg-active) var(--pct),
			var(--bg-active) 100%
		);
		border: none;
	}

	/* Track — Firefox */
	.z-slider-input::-moz-range-track {
		height: 4px;
		border-radius: var(--radius-full);
		background: var(--bg-active);
		border: none;
	}
	.z-slider-input::-moz-range-progress {
		height: 4px;
		border-radius: var(--radius-full);
		background: var(--accent-400);
	}

	/* Thumb — WebKit */
	.z-slider-input::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 16px;
		height: 16px;
		border-radius: 50%;
		background: var(--accent-400);
		border: none;
		margin-top: -6px;
		cursor: pointer;
		transition: transform 100ms ease;
	}

	.z-slider-input::-webkit-slider-thumb:hover {
		transform: scale(1.2);
	}

	/* Thumb — Firefox */
	.z-slider-input::-moz-range-thumb {
		width: 16px;
		height: 16px;
		border-radius: 50%;
		background: var(--accent-400);
		border: none;
		cursor: pointer;
		transition: transform 100ms ease;
	}

	.z-slider-input::-moz-range-thumb:hover {
		transform: scale(1.2);
	}

	.z-slider-value {
		min-width: 40px;
		padding: 3px 10px;
		text-align: center;
		font-family: var(--font-mono, monospace);
		font-size: 12px;
		font-weight: 600;
		color: var(--text-primary);
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-sm);
	}
</style>
