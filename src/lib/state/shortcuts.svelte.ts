import { platform } from '@tauri-apps/plugin-os';

export type Binding = {
	ctrl?: boolean;
	shift?: boolean;
	alt?: boolean;
	meta?: boolean;
	key: string;
};

export type ShortcutId =
	| 'toggleFullscreen'
	| 'refreshData'
	| 'globalSearch'
	| 'cycleProfilePrev'
	| 'cycleProfileNext'
	| 'zoomIn'
	| 'zoomOut'
	| 'toggleMultiplayer'
	| 'consoleServer'
	| 'consoleClient'
	| 'consoleToggle';

export type GroupId = 'general' | 'profiles' | 'display' | 'randomizer' | 'console';

export type ShortcutDef = {
	id: ShortcutId;
	group: GroupId;
	labelKey: string;
	defaultBinding: Binding;
};

const isMac = (() => {
	try {
		return platform() === 'macos';
	} catch {
		return false;
	}
})();

export const SHORTCUT_DEFS: ShortcutDef[] = [
	{
		id: 'toggleFullscreen',
		group: 'general',
		labelKey: 'shortcuts_toggleFullscreen',
		defaultBinding: isMac ? { meta: true, key: 'f' } : { key: 'F11' }
	},
	{
		id: 'refreshData',
		group: 'general',
		labelKey: 'shortcuts_refreshData',
		defaultBinding: { ctrl: true, key: 'r' }
	},
	{
		id: 'globalSearch',
		group: 'general',
		labelKey: 'shortcuts_globalSearch',
		defaultBinding: { ctrl: true, key: 'f' }
	},
	{
		id: 'cycleProfilePrev',
		group: 'profiles',
		labelKey: 'shortcuts_cycleProfilePrev',
		defaultBinding: { ctrl: true, key: 'ArrowLeft' }
	},
	{
		id: 'cycleProfileNext',
		group: 'profiles',
		labelKey: 'shortcuts_cycleProfileNext',
		defaultBinding: { ctrl: true, key: 'ArrowRight' }
	},
	{
		id: 'zoomIn',
		group: 'display',
		labelKey: 'shortcuts_zoomIn',
		defaultBinding: { ctrl: true, key: '=' }
	},
	{
		id: 'zoomOut',
		group: 'display',
		labelKey: 'shortcuts_zoomOut',
		defaultBinding: { ctrl: true, key: '-' }
	},
	{
		id: 'toggleMultiplayer',
		group: 'randomizer',
		labelKey: 'shortcuts_toggleMultiplayer',
		defaultBinding: { ctrl: true, key: 'b' }
	},
	{
		id: 'consoleServer',
		group: 'console',
		labelKey: 'shortcuts_consoleServer',
		defaultBinding: { ctrl: true, shift: true, key: 's' }
	},
	{
		id: 'consoleClient',
		group: 'console',
		labelKey: 'shortcuts_consoleClient',
		defaultBinding: { ctrl: true, shift: true, key: 'c' }
	},
	{
		id: 'consoleToggle',
		group: 'console',
		labelKey: 'shortcuts_consoleToggle',
		defaultBinding: { ctrl: true, shift: true, key: 't' }
	}
];

const STORAGE_KEY = 'zephyr-shortcuts';

function defaults(): Record<ShortcutId, Binding> {
	const out = {} as Record<ShortcutId, Binding>;
	for (const def of SHORTCUT_DEFS) out[def.id] = { ...def.defaultBinding };
	return out;
}

function load(): Record<ShortcutId, Binding> {
	const base = defaults();
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (!raw) return base;
		const parsed = JSON.parse(raw) as Partial<Record<ShortcutId, Binding>>;
		for (const def of SHORTCUT_DEFS) {
			const b = parsed[def.id];
			if (b && typeof b.key === 'string') base[def.id] = b;
		}
	} catch {
		// fall through to defaults
	}
	return base;
}

function persist(map: Record<ShortcutId, Binding>) {
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(map));
	} catch {
		// ignore quota errors
	}
}

const bindings = $state<Record<ShortcutId, Binding>>(load());

