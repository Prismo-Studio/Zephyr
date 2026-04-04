<script lang="ts">
	let { className = '' }: { className?: string } = $props();
</script>

<svg
	class={className}
	xmlns="http://www.w3.org/2000/svg"
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	stroke-width="1.5"
	stroke-linecap="round"
	stroke-linejoin="round"
	width="1em"
	height="1em"
>
	<!-- Center -->
	<circle cx="12" cy="10" r="1.5" fill="currentColor" stroke="none" />
	<!-- Petals -->
	<ellipse cx="12" cy="5.5" rx="2.2" ry="3.8" fill="currentColor" opacity="0.85" stroke="none" />
	<ellipse
		cx="16.3"
		cy="8.6"
		rx="2.2"
		ry="3.8"
		transform="rotate(72 16.3 8.6)"
		fill="currentColor"
		opacity="0.75"
		stroke="none"
	/>
	<ellipse
		cx="14.7"
		cy="13.8"
		rx="2.2"
		ry="3.8"
		transform="rotate(144 14.7 13.8)"
		fill="currentColor"
		opacity="0.7"
		stroke="none"
	/>
	<ellipse
		cx="9.3"
		cy="13.8"
		rx="2.2"
		ry="3.8"
		transform="rotate(-144 9.3 13.8)"
		fill="currentColor"
		opacity="0.7"
		stroke="none"
	/>
	<ellipse
		cx="7.7"
		cy="8.6"
		rx="2.2"
		ry="3.8"
		transform="rotate(-72 7.7 8.6)"
		fill="currentColor"
		opacity="0.75"
		stroke="none"
	/>
	<!-- Stem -->
	<path
		d="M12 14.5 Q12 18 10.5 20 Q9 22 7.5 21.5"
		stroke="currentColor"
		fill="none"
		stroke-width="1.5"
	/>
	<!-- Leaf -->
	<path
		d="M11 18 Q9 16.5 7.5 17.5 Q9.5 17 11 18Z"
		fill="currentColor"
		stroke="none"
		opacity="0.7"
	/>
</svg>
