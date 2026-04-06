<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	// Gamepad button mapping (Xbox / Steam Deck layout)
	const BUTTON = {
		A: 0, // Confirm / Select
		B: 1, // Back / Cancel
		X: 2, // Action 1 (Toggle mod)
		Y: 3, // Action 2 (Context menu)
		LB: 4, // Previous tab / Previous page
		RB: 5, // Next tab / Next page
		LT: 6, // Unused
		RT: 7, // Unused
		SELECT: 8, // Filters toggle
		START: 9, // Launch game
		L3: 10, // Unused
		R3: 11, // Unused
		DPAD_UP: 12,
		DPAD_DOWN: 13,
		DPAD_LEFT: 14,
		DPAD_RIGHT: 15
	};

	const NAV_PATHS = ['/dashboard', '/', '/browse', '/profiles', '/config', '/prefs'];

	let animFrame: number;
	let prevButtons: boolean[] = [];
	let prevAxes: number[] = [];
	let connected = $state(false);
	let debugInfo = $state('');
	let repeatTimer: Record<string, number> = {};
	const REPEAT_DELAY = 400;
	const REPEAT_RATE = 100;
	const AXIS_THRESHOLD = 0.5;

	function getFocusableElements(): HTMLElement[] {
		const selectors = [
			'.z-mod-card',
			'.z-nav-item',
			'.z-category-chip',
			'.z-category-tag',
			'button:not([disabled])',
			'a[href]',
			'input:not([disabled])',
			'select:not([disabled])',
			'[tabindex="0"]'
		].join(',');

		return Array.from(document.querySelectorAll<HTMLElement>(selectors)).filter(
			(el) => el.offsetParent !== null && !el.closest('[aria-hidden="true"]')
		);
	}

	function getCurrentFocusIndex(elements: HTMLElement[]): number {
		const active = document.activeElement as HTMLElement;
		return elements.indexOf(active);
	}

	function focusElement(el: HTMLElement) {
		el.focus();
		el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
	}

	function navigateDirection(direction: 'up' | 'down' | 'left' | 'right') {
		const elements = getFocusableElements();
		const currentIndex = getCurrentFocusIndex(elements);
		const current = document.activeElement as HTMLElement;

		if (direction === 'up' || direction === 'down') {
			// Vertical navigation within mod lists
			const modCards = elements.filter((el) => el.classList.contains('z-mod-card'));

			if (modCards.length > 0) {
				const cardIndex = modCards.indexOf(current);
				if (cardIndex === -1) {
					// No mod card focused, focus first one
					focusElement(modCards[0]);
					return;
				}
				if (direction === 'down') {
					const next = cardIndex < modCards.length - 1 ? cardIndex + 1 : 0;
					focusElement(modCards[next]);
				} else {
					const prev = cardIndex > 0 ? cardIndex - 1 : modCards.length - 1;
					focusElement(modCards[prev]);
				}
				return;
			}

			// Fallback: generic up/down
			if (currentIndex === -1) {
				focusElement(elements[0]);
			} else {
				const next =
					direction === 'down'
						? (currentIndex + 1) % elements.length
						: (currentIndex - 1 + elements.length) % elements.length;
				focusElement(elements[next]);
			}
		} else if (direction === 'left' || direction === 'right') {
			// Left/Right: switch between sidebar nav items
			const navItems = elements.filter((el) => el.classList.contains('z-nav-item'));
			if (navItems.length > 0 && navItems.includes(current)) {
				const navIndex = navItems.indexOf(current);
				if (direction === 'right') {
					// Move focus to main content
					const mainContent = elements.filter(
						(el) => !el.classList.contains('z-nav-item') && !el.closest('.z-sidebar')
					);
					if (mainContent.length > 0) focusElement(mainContent[0]);
				}
				return;
			}

			// Left: go back to sidebar
			if (direction === 'left') {
				const navItems2 = elements.filter((el) => el.classList.contains('z-nav-item'));
				const activeNav = navItems2.find((el) => el.classList.contains('active'));
				if (activeNav) focusElement(activeNav);
				else if (navItems2.length > 0) focusElement(navItems2[0]);
			}
		}
	}

	function handleButton(buttonIndex: number) {
		const active = document.activeElement as HTMLElement;
		const btnNames = [
			'A',
			'B',
			'X',
			'Y',
			'LB',
			'RB',
			'LT',
			'RT',
			'SEL',
			'START',
			'L3',
			'R3',
			'UP',
			'DOWN',
			'LEFT',
			'RIGHT'
		];
		debugInfo = `${btnNames[buttonIndex] ?? buttonIndex} → ${active?.tagName}.${active?.className?.split(' ')[0] ?? ''}`;

		switch (buttonIndex) {
			case BUTTON.A:
				// Confirm: click the focused element
				if (active) active.click();
				break;

			case BUTTON.B:
				// Back: close modals, deselect, or go back
				{
					const modal = document.querySelector('.z-modal-backdrop') as HTMLElement;
					if (modal) {
						modal.click();
						return;
					}
					const closeBtn = document.querySelector('.z-details-close') as HTMLElement;
					if (closeBtn) {
						closeBtn.click();
						return;
					}
					// Press Escape
					document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
				}
				break;

			case BUTTON.X:
				// Toggle mod (enable/disable)
				{
					const toggleBtn = document.querySelector('.z-action-btn') as HTMLElement;
					if (toggleBtn) toggleBtn.click();
				}
				break;

			case BUTTON.Y:
				// Context menu on focused mod card
				if (active?.classList.contains('z-mod-card')) {
					const rect = active.getBoundingClientRect();
					active.dispatchEvent(
						new MouseEvent('contextmenu', {
							bubbles: true,
							clientX: rect.x + rect.width / 2,
							clientY: rect.y + rect.height / 2
						})
					);
				}
				break;

			case BUTTON.LB:
				// Previous nav page
				{
					const navItems = Array.from(document.querySelectorAll<HTMLAnchorElement>('.z-nav-item'));
					const activeIdx = navItems.findIndex((el) => el.classList.contains('active'));
					if (activeIdx > 0) navItems[activeIdx - 1].click();
					else if (navItems.length > 0) navItems[navItems.length - 1].click();
				}
				break;

			case BUTTON.RB:
				// Next nav page
				{
					const navItems = Array.from(document.querySelectorAll<HTMLAnchorElement>('.z-nav-item'));
					const activeIdx = navItems.findIndex((el) => el.classList.contains('active'));
					if (activeIdx < navItems.length - 1) navItems[activeIdx + 1].click();
					else if (navItems.length > 0) navItems[0].click();
				}
				break;

			case BUTTON.SELECT:
				// Toggle filters
				{
					const filterBtn = document.querySelector(
						'.z-filter-toggle-btn, [class*="filter"]'
					) as HTMLElement;
					if (filterBtn) filterBtn.click();
				}
				break;

			case BUTTON.START:
				// Launch game
				{
					const launchBtn = document.querySelector('.z-launch-btn') as HTMLElement;
					if (launchBtn) launchBtn.click();
				}
				break;

			case BUTTON.DPAD_UP:
				navigateDirection('up');
				break;

			case BUTTON.DPAD_DOWN:
				navigateDirection('down');
				break;

			case BUTTON.DPAD_LEFT:
				navigateDirection('left');
				break;

			case BUTTON.DPAD_RIGHT:
				navigateDirection('right');
				break;
		}
	}

	function handleRepeat(key: string, action: () => void) {
		const now = Date.now();
		if (!repeatTimer[key]) {
			repeatTimer[key] = now;
			action();
		} else if (now - repeatTimer[key] > REPEAT_DELAY) {
			if ((now - repeatTimer[key] - REPEAT_DELAY) % REPEAT_RATE < 20) {
				action();
			}
		}
	}

	function clearRepeat(key: string) {
		delete repeatTimer[key];
	}

	function pollGamepad() {
		const gamepads = navigator.getGamepads();
		// Find first connected gamepad (not always index 0)
		let gp: Gamepad | null = null;
		for (const pad of gamepads) {
			if (pad && pad.connected) {
				gp = pad;
				break;
			}
		}

		if (!gp) {
			connected = false;
			animFrame = requestAnimationFrame(pollGamepad);
			return;
		}

		if (!connected) {
			connected = true;
			// Auto-focus first focusable element when gamepad connects
			const first = getFocusableElements()[0];
			if (first) focusElement(first);
		}

		// Show live axes/buttons for debug
		const activeButtons: string[] = [];
		const btnNames = [
			'A',
			'B',
			'X',
			'Y',
			'LB',
			'RB',
			'LT',
			'RT',
			'SEL',
			'START',
			'L3',
			'R3',
			'UP',
			'DOWN',
			'LEFT',
			'RIGHT'
		];

		// Button presses (rising edge) - use .value > 0.5 for better compatibility
		for (let i = 0; i < gp.buttons.length; i++) {
			const pressed = gp.buttons[i].pressed || gp.buttons[i].value > 0.5;
			const wasPressed = prevButtons[i] || false;

			if (pressed) activeButtons.push(btnNames[i] ?? String(i));

			// D-pad with repeat
			if (i >= BUTTON.DPAD_UP && i <= BUTTON.DPAD_RIGHT) {
				if (pressed) {
					handleRepeat(`btn_${i}`, () => handleButton(i));
				} else {
					clearRepeat(`btn_${i}`);
				}
			} else if (pressed && !wasPressed) {
				handleButton(i);
			}

			prevButtons[i] = pressed;
		}

		// Left stick navigation
		const lx = gp.axes[0] || 0;
		const ly = gp.axes[1] || 0;
		const prevLx = prevAxes[0] || 0;
		const prevLy = prevAxes[1] || 0;

		// Always show debug info when gamepad connected
		if (activeButtons.length > 0) {
			debugInfo = activeButtons.join('+');
		} else if (Math.abs(lx) > 0.2 || Math.abs(ly) > 0.2) {
			debugInfo = `L:${lx.toFixed(1)},${ly.toFixed(1)}`;
		} else {
			debugInfo = `${gp.id.substring(0, 20)} | ${gp.buttons.length}btn ${gp.axes.length}ax`;
		}

		if (ly < -AXIS_THRESHOLD) {
			handleRepeat('stick_up', () => navigateDirection('up'));
		} else {
			clearRepeat('stick_up');
		}

		if (ly > AXIS_THRESHOLD) {
			handleRepeat('stick_down', () => navigateDirection('down'));
		} else {
			clearRepeat('stick_down');
		}

		if (lx < -AXIS_THRESHOLD && prevLx >= -AXIS_THRESHOLD) {
			navigateDirection('left');
		}
		if (lx > AXIS_THRESHOLD && prevLx >= AXIS_THRESHOLD) {
			// Only trigger once when crossing threshold
		}
		if (lx > AXIS_THRESHOLD && prevLx <= AXIS_THRESHOLD) {
			navigateDirection('right');
		}

		prevAxes = [...gp.axes];

		animFrame = requestAnimationFrame(pollGamepad);
	}

	onMount(() => {
		window.addEventListener('gamepadconnected', (e) => {
			connected = true;
			console.log(`Gamepad connected: ${e.gamepad.id}`);
		});

		window.addEventListener('gamepaddisconnected', () => {
			connected = false;
			console.log('Gamepad disconnected');
		});

		animFrame = requestAnimationFrame(pollGamepad);
	});

	onDestroy(() => {
		if (animFrame) cancelAnimationFrame(animFrame);
	});
</script>

<style>
	.z-gamepad-indicator {
		position: fixed;
		bottom: 4px;
		left: 74px;
		display: flex;
		align-items: center;
		gap: 6px;
		color: var(--text-accent);
		opacity: 0.8;
		z-index: 9999;
		pointer-events: none;
		font-size: 11px;
		font-family: var(--font-mono);
	}

	.z-gamepad-debug {
		background: rgba(0, 0, 0, 0.6);
		padding: 2px 6px;
		border-radius: 4px;
		color: var(--text-accent);
	}
</style>
