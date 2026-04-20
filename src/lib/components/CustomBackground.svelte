<script lang="ts">
	import { customBgState } from '$lib/design-system/customBg.svelte';
</script>

{#if customBgState.media && customBgState.themeActive}
	<div class="z-custom-bg">
		{#if customBgState.media.kind === 'video'}
			<video
				class="z-custom-bg-media"
				src={customBgState.media.url}
				autoplay
				loop
				muted
				playsinline
				disablepictureinpicture
			></video>
		{:else}
			<img class="z-custom-bg-media" src={customBgState.media.url} alt="" />
		{/if}
	</div>
{/if}

<style>
	.z-custom-bg {
		position: fixed;
		inset: 0;
		width: 100vw;
		height: 100vh;
		z-index: 0;
		pointer-events: none;
		overflow: hidden;
		background: #080e1a;
	}

	.z-custom-bg-media {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}
</style>
