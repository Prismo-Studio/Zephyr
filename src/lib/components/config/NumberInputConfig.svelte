<script lang="ts">
	import { setConfigEntry } from '$lib/config';
	import type { ConfigEntryId, ConfigNum, ConfigValue } from '$lib/types';
	import ResetConfigButton from './ResetConfigButton.svelte';

	type Props = {
		entryId: ConfigEntryId;
		locked: boolean;
	};

	let { entryId, locked }: Props = $props();

	let value = entryId.entry.value;
	let content = $state(value.content as ConfigNum);
	let type = value.type as 'int' | 'float';

	function onReset(value: ConfigValue) {
		content = value.content as ConfigNum;
	}

	function submit() {
		setConfigEntry(entryId, { type, content });
	}
</script>

<input
	type="number"
	disabled={locked}
	step={type === 'int' ? 1 : 'any'}
	bind:value={content.value}
	onchange={submit}
	class="focus:ring-[#00D4AA]! bg-[#0B1628] text-[#8899AA] placeholder-[#556677] enabled:hover:ring-[#1A2A42] disabled:text-[#556677] w-full grow rounded-lg px-3 py-1 focus:ring-2! focus:outline-hidden enabled:hover:ring-1"
/>

<ResetConfigButton {entryId} {onReset} {locked} />

<style>
	input::-webkit-inner-spin-button,
	input::-webkit-outer-spin-button {
		-webkit-appearance: none;
		margin: 0;
	}
</style>
