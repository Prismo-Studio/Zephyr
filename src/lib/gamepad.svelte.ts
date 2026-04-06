/**
 * Gamepad navigation support for Xbox and PlayStation controllers.
 *
 * Button mapping (standard gamepad layout):
 * - A / Cross (0): Confirm / Click
 * - B / Circle (1): Back / Escape
 * - D-pad Up (12), Down (13), Left (14), Right (15): Navigate focusable elements
 * - Right stick: Scroll content
 * - LB/RB (4/5): Switch tabs / sections
 * - Y / Triangle (3): Multi-select (hold + D-pad for chain select)
 * - Start / Menu (9): Focus batch actions
 */

const DEADZONE = 0.4;
const SCROLL_DEADZONE = 0.2;
const BUTTON_THRESHOLD = 0.5;
const REPEAT_DELAY = 400;
const REPEAT_RATE = 120;

// Standard gamepad button indices
const BTN = {
	A: 0, // Xbox A / PS Cross
	B: 1, // Xbox B / PS Circle
	X: 2, // Xbox X / PS Square
	Y: 3, // Xbox Y / PS Triangle
	LB: 4, // Left bumper / L1
	RB: 5, // Right bumper / R1
	LT: 6, // Left trigger / L2
	RT: 7, // Right trigger / R2
	SELECT: 8, // Back / Share
	START: 9, // Start / Options
	L3: 10, // Left stick press
	R3: 11, // Right stick press
	DPAD_UP: 12,
	DPAD_DOWN: 13,
	DPAD_LEFT: 14,
	DPAD_RIGHT: 15
} as const;

type Direction = 'up' | 'down' | 'left' | 'right';

// Plain variables for the rAF poll loop (not $state — rAF reads must be synchronous)
let _enabled = false;
let _connectedGamepad: string | null = null;
let _gamepadIndex: number | null = null;
let _gamepadRef: Gamepad | null = null; // Direct reference from event

// Reactive state for Svelte templates
let enabled = $state(false);
let connectedGamepad = $state<string | null>(null);
let keyboardOpen = $state(false);
let keyboardTarget: HTMLInputElement | null = null;

function syncReactive() {
	enabled = _enabled;
	connectedGamepad = _connectedGamepad;
	// Toggle body class for global CSS focus ring
	if (_enabled && _connectedGamepad) {
		document.body.classList.add('gamepad-active');
	} else {
		document.body.classList.remove('gamepad-active');
	}
}

let animFrameId: number | null = null;
let prevButtons: boolean[] = [];
let repeatTimers: Map<string, { last: number; started: number }> = new Map();
let multiSelectHeld = false; // Track multi-select button held state for chain select
let prevStickDir: Direction | null = null; // Track left stick direction for nav
let stickRepeatTimer: { last: number; started: number } | null = null;
let _initialFocusSet = false;

// Focus persistence across tab/page switches
let focusMemory: Map<string, string> = new Map(); // route -> data-mod-uuid or selector
let lastRoute = '';

// Elements to skip during gamepad navigation
const GAMEPAD_SKIP_SELECTOR =
	'.z-mod-category-tag, .z-mod-checkbox-wrapper *, .z-mod-install-btn, .z-titlebar-btn, .z-gamepad-legend *, .z-gamepad-debug *';

const FOCUSABLE_SELECTOR = [
	'button:not([disabled]):not([aria-hidden="true"])',
	'a[href]:not([disabled])',
	'input:not([disabled])',
	'select:not([disabled])',
	'[tabindex]:not([tabindex="-1"]):not([disabled])',
	'[role="button"]:not([disabled])',
	'[role="tab"]:not([disabled])',
	'[role="menuitem"]:not([disabled])'
].join(', ');

function getModalRoot(): HTMLElement | null {
	const modals = document.querySelectorAll<HTMLElement>('.z-modal-backdrop, .z-kb-overlay');
	for (let i = modals.length - 1; i >= 0; i--) {
		if (modals[i].offsetParent !== null || getComputedStyle(modals[i]).position === 'fixed') {
			return modals[i];
		}
	}
	return null;
}

