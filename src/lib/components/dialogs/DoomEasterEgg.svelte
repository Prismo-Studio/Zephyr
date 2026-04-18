<script lang="ts">
	import { onMount, onDestroy } from 'svelte';

	let open = $state(false);
	let phase = $state<'idle' | 'glitch' | 'open'>('idle');
	let comment = $state('');

	const QUOTES = [
		'// what the fuck?',
		'// evil floating point bit level hacking',
		'// 1st iteration',
		'// 2nd iteration, this can be removed'
	];

	function trigger() {
		if (open) return;
		comment = QUOTES[Math.floor(Math.random() * QUOTES.length)] ?? '// what the fuck?';
		phase = 'glitch';
		setTimeout(() => {
			phase = 'open';
			open = true;
		}, 1100);
	}

	function close() {
		open = false;
		phase = 'idle';
	}

	function onKey(e: KeyboardEvent) {
		if (open && e.key === 'Escape') close();
	}

	onMount(() => {
		window.addEventListener('doom-unlocked', trigger);
		window.addEventListener('keydown', onKey);
	});

	onDestroy(() => {
		window.removeEventListener('doom-unlocked', trigger);
		window.removeEventListener('keydown', onKey);
	});
</script>

{#if phase === 'glitch'}
	<div class="z-doom-flash">
		<pre class="z-doom-pre">float Q_rsqrt( float number ) {'{'}
	long i;
	float x2, y;
	const float threehalfs = 1.5F;

	x2 = number * 0.5F;
	y  = number;
	i  = * ( long * ) &amp;y;
	i  = 0x5f3759df - ( i &gt;&gt; 1 );          {comment}
	y  = * ( float * ) &amp;i;
	y  = y * ( threehalfs - ( x2 * y * y ) );

	return y;
{'}'}</pre>
	</div>
{/if}

{#if open}
	<div
		class="z-doom-backdrop"
		role="button"
		tabindex="0"
		onclick={close}
		onkeydown={(e) => e.key === 'Enter' && close()}
	>
		<div
			class="z-doom-frame"
			role="dialog"
			aria-label="DOOM"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
		>
			<div class="z-doom-titlebar">
				<span class="z-doom-title">RIP AND TEAR</span>
				<button class="z-doom-close" onclick={close} type="button" aria-label="Close">×</button>
			</div>
			<iframe
				src="https://archive.org/embed/DoomsharewareEpisode"
				title="DOOM"
				allow="autoplay; fullscreen; gamepad"
				referrerpolicy="no-referrer"
			></iframe>
		</div>
	</div>
{/if}

<style>
	.z-doom-flash {
		position: fixed;
		inset: 0;
		background: #000;
		z-index: 9998;
		display: flex;
		align-items: center;
		justify-content: center;
		font-family: 'JetBrains Mono', monospace;
		color: #1afffa;
		animation: doom-shake 0.12s linear infinite;
	}

	.z-doom-pre {
		font-size: 13px;
		line-height: 1.55;
		color: #1afffa;
		text-shadow: 0 0 6px #1afffa, 0 0 12px rgba(26, 255, 250, 0.6);
		white-space: pre;
		opacity: 0;
		animation: doom-fade 1.1s ease forwards;
		max-width: 92vw;
		overflow: hidden;
	}

	@keyframes doom-fade {
		0%   { opacity: 0; }
		20%  { opacity: 1; }
		70%  { opacity: 1; }
		100% { opacity: 0; }
	}

	@keyframes doom-shake {
		0%, 100% { transform: translate(0, 0); }
		25% { transform: translate(-2px, 1px); }
		50% { transform: translate(2px, -1px); }
		75% { transform: translate(-1px, -2px); }
	}

	.z-doom-backdrop {
		position: fixed;
		inset: 0;
		background: #000;
		z-index: 9999;
		display: flex;
		flex-direction: column;
		animation: doom-bg 0.3s ease;
		border: none;
	}

	@keyframes doom-bg {
		from { opacity: 0; }
		to { opacity: 1; }
	}

	.z-doom-frame {
		position: relative;
		width: 100%;
		height: 100%;
		background: #000;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.z-doom-titlebar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 6px 12px;
		background: linear-gradient(180deg, #2a0000, #100000);
		border-bottom: 1px solid #b22;
		font-family: 'JetBrains Mono', monospace;
		flex-shrink: 0;
	}

	.z-doom-title {
		font-size: 12px;
		font-weight: 700;
		letter-spacing: 0.15em;
		color: #ff4040;
		text-shadow: 0 0 8px rgba(255, 64, 64, 0.6);
	}

	.z-doom-close {
		width: 24px;
		height: 24px;
		background: transparent;
		border: 1px solid #b22;
		border-radius: 4px;
		color: #ff8888;
		font-size: 18px;
		line-height: 1;
		cursor: pointer;
		font-family: inherit;
	}

	.z-doom-close:hover {
		background: #b22;
		color: #fff;
	}

	.z-doom-frame iframe {
		flex: 1;
		width: 100%;
		border: none;
		background: #000;
	}
</style>
