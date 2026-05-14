<script lang="ts">
	import { page } from '$app/state';
	import { onMount, onDestroy } from 'svelte';
	import { convertFileSrc, invoke } from '@tauri-apps/api/core';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import * as api from '$lib/api';
	import plugins from '$lib/state/plugins.svelte';
	import Header from '$lib/components/layout/Header.svelte';
	import Icon from '@iconify/svelte';

	let pluginId = $derived(page.params.id);
	let pluginEntry = $derived(plugins.list.find((p) => p.id === pluginId));

	let iframeUrl = $state('');
	let loadFailed = $state(false);
	let bust = $state(0);
	let iframeEl: HTMLIFrameElement | null = $state(null);
	let unlisten: UnlistenFn | null = null;

	// Generic ctx API exposed to plugins via postMessage.
	// Plugins call `window.zephyr.<group>.<method>(args)` (see the @zephyr-plugin/sdk
	// helper). Each call routes to the matching handler here. New capabilities
	// require explicit registration — anything unlisted is refused so a
	// compromised plugin bundle can't escalate.
	async function handleBridgeMessage(evt: MessageEvent) {
		const data = evt.data;
		if (!data || typeof data.type !== 'string' || typeof data.id !== 'number') return;
		if (!data.type.startsWith('zephyr.')) return;
		const reply = (body: object) => {
			(evt.source as Window | null)?.postMessage({ id: data.id, ...body }, '*');
		};
		try {
			const result = await dispatchCtxCall(data.type, data.payload ?? {});
			reply({ result });
		} catch (err) {
			reply({ error: err instanceof Error ? err.message : String(err) });
		}
	}

	async function dispatchCtxCall(type: string, payload: any): Promise<unknown> {
		switch (type) {
			case 'zephyr.storage.get':
				return await invoke('plugin_storage_get', { id: pluginId });
			case 'zephyr.storage.set':
				return await invoke('plugin_storage_set', { id: pluginId, value: payload?.value });
			case 'zephyr.openExternal':
				if (typeof payload?.url !== 'string') throw new Error('url required');
				return await invoke('plugin_open_external', { url: payload.url });
			case 'zephyr.notify':
				if (typeof payload?.message === 'string') {
					const { pushToast } = await import('$lib/toast.svelte');
					pushToast({
						type: payload?.kind === 'error' ? 'error' : 'info',
						message: payload.message,
						name: payload?.title
					});
				}
				return null;
			case 'zephyr.plugin.info':
				return {
					id: pluginEntry?.id,
					name: pluginEntry?.name,
					version: pluginEntry?.version,
					dev: pluginEntry?.dev ?? false
				};
			case 'zephyr.fs.writeBlob': {
				if (typeof payload?.filename !== 'string') throw new Error('filename required');
				const bytes = payload.bytes;
				if (!Array.isArray(bytes) && !(bytes instanceof Uint8Array)) {
					throw new Error('bytes required (array or Uint8Array)');
				}
				return await invoke('plugin_fs_write_blob', {
					id: pluginId,
					filename: payload.filename,
					bytes: Array.from(bytes as ArrayLike<number>)
				});
			}
			case 'zephyr.fs.list':
				return await invoke('plugin_fs_list', {
					id: pluginId,
					extension: payload?.extension ?? null
				});
			case 'zephyr.fs.delete':
				if (typeof payload?.filename !== 'string') throw new Error('filename required');
				return await invoke('plugin_fs_delete', { id: pluginId, filename: payload.filename });
			case 'zephyr.fs.getUrl': {
				if (typeof payload?.filename !== 'string') throw new Error('filename required');
				const raw = (await invoke('plugin_fs_get_url', {
					id: pluginId,
					filename: payload.filename
				})) as string;
				// Plugin receives an asset:// URL ready for <video src=…>.
				return raw.startsWith('file://') ? convertFileSrc(raw.slice('file://'.length)) : raw;
			}
			case 'zephyr.fs.openFolder':
				return await invoke('plugin_fs_open_folder', { id: pluginId });
			default:
				throw new Error(`Unknown ctx call: ${type}`);
		}
	}

	async function resolveUrl() {
		try {
			const raw = await api.plugins.getPluginUiUrl(pluginId);
			if (!raw) {
				iframeUrl = '';
				loadFailed = true;
				return;
			}
			// Local dev plugin: raw is `file://<abs-path>`. Convert to the
			// WebView-serveable URL via Tauri's asset protocol.
			const url = raw.startsWith('file://') ? convertFileSrc(raw.slice('file://'.length)) : raw;
			iframeUrl = `${url}?t=${bust || Date.now()}`;
			loadFailed = false;
		} catch {
			iframeUrl = '';
			loadFailed = true;
		}
	}

	$effect(() => {
		pluginId; // re-run when route changes
		bust = Date.now();
		resolveUrl();
	});

	onMount(async () => {
		unlisten = await listen<{ id: string }>('dev_plugin_changed', (evt) => {
			if (evt.payload.id === pluginId) {
				bust = Date.now();
				resolveUrl();
			}
		});
		window.addEventListener('message', handleBridgeMessage);
	});

	onDestroy(() => {
		unlisten?.();
		window.removeEventListener('message', handleBridgeMessage);
	});
</script>

<div class="z-feature-page">
	<div class="z-feature-header-wrapper">
		<Header
			title={pluginEntry?.sidebarLabel || pluginEntry?.name || pluginId}
			subtitle={pluginEntry?.description || 'Community plugin'}
		/>
	</div>

	<div class="z-feature-body">
		{#if iframeUrl}
			<iframe
				bind:this={iframeEl}
				class="z-feature-iframe"
				src={iframeUrl}
				title={pluginEntry?.name || pluginId}
				allow="display-capture; microphone; camera; autoplay; clipboard-read; clipboard-write"
			></iframe>
		{:else if loadFailed}
			<div class="z-feature-missing">
				<Icon icon="mdi:alert-circle-outline" />
				<p>
					This plugin doesn't ship a <code>ui/index.html</code>. Create one in
					<code>{pluginEntry?.devPath || pluginEntry?.name}/ui/index.html</code>
					to render UI here.
				</p>
			</div>
		{:else}
			<div class="z-feature-missing">Loading…</div>
		{/if}
	</div>
</div>

<style>
	.z-feature-page {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.z-feature-header-wrapper {
		width: 100%;
		max-width: 1200px;
		margin: 0 auto;
		padding: var(--space-xl) var(--space-xl) 0;
	}

	.z-feature-body {
		flex: 1;
		min-height: 0;
		padding: 0;
		width: 100%;
		display: flex;
	}

	.z-feature-iframe {
		width: 100%;
		flex: 1;
		border: 0;
		background: transparent;
	}

	.z-feature-missing {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-md);
		color: var(--text-muted);
		font-size: 14px;
		text-align: center;
		padding: var(--space-2xl);
	}

	.z-feature-missing :global(svg) {
		font-size: 36px;
		opacity: 0.6;
	}

	.z-feature-missing code {
		font-family: var(--font-mono);
		font-size: 12px;
		padding: 2px 6px;
		border-radius: 4px;
		background: var(--bg-elevated);
		color: var(--text-accent);
	}
</style>
