<script lang="ts">
	import Icon from '@iconify/svelte';

	type Props = {
		viewMode: 'list' | 'grid';
	};

	let { viewMode }: Props = $props();
</script>

{#if viewMode === 'grid'}
	<div class="z-drop-placeholder-grid"></div>
{:else}
	<div class="z-drop-placeholder">
		<div class="z-drop-line"></div>
		<Icon icon="mdi:plus-circle" class="z-drop-icon" />
		<div class="z-drop-line"></div>
	</div>
{/if}

<style>
	.z-drop-placeholder-grid {
		width: 100%;
		border: 2px dashed var(--accent-400);
		border-radius: var(--radius-lg);
		background: var(--bg-active);
		overflow: hidden;
		position: relative;
		display: flex;
		align-items: center;
		justify-content: center;
		aspect-ratio: 0.7;
		min-height: 220px;
	}

	.z-drop-placeholder-grid::after {
		content: '';
		width: 32px;
		height: 32px;
		border-radius: 50%;
		border: 2px solid var(--accent-400);
		opacity: 0.3;
	}

	.z-drop-placeholder {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm) 0;
		animation: placeholderIn 150ms ease;
	}

	.z-drop-line {
		flex: 1;
		height: 2px;
		border-top: 2px dashed var(--accent-400);
		opacity: 0.6;
	}

	:global(.z-drop-icon) {
		font-size: 18px;
		color: var(--accent-400);
		flex-shrink: 0;
	}

	@keyframes placeholderIn {
		from {
			opacity: 0;
			padding: 0;
		}
		to {
			opacity: 1;
			padding: var(--space-sm) 0;
		}
	}
</style>