function getFocusableElements(): HTMLElement[] {
	const modalRoot = getModalRoot();
	const root = modalRoot ?? document;
	const els = Array.from(root.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR));
	return els.filter((el) => {
		if (el.matches(GAMEPAD_SKIP_SELECTOR)) return false;
		const parentCard = el.closest('.z-mod-card, .z-mod-grid-card');
		if (parentCard && el !== parentCard) return false;
		if (!modalRoot && el.closest('.z-titlebar')) return false;
		if (el.offsetParent === null && getComputedStyle(el).position !== 'fixed') return false;
		const rect = el.getBoundingClientRect();
		return rect.width > 0 && rect.height > 0;
	});
}

function getCurrentFocusIndex(elements: HTMLElement[]): number {
	const active = document.activeElement as HTMLElement;
	if (!active) return -1;
	const idx = elements.indexOf(active);
	if (idx !== -1) return idx;
	// Check if active element is inside any focusable element
	for (let i = 0; i < elements.length; i++) {
		if (elements[i].contains(active)) return i;
	}
	return -1;
}

function focusElement(el: HTMLElement) {
	el.focus({ focusVisible: true } as FocusOptions);
	// Only scroll into view if the element is not already visible
	const rect = el.getBoundingClientRect();
	const inView = rect.top >= 0 && rect.bottom <= window.innerHeight;
	if (!inView) {
		el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
	}
}

function saveFocusForCurrentRoute() {
	const route = window.location.pathname;
	const active = document.activeElement as HTMLElement;
	if (!active || active === document.body) return;
	// Prefer data-mod-uuid for mod cards
	const uuid = active.getAttribute('data-mod-uuid');
	if (uuid) {
		focusMemory.set(route, `[data-mod-uuid="${uuid}"]`);
		return;
	}
	// For other elements, try to build a selector
	const id = active.getAttribute('id');
	if (id) {
		focusMemory.set(route, `#${id}`);
	}
}

function restoreFocusForRoute(route: string) {
	const selector = focusMemory.get(route);
	if (!selector) return;
	// Poll for the element since DOM might not be ready yet
	let attempts = 0;
	const tryRestore = () => {
		const el = document.querySelector<HTMLElement>(selector);
		if (el) {
			focusElement(el);
		} else if (attempts < 10) {
			attempts++;
			setTimeout(tryRestore, 50);
		}
	};
	requestAnimationFrame(tryRestore);
}

function isScrollableContainer(el: HTMLElement | null): el is HTMLElement {
	if (!el) return false;
	const style = getComputedStyle(el);
	return (
		(style.overflowY === 'auto' || style.overflowY === 'scroll' || style.overflowY === 'overlay') &&
		el.scrollHeight > el.clientHeight
	);
}

function getScrollContainer(): HTMLElement | null {
	const active = document.activeElement as HTMLElement | null;
	let el = active;
	while (el) {
		if (isScrollableContainer(el)) return el;
		el = (el as HTMLElement).parentElement;
	}

	// Fallback to known page containers
	const candidates = [
		'.z-settings-content',
		'.z-mods-content',
		'.z-browse-content', // browse page
		'.z-mod-details-body', // mod details panel
		'.z-content' // layout fallback
	];
	for (const sel of candidates) {
		const el = document.querySelector<HTMLElement>(sel);
		if (isScrollableContainer(el)) return el;
	}

	return document.scrollingElement as HTMLElement | null;
}

function focusBatchActionBar(): boolean {
	const selectors = [
		'.z-batch-bar .z-batch-actions button:not([disabled])',
		'.z-batch-bar .z-batch-select-all:not([disabled])',
		'.z-batch-bar .z-batch-clear:not([disabled])',
		'.z-batch-bar button:not([disabled])'
	];

	for (const selector of selectors) {
		const target = document.querySelector<HTMLElement>(selector);
		if (target) {
			focusElement(target);
			return true;
		}
	}

	return false;
}

