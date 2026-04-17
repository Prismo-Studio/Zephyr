<script lang="ts">
	import Icon from '@iconify/svelte';
	import { formatModName } from '$lib/util';
	import { THUNDERSTORE_ICON_URL } from '$lib/constants/api.constants';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	type Props = {
		dependencies: string[] | null;
		ondepclick?: (author: string, name: string) => Promise<boolean> | boolean;
	};

	let { dependencies, ondepclick }: Props = $props();

	function parseDep(dep: string): { author: string; name: string; version: string } {
		const parts = dep.split('-');
		if (parts.length >= 3) {
			return { author: parts[0], name: parts[1], version: parts.slice(2).join('-') };
		} else if (parts.length === 2) {
			return { author: parts[0], name: parts[1], version: '' };
		}
		return { author: '', name: dep, version: '' };
	}
</script>

{#if dependencies && dependencies.length > 0}
	<div class="z-deps-list">
		{#each dependencies as dep}
			{@const parsed = parseDep(dep)}
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="z-dep-item"
				role="button"
				tabindex="0"
				onclick={async () => {
					await ondepclick?.(parsed.author, parsed.name);
				}}
			>
				<img
					src={THUNDERSTORE_ICON_URL(dep)}
					alt={parsed.name}
					class="z-dep-icon-img"
					onerror={(e) => {
						const target = e.currentTarget as HTMLImageElement;
						target.style.display = 'none';
						target.nextElementSibling?.classList.add('visible');
					}}
				/>
				<div class="z-dep-icon-fallback">
					<Icon icon="mdi:puzzle" />
				</div>
				<span class="z-dep-name">{formatModName(parsed.name)}</span>
				{#if parsed.author}
					<span class="z-dep-author">{parsed.author}</span>
				{/if}
				{#if parsed.version}
					<span class="z-dep-version">{parsed.version}</span>
				{/if}
				<span class="z-dep-arrow">
					<Icon icon="mdi:chevron-right" />
				</span>
			</div>
		{/each}
	</div>
{:else}
	<p class="z-details-empty">{i18nState.locale && m.modDetails_noContent()}</p>
{/if}

<style>
	.z-deps-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.z-dep-item {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-sm) var(--space-md);
		border-radius: var(--radius-md);
		transition: background var(--transition-fast);
		cursor: pointer;
	}

	.z-dep-item:hover {
		background: var(--bg-hover);
	}

	.z-dep-icon-img {
		width: 32px;
		height: 32px;
		border-radius: var(--radius-md);
		object-fit: cover;
		flex-shrink: 0;
		background: var(--bg-overlay);
		border: 1px solid var(--border-subtle);
	}

	.z-dep-icon-fallback {
		display: none;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: var(--radius-md);
		background: var(--bg-overlay);
		border: 1px solid var(--border-subtle);
		color: var(--text-muted);
		font-size: 16px;
		flex-shrink: 0;
	}

	.z-dep-icon-fallback.visible {
		display: flex;
	}

	.z-dep-name {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.z-dep-author {
		font-size: 11px;
		color: var(--text-muted);
		margin-left: -6px;
	}

	.z-dep-version {
		font-size: 11px;
		font-weight: 600;
		color: var(--text-accent);
		background: var(--bg-active);
		padding: 2px 8px;
		border-radius: var(--radius-full);
		border: 1px solid var(--shadow-glow);
		white-space: nowrap;
		flex-shrink: 0;
		margin-left: auto;
	}

	.z-dep-arrow {
		color: var(--text-muted);
		font-size: 18px;
		flex-shrink: 0;
		opacity: 0;
		transition: all 150ms ease;
	}

	.z-dep-item:hover .z-dep-arrow {
		opacity: 1;
		color: var(--text-accent);
		transform: translateX(2px);
	}

	.z-details-empty {
		text-align: center;
		color: var(--text-muted);
		font-size: 13px;
		padding: var(--space-2xl);
	}
</style>
