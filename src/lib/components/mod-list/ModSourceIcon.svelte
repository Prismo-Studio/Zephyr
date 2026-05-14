<script lang="ts">
	type Props = {
		uuid: string;
		icon?: string | null;
		extraClass?: string;
	};

	let { uuid, icon, extraClass = '' }: Props = $props();

	let kind = $derived.by(() => {
		if (uuid.startsWith('curseforge:') || uuid.startsWith('zephyr-server:')) {
			return { src: '/logos/curseforge.png', alt: 'CF' };
		}
		if (uuid.startsWith('nexusmods:')) {
			return { src: '/logos/nexusmods.png', alt: 'NX' };
		}
		if (uuid.startsWith('zephyrmods:')) {
			return { src: '/logo.png', alt: 'Zephyr Mods' };
		}
		if (icon && icon.includes('Prismo-Studio/zephyr-mods')) {
			return { src: '/logo.png', alt: 'Zephyr Mods' };
		}
		return null;
	});
</script>

{#if kind}
	<img src={kind.src} alt={kind.alt} class={extraClass} />
{/if}