function navigate(direction: Direction) {
	const elements = getFocusableElements();
	if (elements.length === 0) return;

	const currentIdx = getCurrentFocusIndex(elements);
	if (currentIdx === -1) {
		focusElement(elements[0]);
		if (multiSelectHeld) chainSelect(elements[0]);
		return;
	}

	// Grid navigation for .z-game-card in .z-games-grid
	const current = elements[currentIdx];
	if (
		current.classList.contains('z-game-card') &&
		current.parentElement?.classList.contains('z-games-grid')
	) {
		const grid = current.parentElement;
		const cards = Array.from(grid.querySelectorAll<HTMLElement>('.z-game-card'));
		const cardIdx = cards.indexOf(current);
		if (cardIdx === -1) return;
		const colCount = getComputedStyle(grid).gridTemplateColumns.split(' ').length;
		let nextIdx = cardIdx;
		switch (direction) {
			case 'left':
				nextIdx = Math.max(0, cardIdx - 1);
				break;
			case 'right':
				nextIdx = Math.min(cards.length - 1, cardIdx + 1);
				break;
			case 'up':
				nextIdx = cardIdx - colCount;
				break;
			case 'down':
				nextIdx = Math.min(cards.length - 1, cardIdx + colCount);
				break;
		}
		if (nextIdx >= 0) {
			focusElement(cards[nextIdx]);
			if (multiSelectHeld) chainSelect(cards[nextIdx]);
			return;
		}
	}

	// List navigation for .z-mod-card in .z-browse-list or .z-mods-list
	const parentList = current.closest('.z-browse-list, .z-mods-list');
	if (
		(current.classList.contains('z-mod-card') || current.classList.contains('z-mod-grid-card')) &&
		parentList
	) {
		const isGrid = parentList.classList.contains('z-grid-layout');
		const cards = Array.from(
			parentList.querySelectorAll<HTMLElement>('.z-mod-card, .z-mod-grid-card')
		);
		const cardIdx = cards.indexOf(current);
		let nextIdx = cardIdx;

		if (isGrid) {
			const colCount = getComputedStyle(parentList).gridTemplateColumns.split(' ').length;
			switch (direction) {
				case 'left':
					nextIdx = Math.max(0, cardIdx - 1);
					break;
				case 'right':
					nextIdx = cardIdx + 1;
					break;
				case 'up':
					nextIdx = cardIdx - colCount;
					break;
				case 'down':
					nextIdx = cardIdx + colCount;
					break;
			}
		} else {
			switch (direction) {
				case 'up':
					nextIdx = cardIdx - 1;
					break;
				case 'down':
					nextIdx = cardIdx + 1;
					break;
				case 'left':
				case 'right':
					return;
			}
		}
		if (nextIdx >= 0 && nextIdx < cards.length) {
			focusElement(cards[nextIdx]);
			if (multiSelectHeld) chainSelect(cards[nextIdx] as HTMLElement);
			return;
		}
		// nextIdx < 0 means we're at the top — fall through to spatial navigation
	}

	// Fallback: spatial navigation
	const currentRect = current.getBoundingClientRect();
	const cx = currentRect.left + currentRect.width / 2;
	const cy = currentRect.top + currentRect.height / 2;
	let bestIdx = -1;
	let bestScore = Infinity;
	for (let i = 0; i < elements.length; i++) {
		if (i === currentIdx) continue;
		const rect = elements[i].getBoundingClientRect();
		const ex = rect.left + rect.width / 2;
		const ey = rect.top + rect.height / 2;
		const dx = ex - cx;
		const dy = ey - cy;
		let valid = false;
		switch (direction) {
			case 'up':
				valid = dy < -5;
				break;
			case 'down':
				valid = dy > 5;
				break;
			case 'left':
				valid = dx < -5;
				break;
			case 'right':
				valid = dx > 5;
				break;
		}
		if (!valid) continue;
		let score: number;
		if (direction === 'up' || direction === 'down') {
			score = Math.abs(dy) + Math.abs(dx) * 2;
		} else {
			score = Math.abs(dx) + Math.abs(dy) * 2;
		}
		if (score < bestScore) {
			bestScore = score;
			bestIdx = i;
		}
	}
	if (bestIdx === -1) {
		if (direction === 'down' || direction === 'right') {
			bestIdx = (currentIdx + 1) % elements.length;
		} else {
			bestIdx = (currentIdx - 1 + elements.length) % elements.length;
		}
	}
	focusElement(elements[bestIdx]);
	if (multiSelectHeld) {
		chainSelect(elements[bestIdx]);
	}
}

function chainSelect(el: HTMLElement) {
	const card = el.closest('.z-mod-card') as HTMLElement | null;
	if (card) {
		card.dispatchEvent(
			new MouseEvent('click', {
				bubbles: true,
				cancelable: true,
				ctrlKey: true
			})
		);
	}
}

