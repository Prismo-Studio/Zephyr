<script lang="ts">
	import SidebarGameSelector from './SidebarGameSelector.svelte';
	import SidebarNav from './SidebarNav.svelte';
	import SidebarLauncher from './SidebarLauncher.svelte';
	import SidebarProfileSwitcher from './SidebarProfileSwitcher.svelte';

	const { legendActive = false } = $props<{ legendActive?: boolean }>();
</script>

<aside class="z-sidebar">
	<SidebarGameSelector />
	<SidebarNav />

	<div class="z-sidebar-bottom" class:legend-active={legendActive}>
		<SidebarLauncher />
		<SidebarProfileSwitcher />
	</div>
</aside>

<style>
	.z-sidebar {
		display: flex;
		flex-direction: column;
		width: 78px;
		min-width: 78px;
		height: 100%;
		background: var(--bg-surface);
		border-right: 1px solid var(--border-subtle);
		padding: var(--space-md) 0;
		gap: var(--space-sm);
		position: relative;
		z-index: var(--z-dropdown);
	}

	.z-sidebar-bottom {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-sm);
		padding: 0 var(--space-sm);
		margin-top: auto;
	}

	/* Bump bottom margin when the gamepad legend bar is visible — the launcher
	   button would otherwise overlap with it on small viewports. */
	.z-sidebar-bottom.legend-active {
		margin-bottom: 32px;
	}
</style>
