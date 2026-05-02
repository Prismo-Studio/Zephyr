<script lang="ts">
	type Props = {
		uuid: string;
		extraClass?: string;
	};

	let { uuid, extraClass = '' }: Props = $props();

	let kind = $derived.by(() => {
		if (uuid.startsWith('curseforge:') || uuid.startsWith('zephyr-server:')) {
			return { src: '/logos/curseforge.png', alt: 'CF' };
		}
		if (uuid.startsWith('nexusmods:')) {
			return { src: '/logos/nexusmods.png', alt: 'NX' };
		}
		if (uuid.startsWith('zephyr:')) {
			return { src: '/logo.png', alt: 'Z' };
		}
		return null;
	});
</script>

{#if kind}
	<img src={kind.src} alt={kind.alt} class={extraClass} />
{/if}