function pressButton(button: number) {
	const active = document.activeElement as HTMLElement;

	switch (button) {
		case BTN.A:
			if (active && active !== document.body) {
				if (active.tagName === 'INPUT' && (active as HTMLInputElement).type === 'text') {
					keyboardTarget = active as HTMLInputElement;
					keyboardOpen = true;
					return;
				}
				if (active.closest('a.z-nav-item')) {
					saveFocusForCurrentRoute();
				}
				if (
					active.classList.contains('z-mod-card') ||
					active.classList.contains('z-mod-grid-card')
				) {
					active.click();
				} else {
					const clickTarget =
						active.querySelector<HTMLElement>('button, a, [role="button"]') ?? active;
					clickTarget.click();
				}
			} else {
				navigate('down');
			}
			break;

		case BTN.Y: {
			if (active && active !== document.body) {
				const card = active.closest('.z-mod-card, .z-mod-grid-card') as HTMLElement | null;
				const target = card ?? active;
				target.dispatchEvent(
					new MouseEvent('click', {
						bubbles: true,
						cancelable: true,
						ctrlKey: true
					})
				);
			}
			break;
		}

		case BTN.SELECT:
			focusBatchActionBar();
			break;

		case BTN.START: {
			const launchBtn = document.querySelector<HTMLElement>('.z-launch-btn');
			if (launchBtn) focusElement(launchBtn);
			break;
		}

		case BTN.B: {
			// Close any open dropdown/menu by simulating a click outside
			const openMenu = document.querySelector(
				'.z-sort-dropdown, .z-version-dropdown, .z-game-dropdown'
			);
			if (openMenu) {
				document.body.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
				break;
			}
			const openDetails = document.querySelector('details[open]');
			if (openDetails) {
				(openDetails as HTMLDetailsElement).open = false;
				break;
			}
			const target = active && active !== document.body ? active : document.body;
			target.dispatchEvent(
				new KeyboardEvent('keydown', {
					key: 'Escape',
					code: 'Escape',
					bubbles: true,
					cancelable: true
				})
			);
			break;
		}

		case BTN.DPAD_UP:
			navigate('up');
			break;

		case BTN.DPAD_DOWN:
			navigate('down');
			break;

		case BTN.DPAD_LEFT:
			navigate('left');
			break;

		case BTN.DPAD_RIGHT:
			navigate('right');
			break;

		case BTN.RT: {
			const filterTarget =
				document.querySelector<HTMLElement>('.z-search-input input') ??
				document.querySelector<HTMLElement>('.z-filter-btn') ??
				document.querySelector<HTMLElement>('.z-sort-trigger');
			if (filterTarget) focusElement(filterTarget);
			break;
		}

		case BTN.LB:
		case BTN.RB: {
			// Save focus before tab switch
			saveFocusForCurrentRoute();

			// Try role="tab" elements first
			const tabs = Array.from(document.querySelectorAll<HTMLElement>('[role="tab"]'));
			const activeTab = tabs.findIndex((t) => t.getAttribute('aria-selected') === 'true');
			if (tabs.length > 0) {
				const nextIdx = button === BTN.LB ? activeTab - 1 : activeTab + 1;
				if (nextIdx >= 0 && nextIdx < tabs.length) {
					tabs[nextIdx].click();
					tabs[nextIdx].focus();
					break;
				}
			}

			// Fallback: navigate sidebar links
			const navLinks = Array.from(document.querySelectorAll<HTMLAnchorElement>('a.z-nav-item'));
			const currentNav = navLinks.findIndex((l) => l.classList.contains('active'));
			if (navLinks.length > 0) {
				const nextNav =
					button === BTN.LB
						? Math.max(0, currentNav - 1)
						: Math.min(navLinks.length - 1, currentNav + 1);
				if (nextNav !== currentNav) {
					navLinks[nextNav].click();
					// Restore focus for the new route after navigation
					const href = navLinks[nextNav].getAttribute('href') ?? '';
					setTimeout(() => restoreFocusForRoute(href), 100);
				}
			}
			break;
		}
	}
}

const REPEATABLE_BUTTONS: readonly number[] = [
	BTN.DPAD_UP,
	BTN.DPAD_DOWN,
	BTN.DPAD_LEFT,
	BTN.DPAD_RIGHT
];

