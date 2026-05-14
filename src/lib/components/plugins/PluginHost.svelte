<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { convertFileSrc, invoke } from '@tauri-apps/api/core';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import * as api from '$lib/api';
	import plugins from '$lib/state/plugins.svelte';
	import games from '$lib/state/game.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { i18nState } from '$lib/i18nCore.svelte';
	import type { Plugin } from '$lib/types';

	let features = $derived(
		plugins.list.filter((p) => p.kind === 'feature' && p.enabled && !p.builtIn)
	);

	let currentPath = $state(typeof window !== 'undefined' ? window.location.pathname : '');
	let iframeUrls = $state<Record<string, string>>({});
	let iframeBust = $state<Record<string, number>>({});
	let iframeEls = $state<Record<string, HTMLIFrameElement | undefined>>({});
	let unlistenFns: UnlistenFn[] = [];
	let themeObserver: MutationObserver | null = null;

	async function resolveUrl(plugin: Plugin) {
		try {
			const raw = await api.plugins.getPluginUiUrl(plugin.id);
			if (!raw) {
				iframeUrls[plugin.id] = '';
				return;
			}
			const url = raw.startsWith('file://') ? convertFileSrc(raw.slice('file://'.length)) : raw;
			const bust = iframeBust[plugin.id] || Date.now();
			iframeUrls[plugin.id] = `${url}?t=${bust}`;
		} catch {
			iframeUrls[plugin.id] = '';
		}
	}

	$effect(() => {
		for (const plugin of features) {
			if (!(plugin.id in iframeUrls)) {
				iframeBust[plugin.id] = Date.now();
				resolveUrl(plugin);
			}
		}
		for (const id of Object.keys(iframeUrls)) {
			if (!features.some((p) => p.id === id)) {
				delete iframeUrls[id];
				delete iframeBust[id];
				delete iframeEls[id];
			}
		}
	});

	$effect(() => {
		const loc = i18nState.locale;
		broadcastEvent('locale.changed', { locale: loc });
	});

	function pluginIdFromSource(source: Window | null): string | null {
		if (!source) return null;
		for (const [id, el] of Object.entries(iframeEls)) {
			if (el?.contentWindow === source) return id;
		}
		return null;
	}

	async function handleBridgeMessage(evt: MessageEvent) {
		const data = evt.data;
		if (!data || typeof data.type !== 'string' || typeof data.id !== 'number') return;
		if (!data.type.startsWith('zephyr.')) return;
		const pluginId = pluginIdFromSource(evt.source as Window | null);
		if (!pluginId) return;
		const reply = (body: object) => {
			(evt.source as Window | null)?.postMessage({ id: data.id, ...body }, '*');
		};
		try {
			const result = await dispatchCtxCall(pluginId, data.type, data.payload ?? {});
			reply({ result });
		} catch (err) {
			reply({ error: err instanceof Error ? err.message : String(err) });
		}
	}

	async function dispatchCtxCall(pluginId: string, type: string, payload: any): Promise<unknown> {
		const entry = features.find((p) => p.id === pluginId);
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
					id: entry?.id,
					name: entry?.name,
					version: entry?.version,
					dev: entry?.dev ?? false
				};
			case 'zephyr.locale':
				return i18nState.locale;
			case 'zephyr.activeGame':
				return snapshotActiveGame();
			case 'zephyr.activeProfile':
				return snapshotActiveProfile();
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
				return raw.startsWith('file://') ? convertFileSrc(raw.slice('file://'.length)) : raw;
			}
			case 'zephyr.fs.openFolder':
				return await invoke('plugin_fs_open_folder', { id: pluginId });
			case 'zephyr.recording.start':
				return await invoke('plugin_recording_start', {
					args: {
						pluginId,
						filename: payload?.filename,
						fps: payload?.fps ?? 60,
						quality: payload?.quality ?? '1080p',
						windowTitle: payload?.windowTitle ?? null,
						withAudio: !!payload?.withAudio
					}
				});
			case 'zephyr.recording.stop':
				if (typeof payload?.sessionId !== 'string') throw new Error('sessionId required');
				return await invoke('plugin_recording_stop', { sessionId: payload.sessionId });
			default:
				throw new Error(`Unknown ctx call: ${type}`);
		}
	}

	function broadcastEvent(name: string, payload: unknown, only?: Window | null) {
		for (const el of Object.values(iframeEls)) {
			const target = el?.contentWindow;
			if (!target) continue;
			if (only && target !== only) continue;
			target.postMessage({ type: 'zephyr.event', event: name, payload }, '*');
		}
	}

	const THEME_VARS = [
		'bg-base',
		'bg-surface',
		'bg-elevated',
		'bg-overlay',
		'bg-hover',
		'bg-active',
		'border-subtle',
		'border-default',
		'border-strong',
		'border-accent',
		'text-primary',
		'text-secondary',
		'text-muted',
		'text-accent',
		'text-inverse',
		'accent-300',
		'accent-400',
		'accent-500',
		'accent-600',
		'accent-700',
		'success',
		'warning',
		'error',
		'info',
		'radius-sm',
		'radius-md',
		'radius-lg',
		'radius-xl',
		'radius-full',
		'shadow-sm',
		'shadow-md',
		'shadow-lg',
		'shadow-glow',
		'transition-fast',
		'transition-normal',
		'space-xs',
		'space-sm',
		'space-md',
		'space-lg',
		'space-xl',
		'space-2xl',
		'space-3xl',
		'font-display',
		'font-body',
		'font-mono'
	];

	function snapshotTheme(): Record<string, string> {
		if (typeof window === 'undefined') return {};
		const styles = getComputedStyle(document.documentElement);
		const out: Record<string, string> = {};
		for (const name of THEME_VARS) {
			const value = styles.getPropertyValue(`--${name}`).trim();
			if (value) out[name] = value;
		}
		return out;
	}

	function broadcastTheme(target?: Window | null) {
		broadcastEvent('theme.changed', snapshotTheme(), target);
	}

	function snapshotActiveGame() {
		const g = games.active;
		if (!g) return null;
		return { id: g.slug, name: g.name };
	}

	function snapshotActiveProfile() {
		const p = profiles.active;
		if (!p) return null;
		return { id: p.id, name: p.name };
	}

	onMount(async () => {
		const update = () => {
			currentPath = window.location.pathname;
		};
		window.addEventListener('popstate', update);
		const origPush = history.pushState.bind(history);
		history.pushState = function (data: any, unused: string, url?: string | URL | null) {
			origPush(data, unused, url);
			currentPath = window.location.pathname;
		};

		window.addEventListener('message', handleBridgeMessage);

		unlistenFns.push(
			await listen<{ id: string }>('dev_plugin_changed', (evt) => {
				iframeBust[evt.payload.id] = Date.now();
				const plugin = features.find((p) => p.id === evt.payload.id);
				if (plugin) resolveUrl(plugin);
			})
		);

		unlistenFns.push(
			await listen<{ gameId: string; gameName: string }>('zephyr_game_launched', (evt) => {
				broadcastEvent('game.launched', evt.payload);
			})
		);

		unlistenFns.push(
			await listen<{ gameId: string; gameName: string }>('zephyr_game_exited', (evt) => {
				broadcastEvent('game.exited', evt.payload);
			})
		);

		unlistenFns.push(
			await listen<{ gameId: string; gameName: string }>('zephyr_game_changed', (evt) => {
				broadcastEvent('game.changed', evt.payload);
			})
		);

		unlistenFns.push(
			await listen<{ profileName: string; profileIndex: number }>(
				'zephyr_profile_switched',
				(evt) => {
					broadcastEvent('profile.switched', { ...evt.payload, ...snapshotActiveProfile() });
				}
			)
		);

		themeObserver = new MutationObserver(() => broadcastTheme());
		themeObserver.observe(document.documentElement, {
			attributes: true,
			attributeFilter: ['data-theme', 'style']
		});
	});

	onDestroy(() => {
		for (const fn of unlistenFns) fn();
		window.removeEventListener('message', handleBridgeMessage);
		themeObserver?.disconnect();
	});

	function handleIframeLoad(plugin: Plugin) {
		const el = iframeEls[plugin.id];
		const target = el?.contentWindow;
		if (!target) return;
		broadcastTheme(target);
		broadcastEvent('locale.changed', { locale: i18nState.locale }, target);
		const g = snapshotActiveGame();
		if (g) broadcastEvent('game.changed', g, target);
		const p = snapshotActiveProfile();
		if (p) broadcastEvent('profile.switched', p, target);
	}
</script>

{#each features as plugin (plugin.id)}
	{#if iframeUrls[plugin.id]}
		<iframe
			bind:this={iframeEls[plugin.id]}
			class="z-plugin-iframe"
			class:visible={currentPath === `/plugins/feature/${plugin.id}`}
			src={iframeUrls[plugin.id]}
			title={plugin.name}
			onload={() => handleIframeLoad(plugin)}
			allow="display-capture; microphone; camera; autoplay; clipboard-read; clipboard-write"
		></iframe>
	{/if}
{/each}

<style>
	.z-plugin-iframe {
		display: none;
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		border: 0;
		background: transparent;
	}
	.z-plugin-iframe.visible {
		display: block;
	}
</style>
