<script lang="ts">
	import Icon from '@iconify/svelte';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		count: number;
		onclick: () => void;
	};

	let { count, onclick }: Props = $props();
</script>

<button class="z-unknown-banner" {onclick}>
	<Icon icon="mdi:help-circle" />
	<span>{m.mods_unknownMods({ count: count.toString() })}</span>
	<Icon icon="mdi:chevron-right" class="z-unknown-chev" />
</button>

<style>
	.z-unknown-banner {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-md);
		background: rgba(255, 179, 71, 0.08);
		border: 1px solid rgba(255, 179, 71, 0.2);
		color: var(--warning);
		font-size: 12px;
		margin-bottom: var(--space-sm);
		width: 100%;
		text-align: left;
		cursor: pointer;
		font-family: inherit;
		transition: all var(--transition-fast);
	}

	.z-unknown-banner:hover {
		background: rgba(255, 179, 71, 0.15);
		border-color: rgba(255, 179, 71, 0.4);
	}

	.z-unknown-banner span {
		flex: 1;
	}

	.z-unknown-banner :global(.z-unknown-chev) {
		font-size: 16px;
		opacity: 0.6;
	}
</style>
