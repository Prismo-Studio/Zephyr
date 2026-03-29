<script lang="ts">
	import { setConfigEntry } from '$lib/config';
	import type { ConfigValue, ConfigNum, ConfigEntryId, ConfigRange } from '$lib/types';
	import ResetConfigButton from './ResetConfigButton.svelte';

	type Props = {
		entryId: ConfigEntryId;
		locked: boolean;
	};

	let { entryId, locked }: Props = $props();

	let content = $state(entryId.entry.value.content as ConfigNum);
	let range = $derived(content.range as ConfigRange);
	let type = entryId.entry.value.type as 'int' | 'float';

	let element: HTMLDivElement;

	let isDragging = $state(false);
	// svelte-ignore state_referenced_locally
	let inputString = $state(content.value.toString());

	function onReset(newValue: ConfigValue) {
		content = newValue.content as ConfigNum;
		inputString = content.value.toString();
	}

	function submitValue() {
		setConfigEntry(entryId, { type, content });
	}

	const DECIMAL_STEP = 0.1;
	const X_OFFSET = 16;

	function calculateNewValue(clientX: number) {
		let rect = element.getBoundingClientRect();
		rect.width -= X_OFFSET;
		rect.x += X_OFFSET / 2;

		let x = clientX - rect.left;
		let newValue = range.start + (range.end - range.start) * (x / rect.width);

		if (type === 'float') {
			newValue = Math.round(newValue / DECIMAL_STEP) * DECIMAL_STEP;
		} else if (type === 'int') {
			newValue = Math.round(newValue);
		}

		newValue = clamp(newValue, range.start, range.end);
		inputString = newValue.toFixed(decimals);
		content.value = newValue;
	}

	function clamp(value: number, min: number, max: number) {
		return Math.max(min, Math.min(max, value));
	}

	let fillPercent = $derived(
		clamp(((content.value - range.start) / (range.end - range.start)) * 100, 0, 100)
	);

	let decimals = $derived(type === 'int' ? 0 : 1);
</script>

<svelte:window
	onmousemove={(evt) => {
		if (isDragging) {
			calculateNewValue(evt.clientX);
		}
	}}
	onmouseup={(evt) => {
		if (isDragging) {
			isDragging = false;
			calculateNewValue(evt.clientX);
			submitValue();
		}
	}}
/>

<div
	class="group bg-[#0B1628] h-5 grow rounded-full py-1 pr-2 pl-1"
	role="slider"
	aria-valuemin={range.start}
	aria-valuemax={range.end}
	aria-valuenow={content.value}
	aria-disabled={locked}
	tabindex="0"
	bind:this={element}
	onkeydown={(e) => {
		if (locked) return;

		if (e.key === 'ArrowLeft' || e.key === 'ArrowDown') {
			content.value = Math.max(range.start, content.value - 1);
		} else if (e.key === 'ArrowRight' || e.key === 'ArrowUp') {
			content.value = Math.min(range.end, content.value + 1);
		}

		inputString = content.value.toFixed(decimals);
	}}
	onmousedown={(evt) => {
		if (locked) return;

		isDragging = true;
		calculateNewValue(evt.clientX);
	}}
>
	<div
		class={[!locked && 'group-hover:bg-[#1A2A42]', 'relative h-full min-w-1 rounded-l-full']}
		style="width: {fillPercent}%;"
		class:bg-[#142240]={!isDragging}
		class:bg-[#1A2A42]={isDragging}
	>
		<div
			class="absolute right-[-0.5rem] h-3 w-3 rounded-full"
			class:bg-[#556677]={!isDragging}
			class:bg-[#8899AA]={isDragging}
			draggable="false"
		></div>
	</div>
</div>

<input
	type="number"
	disabled={locked}
	bind:value={inputString}
	onchange={() => {
		if (locked) return;
		let newValue = parseFloat(inputString);

		if (!isNaN(newValue)) {
			newValue = clamp(newValue, range.start, range.end);

			if (type === 'int') {
				newValue = Math.round(newValue);
			}

			content.value = newValue;
			submitValue();
		}

		inputString = content.value.toString();
	}}
	class="focus:ring-[#00D4AA] bg-[#0B1628] text-[#8899AA] placeholder-[#556677] enabled:hover:ring-[#1A2A42] disabled:text-[#556677] ml-3 w-1/6 min-w-0 shrink rounded-lg px-3 py-1 focus:ring-2 focus:outline-hidden enabled:hover:ring-1"
/>

<ResetConfigButton {entryId} {locked} {onReset} />

<style>
	input[type='number']::-webkit-inner-spin-button,
	input[type='number']::-webkit-outer-spin-button {
		-webkit-appearance: none;
		margin: 0;
	}
</style>
