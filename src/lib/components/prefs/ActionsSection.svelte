<script lang="ts">
	import Icon from '@iconify/svelte';
	import PrefSection from './PrefSection.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import * as api from '$lib/api';
	import { pushInfoToast } from '$lib/toast.svelte';
	import { shortenFileSize } from '$lib/util';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	async function clearCache(soft: boolean) {
		const freed = await api.profile.install.clearDownloadCache(soft);
		const message = soft ? m.menuBar_clearModCache_message_unsed : m.menuBar_clearModCache_message;
		pushInfoToast({ message: message({ size: shortenFileSize(freed) }) });
	}

	async function openLog() {
		await api.logger.openZephyrLog();
	}
</script>

{#key i18nState.locale}
	<PrefSection icon="mdi:wrench" title={m.dashboard_quickActions_title()}>
		<div class="z-settings-actions">
			<Button variant="secondary" size="sm" onclick={() => clearCache(true)}>
				{#snippet icon()}<Icon icon="mdi:broom" />{/snippet}
				{#snippet children()}{m.menuBar_file_item_6()}{/snippet}
			</Button>
			<Button variant="secondary" size="sm" onclick={() => clearCache(false)}>
				{#snippet icon()}<Icon icon="mdi:delete-sweep" />{/snippet}
				{#snippet children()}{m.menuBar_file_item_5()}{/snippet}
			</Button>
			<Button variant="ghost" size="sm" onclick={openLog}>
				{#snippet icon()}<Icon icon="mdi:file-document" />{/snippet}
				{#snippet children()}{m.menuBar_file_item_4()}{/snippet}
			</Button>
		</div>
	</PrefSection>
{/key}

<style>
	.z-settings-actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-sm);
	}
</style>
