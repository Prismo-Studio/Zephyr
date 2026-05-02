<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import * as api from '$lib/api';
	import { customBgState, setCustomBg, clearCustomBg } from '$lib/design-system/customBg.svelte';
	import { open as selectFile } from '@tauri-apps/plugin-dialog';
	import { m } from '$lib/paraglide/messages';
	import { i18nState } from '$lib/i18nCore.svelte';

	const ALLOWED_EXTENSIONS = [
		'png',
		'jpg',
		'jpeg',
		'webp',
		'gif',
		'bmp',
		'avif',
		'mp4',
		'webm',
		'mov',
		'm4v'
	];

	let uploading = $state(false);
	let error = $state<string | null>(null);

	async function pick() {
		const selected = await selectFile({
			multiple: false,
			directory: false,
			filters: [{ name: 'Image or video', extensions: ALLOWED_EXTENSIONS }]
		});
		if (!selected || typeof selected !== 'string') return;
		error = null;
		uploading = true;
		try {
			const probe = await api.prefs.probeCustomBackground(selected);
			const limit = probe.kind === 'video' ? probe.max_video_bytes : probe.max_image_bytes;
			if (probe.size > limit) {
				const sizeMb = (probe.size / 1024 / 1024).toFixed(1);
				const maxMb = Math.round(limit / 1024 / 1024).toString();
				error =
					(i18nState.locale &&
						m.prefs_custom_bgMedia_tooLarge({
							size: sizeMb,
							max: maxMb,
							kind: probe.kind
						})) ||
					`File too large (${sizeMb} MB). Max: ${maxMb} MB.`;
				return;
			}
			const uploaded = await api.prefs.uploadCustomBackground(selected);
			setCustomBg({ kind: uploaded.kind, url: uploaded.url });
		} catch (err) {
			const raw = err instanceof Error ? err.message : String(err);
			error = (i18nState.locale && m.prefs_custom_bgMedia_uploadFailed({ reason: raw })) || raw;
		} finally {
			uploading = false;
		}
	}
</script>

<div class="z-custom-media">
	<div class="z-custom-media-head">
		<div>
			<div class="z-custom-media-title">
				{i18nState.locale && m.prefs_custom_bgMedia_title()}
			</div>
			<div class="z-custom-media-desc">
				{i18nState.locale && m.prefs_custom_bgMedia_desc()}
			</div>
		</div>
		<div class="z-custom-media-actions">
			<Button variant="primary" disabled={uploading} onclick={pick}>
				{#snippet icon()}<Icon icon="mdi:upload" />{/snippet}
				{uploading
					? i18nState.locale && m.prefs_custom_bgMedia_uploading()
					: customBgState.media
						? i18nState.locale && m.prefs_custom_bgMedia_replace()
						: i18nState.locale && m.prefs_custom_bgMedia_pick()}
			</Button>
			{#if customBgState.media}
				<Button variant="ghost" disabled={uploading} onclick={() => clearCustomBg()}>
					{#snippet icon()}<Icon icon="mdi:delete" />{/snippet}
					{i18nState.locale && m.prefs_custom_bgMedia_clear()}
				</Button>
			{/if}
		</div>
	</div>

	{#if customBgState.media}
		<div class="z-custom-media-preview">
			{#if customBgState.media.kind === 'video'}
				<!-- svelte-ignore a11y_media_has_caption -->
				<video src={customBgState.media.url} autoplay loop muted playsinline></video>
			{:else}
				<img src={customBgState.media.url} alt="" />
			{/if}
			<span class="z-custom-media-kind">
				<Icon icon={customBgState.media.kind === 'video' ? 'mdi:video' : 'mdi:image'} />
				{customBgState.media.kind}
			</span>
		</div>
	{/if}

	{#if error}
		<p class="z-custom-media-err">{error}</p>
	{/if}
</div>

<style>
	.z-custom-media {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 12px;
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		border: 1px solid var(--border-subtle);
	}

	.z-custom-media-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}

	.z-custom-media-title {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.z-custom-media-desc {
		font-size: 11px;
		color: var(--text-muted);
		margin-top: 2px;
	}

	.z-custom-media-actions {
		display: flex;
		gap: 8px;
		flex-shrink: 0;
	}

	.z-custom-media-preview {
		position: relative;
		width: 100%;
		aspect-ratio: 16 / 9;
		border-radius: var(--radius-sm);
		overflow: hidden;
		background: #000;
		border: 1px solid var(--border-subtle);
	}

	.z-custom-media-preview img,
	.z-custom-media-preview video {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}

	.z-custom-media-kind {
		position: absolute;
		top: 8px;
		right: 8px;
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 4px 10px;
		border-radius: var(--radius-full);
		background: rgba(0, 0, 0, 0.6);
		color: #fff;
		font-size: 10px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.z-custom-media-kind :global(svg) {
		font-size: 12px;
	}

	.z-custom-media-err {
		margin: 0;
		color: var(--error);
		font-size: 11px;
	}
</style>
