/**
 * Anonymous usage telemetry via PostHog (EU cloud).
 *
 * Privacy guarantees:
 *  - **Opt-in only**: nothing is sent until the user enables the toggle in
 *    Settings → Privacy. Default = off.
 *  - **Anonymous**: PostHog generates a per-install random ID stored in
 *    localStorage. No Discord username, no IP logged, no PII captured.
 *  - **No autocapture**: pageviews, clicks, form events are NOT auto-tracked.
 *    Only the events explicitly listed in `EventName` below are sent.
 *  - **EU region**: data hosted in PostHog's EU cloud (RGPD-friendly).
 *
 * If `VITE_POSTHOG_KEY` is empty (e.g. local dev without env, forks without
 * their own key), telemetry is a no-op regardless of the toggle.
 */
import { PersistedState } from 'runed';

// Strongly-typed event catalog. Add new events here so the call sites get
// autocomplete + typo protection.
export type EventName =
	| 'app_started'
	| 'page_viewed'
	| 'game_selected'
	| 'mod_installed'
	| 'mod_removed'
	| 'mod_updated'
	| 'profile_created'
	| 'profile_synced'
	| 'profile_unsynced'
	| 'profile_launched'
	| 'discord_login'
	| 'discord_logout'
	| 'discord_session_expired'
	| 'ap_server_started'
	| 'ap_server_stopped'
	| 'ap_client_connected'
	| 'ap_seed_generated'
	| 'ap_apworld_imported'
	| 'ap_runtime_installed'
	| 'modal_opened'
	| 'feature_used'
	| 'error_occurred';

type EventProps = Record<string, string | number | boolean | null | undefined>;

const KEY = import.meta.env.VITE_POSTHOG_KEY as string | undefined;
const HOST =
	(import.meta.env.VITE_POSTHOG_HOST as string | undefined) ?? 'https://eu.i.posthog.com';

/** User-controlled toggle. Persisted in localStorage. Defaults to enabled. */
export const telemetryEnabled = new PersistedState<boolean>('telemetry.enabled', true);

let initialized = false;
// Lazy import — keep posthog-js out of the initial bundle when disabled.
let posthog: typeof import('posthog-js').default | null = null;

async function ensureInit() {
	if (initialized) return true;
	if (!KEY) return false;
	const mod = await import('posthog-js');
	posthog = mod.default;
	posthog.init(KEY, {
		api_host: HOST,
		person_profiles: 'identified_only',
		autocapture: false,
		capture_pageview: false,
		capture_pageleave: false,
		disable_session_recording: true
	});
	initialized = true;
	return true;
}

/**
 * Call once at app startup. Initialises PostHog only if the user has opted in.
 * Safe to call when `VITE_POSTHOG_KEY` is empty (no-op).
 */
export async function initTelemetry() {
	if (!telemetryEnabled.current) return;
	await ensureInit();
}

/**
 * Update the opt-in state. Initialises PostHog on enable, opts out on disable.
 */
export async function setTelemetryEnabled(value: boolean) {
	telemetryEnabled.current = value;
	if (value) {
		await ensureInit();
		posthog?.opt_in_capturing();
	} else {
		posthog?.opt_out_capturing();
	}
}

/**
 * Capture an event. Silently skipped when telemetry is disabled or unavailable.
 */
export function captureEvent(name: EventName, props?: EventProps) {
	if (!telemetryEnabled.current || !initialized || !posthog) return;
	posthog.capture(name, props);
}

/** Whether telemetry is technically available (i.e. the build has a key). */
export function telemetryAvailable(): boolean {
	return !!KEY;
}
