<script lang="ts">
	type Tab = {
		id: string;
		label: string;
		icon?: string;
	};

	type Props = {
		tabs: Tab[];
		active?: string;
		onchange?: (id: string) => void;
		class?: string;
	};

	let { tabs, active = $bindable(''), onchange, class: className = '' }: Props = $props();

	if (!active && tabs.length > 0) active = tabs[0].id;
</script>

<div class="z-tabs {className}" role="tablist">
	{#each tabs as tab}
		<button
			class="z-tab"
			class:active={active === tab.id}
			role="tab"
			aria-selected={active === tab.id}
			onclick={() => {
				active = tab.id;
				onchange?.(tab.id);
			}}
		>
			{tab.label}
		</button>
	{/each}
</div>

<style>
	.z-tabs {
		display: flex;
		gap: 2px;
		background: var(--bg-elevated);
		border-radius: var(--radius-md);
		padding: 3px;
		border: 1px solid var(--border-subtle);
	}

	.z-tab {
		flex: 1;
		padding: 6px 12px;
		font-size: 12px;
		font-weight: 600;
		color: var(--text-muted);
		background: transparent;
		border: none;
		border-radius: var(--radius-sm);
		cursor: pointer;
		transition: all var(--transition-fast);
		font-family: var(--font-body);
	}

	.z-tab:hover {
		color: var(--text-secondary);
	}

	.z-tab.active {
		background: var(--bg-overlay);
		color: var(--text-accent);
		box-shadow: var(--shadow-sm);
	}
</style>
