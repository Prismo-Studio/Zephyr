<script lang="ts">
	type Props = {
		value: number;
		max?: number;
		variant?: 'default' | 'accent' | 'success';
		size?: 'sm' | 'md';
		class?: string;
	};

	let {
		value,
		max = 100,
		variant = 'accent',
		size = 'md',
		class: className = ''
	}: Props = $props();

	let percent = $derived(Math.min(100, Math.max(0, (value / max) * 100)));
</script>

<div class="z-progress z-progress-{size} {className}">
	<div class="z-progress-bar z-progress-{variant}" style="width: {percent}%"></div>
</div>

<style>
	.z-progress {
		width: 100%;
		background: var(--bg-overlay);
		border-radius: var(--radius-full);
		overflow: hidden;
	}

	.z-progress-sm {
		height: 4px;
	}
	.z-progress-md {
		height: 6px;
	}

	.z-progress-bar {
		height: 100%;
		border-radius: var(--radius-full);
		transition: width var(--transition-normal);
	}

	.z-progress-accent {
		background: linear-gradient(90deg, var(--accent-600), var(--accent-400));
		box-shadow: 0 0 8px var(--accent-400);
	}
	.z-progress-default {
		background: var(--text-muted);
	}
	.z-progress-success {
		background: linear-gradient(90deg, #00a88a, var(--success));
	}
</style>
