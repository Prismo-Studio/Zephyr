<script lang="ts">
	import { onMount, onDestroy } from 'svelte';

	type Props = {
		value: string;
		onchange?: (value: string) => void;
	};

	let { value = $bindable('#000000'), onchange }: Props = $props();

	const PRESETS = [
		'#1afffa',
		'#4da3ff',
		'#6e4bdc',
		'#f05fa5',
		'#fa3750',
		'#ffb347',
		'#ffd166',
		'#00d4aa',
		'#1a2438',
		'#0d1526',
		'#080e1a',
		'#ffffff'
	];

	let open = $state(false);
	let wrapper: HTMLDivElement | undefined = $state();
	let popoverStyle = $state('');

	let hue = $state(0);
	let sat = $state(100);
	let light = $state(50);

	let plateEl: HTMLDivElement | undefined = $state();
	let hueEl: HTMLDivElement | undefined = $state();
	let dragging: 'plate' | 'hue' | null = null;

	function clamp(n: number, min: number, max: number) {
		return Math.max(min, Math.min(max, n));
	}

	function hexToRgb(h: string) {
		const m = h.replace('#', '').match(/^([\da-f]{2})([\da-f]{2})([\da-f]{2})$/i);
		if (!m) return { r: 0, g: 0, b: 0 };
		return { r: parseInt(m[1], 16), g: parseInt(m[2], 16), b: parseInt(m[3], 16) };
	}

	function rgbToHex(r: number, g: number, b: number) {
		const c = (n: number) => clamp(Math.round(n), 0, 255).toString(16).padStart(2, '0');
		return `#${c(r)}${c(g)}${c(b)}`;
	}

	function rgbToHsv(r: number, g: number, b: number) {
		r /= 255;
		g /= 255;
		b /= 255;
		const max = Math.max(r, g, b);
		const min = Math.min(r, g, b);
		const d = max - min;
		let h = 0;
		const s = max === 0 ? 0 : d / max;
		const v = max;
		if (d !== 0) {
			if (max === r) h = ((g - b) / d) % 6;
			else if (max === g) h = (b - r) / d + 2;
			else h = (r - g) / d + 4;
			h = (h * 60 + 360) % 360;
		}
		return { h, s: s * 100, v: v * 100 };
	}

	function hsvToRgb(h: number, s: number, v: number) {
		s /= 100;
		v /= 100;
		const c = v * s;
		const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
		const m = v - c;
		let r = 0;
		let g = 0;
		let b = 0;
		if (h < 60) [r, g, b] = [c, x, 0];
		else if (h < 120) [r, g, b] = [x, c, 0];
		else if (h < 180) [r, g, b] = [0, c, x];
		else if (h < 240) [r, g, b] = [0, x, c];
		else if (h < 300) [r, g, b] = [x, 0, c];
		else [r, g, b] = [c, 0, x];
		return { r: (r + m) * 255, g: (g + m) * 255, b: (b + m) * 255 };
	}

	function syncFromValue() {
		const { r, g, b } = hexToRgb(value);
		const hsv = rgbToHsv(r, g, b);
		hue = hsv.h;
		sat = hsv.s;
		light = hsv.v;
	}

	function commit() {
		const { r, g, b } = hsvToRgb(hue, sat, light);
		const hex = rgbToHex(r, g, b);
		if (hex !== value) {
			value = hex;
			onchange?.(hex);
		}
	}

	function onPlateDown(e: PointerEvent) {
		dragging = 'plate';
		(e.target as Element).setPointerCapture?.(e.pointerId);
		updatePlate(e);
	}

	function onHueDown(e: PointerEvent) {
		dragging = 'hue';
		(e.target as Element).setPointerCapture?.(e.pointerId);
		updateHue(e);
	}

	function onMove(e: PointerEvent) {
		if (dragging === 'plate') updatePlate(e);
		else if (dragging === 'hue') updateHue(e);
	}

	function onUp() {
		dragging = null;
	}

	function updatePlate(e: PointerEvent) {
		if (!plateEl) return;
		const rect = plateEl.getBoundingClientRect();
		const x = clamp(e.clientX - rect.left, 0, rect.width);
		const y = clamp(e.clientY - rect.top, 0, rect.height);
		sat = (x / rect.width) * 100;
		light = 100 - (y / rect.height) * 100;
		commit();
	}

	function updateHue(e: PointerEvent) {
		if (!hueEl) return;
		const rect = hueEl.getBoundingClientRect();
		const x = clamp(e.clientX - rect.left, 0, rect.width);
		hue = (x / rect.width) * 360;
		commit();
	}

	function pickPreset(hex: string) {
		value = hex;
		syncFromValue();
		onchange?.(hex);
	}

	function onHexInput(e: Event) {
		const raw = (e.target as HTMLInputElement).value.trim();
		const normalized = raw.startsWith('#') ? raw : '#' + raw;
		if (/^#[\da-f]{6}$/i.test(normalized)) {
			value = normalized;
			syncFromValue();
			onchange?.(normalized);
		}
	}

	function toggle() {
		open = !open;
		if (open) {
			syncFromValue();
			requestAnimationFrame(positionPopover);
		}
	}

	function positionPopover() {
		if (!wrapper) return;
		const rect = wrapper.getBoundingClientRect();
		const popoverHeight = 280;
		const spaceBelow = window.innerHeight - rect.bottom;
		if (spaceBelow < popoverHeight + 20 && rect.top > popoverHeight) {
			popoverStyle = `bottom: calc(100% + 6px); right: 0;`;
		} else {
			popoverStyle = `top: calc(100% + 6px); right: 0;`;
		}
	}

	function handleClickOutside(e: MouseEvent) {
		if (wrapper && !wrapper.contains(e.target as Node)) {
			open = false;
		}
	}

	$effect(() => {
		if (open) {
			document.addEventListener('mousedown', handleClickOutside);
			window.addEventListener('pointermove', onMove);
			window.addEventListener('pointerup', onUp);
		} else {
			document.removeEventListener('mousedown', handleClickOutside);
			window.removeEventListener('pointermove', onMove);
			window.removeEventListener('pointerup', onUp);
		}
	});

	onDestroy(() => {
		document.removeEventListener('mousedown', handleClickOutside);
		window.removeEventListener('pointermove', onMove);
		window.removeEventListener('pointerup', onUp);
	});

	$effect(() => {
		if (!open) syncFromValue();
	});

	let plateBg = $derived(`hsl(${hue}, 100%, 50%)`);