export const shortcuts = {
	get all() {
		return bindings;
	},
	get(id: ShortcutId): Binding {
		return bindings[id];
	},
	set(id: ShortcutId, binding: Binding) {
		bindings[id] = binding;
		persist(bindings);
	},
	resetOne(id: ShortcutId) {
		const def = SHORTCUT_DEFS.find((d) => d.id === id);
		if (!def) return;
		bindings[id] = { ...def.defaultBinding };
		persist(bindings);
	},
	resetAll() {
		const d = defaults();
		for (const def of SHORTCUT_DEFS) bindings[def.id] = d[def.id];
		persist(bindings);
	},
	conflictWith(id: ShortcutId, binding: Binding): ShortcutId | null {
		for (const def of SHORTCUT_DEFS) {
			if (def.id === id) continue;
			if (sameBinding(bindings[def.id], binding)) return def.id;
		}
		return null;
	}
};

function normKey(k: string): string {
	return k.length === 1 ? k.toLowerCase() : k;
}

export function sameBinding(a: Binding, b: Binding): boolean {
	return (
		!!a.ctrl === !!b.ctrl &&
		!!a.shift === !!b.shift &&
		!!a.alt === !!b.alt &&
		!!a.meta === !!b.meta &&
		normKey(a.key) === normKey(b.key)
	);
}

/**
 * Match a keyboard event against a shortcut. The `=` and `-` keys also accept
 * their shifted variants (`+`, `_`) so zoom keeps working across keyboards.
 */
export function matchesShortcut(e: KeyboardEvent, id: ShortcutId): boolean {
	const b = bindings[id];
	if (!b) return false;
	if (!!b.ctrl !== e.ctrlKey) return false;
	if (!!b.shift !== e.shiftKey) return false;
	if (!!b.alt !== e.altKey) return false;
	if (!!b.meta !== e.metaKey) return false;
	const key = normKey(e.key);
	const target = normKey(b.key);
	if (key === target) return true;
	if (target === '=' && (key === '+' || key === '=')) return true;
	if (target === '-' && (key === '_' || key === '-')) return true;
	return false;
}

export function isEditableTarget(target: EventTarget | null): boolean {
	const el = target as HTMLElement | null;
	if (!el) return false;
	const tag = el.tagName;
	return tag === 'INPUT' || tag === 'TEXTAREA' || !!el.isContentEditable;
}

const MOD_LABELS_MAC = { ctrl: '⌃', shift: '⇧', alt: '⌥', meta: '⌘' };
const MOD_LABELS_OTHER = { ctrl: 'Ctrl', shift: 'Shift', alt: 'Alt', meta: 'Meta' };

const KEY_LABELS: Record<string, string> = {
	ArrowLeft: '←',
	ArrowRight: '→',
	ArrowUp: '↑',
	ArrowDown: '↓',
	' ': 'Space',
	Escape: 'Esc'
};

export function formatBinding(b: Binding): string {
	const labels = isMac ? MOD_LABELS_MAC : MOD_LABELS_OTHER;
	const parts: string[] = [];
	if (b.ctrl) parts.push(labels.ctrl);
	if (b.shift) parts.push(labels.shift);
	if (b.alt) parts.push(labels.alt);
	if (b.meta) parts.push(labels.meta);
	const key = KEY_LABELS[b.key] ?? (b.key.length === 1 ? b.key.toUpperCase() : b.key);
	parts.push(key);
	return parts.join(isMac ? '' : '+');
}

const MOD_KEYS = new Set(['Control', 'Shift', 'Alt', 'Meta', 'OS']);

/** Build a Binding from a keydown event, or null if only modifiers are held. */
export function bindingFromEvent(e: KeyboardEvent): Binding | null {
	if (MOD_KEYS.has(e.key)) return null;
	return {
		ctrl: e.ctrlKey || undefined,
		shift: e.shiftKey || undefined,
		alt: e.altKey || undefined,
		meta: e.metaKey || undefined,
		key: e.key.length === 1 ? e.key.toLowerCase() : e.key
	};
}