function isButtonPressed(btn: GamepadButton): boolean {
	return btn.pressed || btn.value > BUTTON_THRESHOLD;
}

function handleRepeat(key: string, timestamp: number, action: () => void) {
	const timer = repeatTimers.get(key);
	if (!timer) {
		action();
		repeatTimers.set(key, { last: timestamp, started: timestamp });
	} else {
		const elapsed = timestamp - timer.started;
		const sinceLastRepeat = timestamp - timer.last;
		if (elapsed > REPEAT_DELAY && sinceLastRepeat > REPEAT_RATE) {
			action();
			timer.last = timestamp;
		}
	}
}

function getGamepad(): Gamepad | null {
	const gamepads = navigator.getGamepads();

	// Try stored index first
	if (_gamepadIndex !== null) {
		const gp = gamepads[_gamepadIndex];
		if (gp && gp.connected) return gp;
		_gamepadIndex = null;
	}

	// Try event reference
	if (_gamepadRef && _gamepadRef.connected) {
		_gamepadIndex = _gamepadRef.index;
		const fresh = gamepads[_gamepadRef.index];
		if (fresh && fresh.connected) return fresh;
		return _gamepadRef;
	}

	// Scan all slots
	for (let i = 0; i < gamepads.length; i++) {
		const gp = gamepads[i];
		if (gp && gp.connected) {
			_gamepadIndex = gp.index;
			if (_connectedGamepad !== gp.id) {
				_connectedGamepad = gp.id;
				syncReactive();
			}
			return gp;
		}
	}

	if (_connectedGamepad) {
		_connectedGamepad = null;
		_gamepadIndex = null;
		_gamepadRef = null;
		syncReactive();
	}
	return null;
}