</script>

<div class="z-cp" bind:this={wrapper}>
	<button
		type="button"
		class="z-cp-trigger"
		style:background={value}
		onclick={toggle}
		aria-label="Pick color"
	></button>

	{#if open}
		<div class="z-cp-pop" style={popoverStyle}>
			<div
				class="z-cp-plate"
				bind:this={plateEl}
				style:background="linear-gradient(to top, #000, transparent), linear-gradient(to right,
				#fff, {plateBg})"
				onpointerdown={onPlateDown}
			>
				<div class="z-cp-plate-marker" style:left="{sat}%" style:top="{100 - light}%"></div>
			</div>

			<div class="z-cp-hue" bind:this={hueEl} onpointerdown={onHueDown}>
				<div class="z-cp-hue-marker" style:left="{(hue / 360) * 100}%"></div>
			</div>

			<div class="z-cp-row">
				<div class="z-cp-swatch" style:background={value}></div>
				<input
					class="z-cp-hex"
					type="text"
					value={value.toUpperCase()}
					oninput={onHexInput}
					spellcheck="false"
					maxlength="7"
				/>
			</div>

			<div class="z-cp-presets">
				{#each PRESETS as p}
					<button
						type="button"
						class="z-cp-preset"
						class:selected={p.toLowerCase() === value.toLowerCase()}
						style:background={p}
						title={p}
						onclick={() => pickPreset(p)}
					></button>
				{/each}
			</div>
		</div>
	{/if}
</div>

<style>
	.z-cp {
		position: relative;
		display: inline-block;
	}

	.z-cp-trigger {
		width: 36px;
		height: 36px;
		padding: 0;
		border: 1px solid var(--border-default);
		border-radius: var(--radius-sm);
		cursor: pointer;
		transition: border-color var(--transition-fast);
	}

	.z-cp-trigger:hover {
		border-color: var(--border-strong);
	}

	.z-cp-pop {
		position: absolute;
		z-index: 500;
		width: 240px;
		padding: 12px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		box-shadow: var(--shadow-lg);
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.z-cp-plate {
		position: relative;
		width: 100%;
		height: 140px;
		border-radius: var(--radius-sm);
		cursor: crosshair;
		touch-action: none;
		overflow: hidden;
	}

	.z-cp-plate-marker {
		position: absolute;
		width: 12px;
		height: 12px;
		border: 2px solid #fff;
		border-radius: 50%;
		transform: translate(-50%, -50%);
		box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.5);
		pointer-events: none;
	}

	.z-cp-hue {
		position: relative;
		width: 100%;
		height: 12px;
		border-radius: 999px;
		background: linear-gradient(
			to right,
			#f00 0%,
			#ff0 17%,
			#0f0 33%,
			#0ff 50%,
			#00f 67%,
			#f0f 83%,
			#f00 100%
		);
		cursor: pointer;
		touch-action: none;
	}

	.z-cp-hue-marker {
		position: absolute;
		top: 50%;
		width: 14px;
		height: 14px;
		background: #fff;
		border: 2px solid rgba(0, 0, 0, 0.5);
		border-radius: 50%;
		transform: translate(-50%, -50%);
		pointer-events: none;
	}

	.z-cp-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.z-cp-swatch {
		width: 28px;
		height: 28px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-default);
		flex-shrink: 0;
	}

	.z-cp-hex {
		flex: 1;
		min-width: 0;
		padding: 6px 8px;
		font-family: var(--font-mono);
		font-size: 12px;
		text-transform: uppercase;
		background: var(--bg-surface);
		color: var(--text-primary);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-sm);
	}

	.z-cp-hex:focus {
		outline: none;
		border-color: var(--accent-400);
	}

	.z-cp-presets {
		display: grid;
		grid-template-columns: repeat(6, 1fr);
		gap: 6px;
	}

	.z-cp-preset {
		width: 100%;
		aspect-ratio: 1;
		padding: 0;
		border: 1px solid var(--border-default);
		border-radius: var(--radius-sm);
		cursor: pointer;
		transition: transform var(--transition-fast);
	}

	.z-cp-preset:hover {
		transform: scale(1.1);
	}

	.z-cp-preset.selected {
		border-color: var(--text-primary);
		box-shadow: 0 0 0 1px var(--text-primary);
	}
</style>
