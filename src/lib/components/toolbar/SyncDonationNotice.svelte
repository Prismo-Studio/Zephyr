<script lang="ts">
	import { PersistedState } from 'runed';
	import Link from '../ui/Link.svelte';
	import Icon from '@iconify/svelte';
	import InfoBox from '../ui/InfoBox.svelte';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		show?: boolean;
	};

	let { show: showProp = true }: Props = $props();

	const closeDuration = 1000 * 60 * 60 * 24 * 7; // 1 week

	let closedAt = new PersistedState<string | null>('donationClosedAt', null);

	let show = $derived(
		showProp &&
			(!closedAt.current || Date.now() - new Date(closedAt.current).getTime() > closeDuration)
	);
</script>

<InfoBox class={!show && 'hidden'}>
	<div class="text-lg font-semibold text-white">{m.syncDonationNotice_content_1()}</div>

	<div class="text-[#8899AA]">
		{m.syncDonationNotice_content_2()}<Link href="https://ko-fi.com/kesomannen">Kofi</Link>

		<Icon class="mb-1 inline" icon="mdi:heart" />.
	</div>

	<button
		class="text-[#556677] hover:text-[#2D8CF0] mt-2 flex items-center gap-1 text-sm hover:underline"
		onclick={() => {
			closedAt.current = new Date().toISOString();
		}}
	>
		<Icon icon="mdi:close" />
		{m.syncDonationNotice_button()}
	</button>
</InfoBox>