function pollGamepads(timestamp: number) {
	const gp = getGamepad();

	if (!gp) {
		animFrameId = requestAnimationFrame(pollGamepads);
		return;
	}

	// Only process inputs when enabled
	if (!_enabled) {
		prevButtons = [];
		animFrameId = requestAnimationFrame(pollGamepads);
		return;
	}

	// Virtual keyboard mode — handle inputs directly
	if (keyboardOpen) {
		for (let i = 0; i < gp.buttons.length; i++) {
			const pressed = isButtonPressed(gp.buttons[i]);
			const was = prevButtons[i] ?? false;
			if (pressed && !was) {
				window.dispatchEvent(new CustomEvent('gamepad-kb', { detail: { button: i } }));
			}
			if (pressed && was && [12, 13, 14, 15].includes(i)) {
				handleRepeat(`kb${i}`, timestamp, () =>
					window.dispatchEvent(new CustomEvent('gamepad-kb', { detail: { button: i } }))
				);
			}
			if (!pressed && was) repeatTimers.delete(`kb${i}`);
		}
		// Left stick
		const lx = gp.axes[0] ?? 0;
		const ly = gp.axes[1] ?? 0;
		let dir: number | null = null;
		if (Math.abs(lx) > 0.4 || Math.abs(ly) > 0.4) {
			dir = Math.abs(lx) > Math.abs(ly) ? (lx > 0 ? 15 : 14) : ly > 0 ? 13 : 12;
		}
		if (dir !== null) {
			handleRepeat(`kbstick`, timestamp, () =>
				window.dispatchEvent(new CustomEvent('gamepad-kb', { detail: { button: dir } }))
			);
		} else {
			repeatTimers.delete('kbstick');
		}
		prevButtons = gp.buttons.map((b) => isButtonPressed(b));
		animFrameId = requestAnimationFrame(pollGamepads);
		return;
	}

	// Process buttons
	for (let i = 0; i < gp.buttons.length; i++) {
		const pressed = isButtonPressed(gp.buttons[i]);
		const wasPressed = prevButtons[i] ?? false;

		if (pressed && !wasPressed) {
			// Button just pressed
			pressButton(i);
			if (REPEATABLE_BUTTONS.includes(i)) {
				repeatTimers.set(`btn${i}`, { last: timestamp, started: timestamp });
			}
		} else if (pressed && wasPressed && REPEATABLE_BUTTONS.includes(i)) {
			handleRepeat(`btn${i}`, timestamp, () => pressButton(i));
		} else if (!pressed && wasPressed) {
			repeatTimers.delete(`btn${i}`);
		}
	}

	prevButtons = gp.buttons.map((b) => isButtonPressed(b));

	// Track multi-select button held state for chain multi-select
	multiSelectHeld = gp.buttons[BTN.Y] ? isButtonPressed(gp.buttons[BTN.Y]) : false;

	// Left stick navigation (axes 0/1) — primary navigation method
	const lx = gp.axes[0] ?? 0;
	const ly = gp.axes[1] ?? 0;
	let stickDir: Direction | null = null;

	if (Math.abs(lx) > DEADZONE || Math.abs(ly) > DEADZONE) {
		// Pick the dominant axis
		if (Math.abs(lx) > Math.abs(ly)) {
			stickDir = lx > 0 ? 'right' : 'left';
		} else {
			stickDir = ly > 0 ? 'down' : 'up';
		}
	}

	// D-pad axes fallback (some controllers map D-pad to axes 6/7 or 9)
	// Only trigger on edge (not hold)
	if (!stickDir && gp.axes.length > 7) {
		const dpadX = gp.axes[6] ?? 0;
		const dpadY = gp.axes[7] ?? 0;
		if (dpadX > 0.5 && prevStickDir !== 'right') stickDir = 'right';
		else if (dpadX < -0.5 && prevStickDir !== 'left') stickDir = 'left';
		else if (dpadY > 0.5 && prevStickDir !== 'down') stickDir = 'down';
		else if (dpadY < -0.5 && prevStickDir !== 'up') stickDir = 'up';
	}

	if (stickDir && stickDir !== prevStickDir) {
		// New direction — navigate immediately
		navigate(stickDir);
		stickRepeatTimer = { last: timestamp, started: timestamp };
	} else if (stickDir && stickDir === prevStickDir && stickRepeatTimer) {
		// Held — repeat
		const elapsed = timestamp - stickRepeatTimer.started;
		const since = timestamp - stickRepeatTimer.last;
		const rate = elapsed > 600 ? REPEAT_RATE : REPEAT_DELAY;
		if (since >= rate) {
			navigate(stickDir);
			stickRepeatTimer.last = timestamp;
		}
	} else if (!stickDir) {
		stickRepeatTimer = null;
	}
	prevStickDir = stickDir;

	// Set initial focus to content when gamepad first activates
	if (!_initialFocusSet) {
		_initialFocusSet = true;
		const elements = getFocusableElements();
		// Find the first element inside main content area (skip sidebar/titlebar)
		const contentEl = elements.find((el) => {
			const rect = el.getBoundingClientRect();
			return rect.left > 50 && rect.top > 40; // skip sidebar and titlebar
		});
		if (contentEl) {
			focusElement(contentEl);
		}
	}

	// Track route changes and restore focus
	const currentRoute = window.location.pathname;
	if (lastRoute && lastRoute !== currentRoute) {
		restoreFocusForRoute(currentRoute);
	}
	lastRoute = currentRoute;

	// Right stick scroll (axes 3 on most, 5 on some Linux controllers)
	let ry = 0;
	for (const idx of [3, 5, 4]) {
		const val = gp.axes[idx] ?? 0;
		if (Math.abs(val) > SCROLL_DEADZONE) {
			ry = val;
			break;
		}
	}
	const scrollContainer = getScrollContainer();

	if (scrollContainer && Math.abs(ry) > SCROLL_DEADZONE) {
		const scrollSpeed = Math.sign(ry) * Math.pow(Math.abs(ry), 1.5) * 14;
		scrollContainer.scrollTop += scrollSpeed;
	}

	animFrameId = requestAnimationFrame(pollGamepads);
}

function onGamepadConnected(e: GamepadEvent) {
	_connectedGamepad = e.gamepad.id;
	_gamepadIndex = e.gamepad.index;
	_gamepadRef = e.gamepad;
	syncReactive();
	if (_enabled && !animFrameId) {
		animFrameId = requestAnimationFrame(pollGamepads);
	}
}

function onGamepadDisconnected(_e: GamepadEvent) {
	_connectedGamepad = null;
	_gamepadIndex = null;
	_gamepadRef = null;
	prevButtons = [];
	repeatTimers.clear();
	syncReactive();
}

