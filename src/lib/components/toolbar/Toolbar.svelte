<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';

	import * as api from '$lib/api';
	import Icon from '@iconify/svelte';
	import GameSelect from '$lib/components/toolbar/GameSelect.svelte';
	import Updater from './Updater.svelte';
	import Syncer from './Syncer.svelte';
	import ProfilesDropdown from './ProfilesDropdown.svelte';
	import games from '$lib/state/game.svelte';
	import InstallPopover from './InstallPopover.svelte';
	import { message } from '@tauri-apps/plugin-dialog';
	import { m } from '$lib/paraglide/messages';
	import { gameIconSrc, timeSince } from '$lib/util';

	let launchDialogOpen = $state(false);
	let gamesOpen = $state(false);

	let timeSinceGamesUpdate = $derived.by(() => {
		gamesOpen; // refresh whenever the dialog is opened
		return timeSince(games.lastUpdated);
	});

	async function launchGame() {
		if (await api.profile.install.hasPendingInstallations()) {
			await message(m.toolBar_launchGame_message());
			return;
		}

		launchDialogOpen = true;
		try {
			await api.profile.launch.launchGame();
		} catch {
			launchDialogOpen = false;
		}
	}
</script>

<div class="zephyr-toolbar flex h-12 shrink-0 flex-row items-stretch">
	<!-- Launch button with gradient -->
	<button
		class="zephyr-launch-btn flex shrink-0 items-center gap-2 border-r border-[#1A2A42] px-6 font-semibold text-white transition-all duration-200"
		onclick={launchGame}
	>
		<div class="flex items-center justify-center rounded-full bg-gradient-to-r from-[#2D8CF0] to-[#00D4AA] p-1">
			<Icon icon="mdi:play" class="text-sm text-white" />
		</div>
		<span class="text-sm">{m.toolBar_launchGame_button()}</span>
	</button>

	<!-- Game selector -->
	<button
		onclick={() => (gamesOpen = !gamesOpen)}
		class="group flex shrink-0 items-center gap-2 border-r border-[#1A2A42] px-4 transition-all duration-200 hover:bg-[#142240]"
	>
		<img
			src={games.active ? gameIconSrc(games.active) : ''}
			class="max-h-7 max-w-7 rounded"
			alt={games.active?.name}
		/>
		<span class="text-sm font-medium text-[#E8ECF1]">{games.active?.name}</span>
		<Icon
			icon="mdi:chevron-down"
			class="text-[#556677] group-hover:text-[#8899AA] text-lg transition-colors"
		/>
	</button>

	<ProfilesDropdown />
	<Syncer />
	<InstallPopover />

	<div class="grow"></div>

	<Updater />
</div>

<Dialog
	title={m.toolBar_dialog_launch_title({ name: games.active?.name ?? m.unknown() })}
	bind:open={launchDialogOpen}
>
	<p class="text-[#556677]">
		{m.toolBar_dialog_launch_content()}
	</p>
</Dialog>

<Dialog title={m.toolBar_dialog_games_title()} bind:open={gamesOpen}>
	<GameSelect onselect={() => (gamesOpen = false)} />
	<div class="text-[#556677] my-1 text-center text-sm">
		{m.toolBar_dialog_games_lastUpdated({ time: timeSinceGamesUpdate })}
	</div>
</Dialog>

<style>
	.zephyr-toolbar {
		background: #0B1628;
		border-bottom: 1px solid #1A2A42;
	}

	.zephyr-launch-btn {
		background: rgba(45, 140, 240, 0.08);
	}

	.zephyr-launch-btn:hover {
		background: rgba(45, 140, 240, 0.15);
	}
</style>
