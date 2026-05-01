<script lang="ts">
	import Icon from '@iconify/svelte';
	import auth from '$lib/state/auth.svelte';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';
</script>

{#if auth.user}
	<div class="z-auth-pill">
		{#if auth.user.avatar}
			<img src={auth.user.avatar} alt={auth.user.displayName} />
		{:else}
			<Icon icon="mdi:account-circle" />
		{/if}
		<span>{auth.user.displayName}</span>
		<button
			class="z-auth-logout"
			onclick={async () => await auth.logout()}
			title={(i18nState.locale && m.sync_logOut()) || ''}
		>
			<Icon icon="mdi:logout" />
		</button>
	</div>
{/if}

<style>
	.z-auth-pill {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 4px 4px 4px 12px;
		border-radius: var(--radius-full);
		background: var(--bg-active);
		border: 1px solid var(--border-accent);
		font-size: 12px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.z-auth-pill img,
	.z-auth-pill :global(svg:first-child) {
		width: 20px;
		height: 20px;
		border-radius: 50%;
		object-fit: cover;
	}

	.z-auth-logout {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: none;
		border-radius: 50%;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.z-auth-logout:hover {
		background: var(--bg-hover);
		color: var(--error);
	}
</style>