function startPolling() {
	if (animFrameId) return;
	// Check if already connected
	const gamepads = navigator.getGamepads();
	for (let i = 0; i < gamepads.length; i++) {
		const gp = gamepads[i];
		if (gp && gp.connected) {
			_connectedGamepad = gp.id;
			_gamepadIndex = gp.index;
			syncReactive();
			break;
		}
	}
	animFrameId = requestAnimationFrame(pollGamepads);
}

function stopPolling() {
	if (animFrameId) {
		cancelAnimationFrame(animFrameId);
		animFrameId = null;
	}
	prevButtons = [];
	multiSelectHeld = false;
	repeatTimers.clear();
}

export function initGamepad() {
	// Remove any stale listeners (HMR safety)
	window.removeEventListener('gamepadconnected', onGamepadConnected);
	window.removeEventListener('gamepaddisconnected', onGamepadDisconnected);

	window.addEventListener('gamepadconnected', onGamepadConnected);
	window.addEventListener('gamepaddisconnected', onGamepadDisconnected);

	// Track initial route
	lastRoute = window.location.pathname;

	// Check for already-connected gamepads
	const gamepads = navigator.getGamepads();
	for (let i = 0; i < gamepads.length; i++) {
		const gp = gamepads[i];
		if (gp && gp.connected) {
			_connectedGamepad = gp.id;
			_gamepadIndex = gp.index;
			syncReactive();
			break;
		}
	}

	if (_enabled) {
		startPolling();
	}
}

export function destroyGamepad() {
	window.removeEventListener('gamepadconnected', onGamepadConnected);
	window.removeEventListener('gamepaddisconnected', onGamepadDisconnected);
	stopPolling();
}

export function setGamepadEnabled(value: boolean) {
	_enabled = value;
	if (value) {
		_initialFocusSet = false;
		prevButtons = [];
		const gamepads = navigator.getGamepads();
		for (let i = 0; i < gamepads.length; i++) {
			const gp = gamepads[i];
			if (gp && gp.connected) {
				_connectedGamepad = gp.id;
				_gamepadIndex = gp.index;
				_gamepadRef = gp;
				break;
			}
		}
	}
	syncReactive();
	if (value) {
		startPolling();
	} else {
		stopPolling();
	}
}

export function getGamepadEnabled(): boolean {
	return _enabled;
}

export function getConnectedGamepad(): string | null {
	return _connectedGamepad;
}

export const gamepadKeyboard = {
	get open() {
		return keyboardOpen;
	},
	get value() {
		return keyboardTarget?.value ?? '';
	},
	submit(val: string) {
		if (keyboardTarget) {
			keyboardTarget.value = val;
			keyboardTarget.dispatchEvent(new Event('input', { bubbles: true }));
		}
		keyboardOpen = false;
		keyboardTarget = null;
	},
	cancel() {
		keyboardOpen = false;
		keyboardTarget = null;
	}
};

/** Reactive state for use in Svelte components */
export const gamepadState = {
	get enabled() {
		return enabled;
	},
	get connected() {
		return connectedGamepad;
	},
	get controllerType(): 'xbox' | 'playstation' | 'nintendo' | 'generic' {
		if (!connectedGamepad) return 'generic';
		const lower = connectedGamepad.toLowerCase();
		if (lower.includes('xbox') || lower.includes('xinput') || lower.includes('045e')) return 'xbox';
		if (
			lower.includes('playstation') ||
			lower.includes('dualshock') ||
			lower.includes('dualsense') ||
			lower.includes('054c')
		)
			return 'playstation';
		if (lower.includes('nintendo') || lower.includes('pro controller') || lower.includes('057e'))
			return 'nintendo';
		return 'generic';
	}
};

/**
 * Returns a human-friendly controller name from the raw gamepad ID.
 */
export function getGamepadDisplayName(rawId: string): string {
	const lower = rawId.toLowerCase();
	if (lower.includes('xbox') || lower.includes('xinput') || lower.includes('045e')) {
		return 'Xbox Controller';
	}
	if (
		lower.includes('playstation') ||
		lower.includes('dualshock') ||
		lower.includes('dualsense') ||
		lower.includes('054c')
	) {
		return 'PlayStation Controller';
	}
	if (lower.includes('nintendo') || lower.includes('pro controller') || lower.includes('057e')) {
		return 'Nintendo Controller';
	}
	// Truncate long raw IDs
	if (rawId.length > 40) {
		return rawId.slice(0, 37) + '...';
	}
	return rawId;
}
