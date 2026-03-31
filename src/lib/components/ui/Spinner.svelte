<script lang="ts">
	type Props = {
		size?: number;
		class?: string;
	};

	let { size = 16, class: className = '' }: Props = $props();
</script>

<span
	class="z-spinner {className}"
	style="width: {size}px; height: {size}px; border-width: {Math.max(2, size / 8)}px;"
></span>

<style>
	.z-spinner {
		display: inline-block;
		border: 2px solid currentColor;
		border-top-color: transparent;
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
