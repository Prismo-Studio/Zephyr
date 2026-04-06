import { paraglideVitePlugin } from '@inlang/paraglide-js';
import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, type Plugin } from 'vite';

/**
 * Workaround for @tailwindcss/vite receiving raw .svelte content
 * instead of extracted CSS for style virtual modules.
 * Extracts just the <style> block so Tailwind can parse it.
 */
function svelteStyleFix(): Plugin {
	return {
		name: 'svelte-style-fix',
		enforce: 'pre',
		transform(code, id) {
			if (id.includes('lang.css') && id.includes('.svelte') && code.trimStart().startsWith('<')) {
				const styleMatch = code.match(/<style[^>]*>([\s\S]*?)<\/style>/);
				return styleMatch ? styleMatch[1] : '';
			}
		}
	};
}

export default defineConfig({
	plugins: [
		paraglideVitePlugin({ project: './project.inlang', outdir: './src/lib/paraglide' }),
		svelteStyleFix(),
		tailwindcss(),
		sveltekit()
	],
	build: {
		chunkSizeWarningLimit: 1000
	},
	test: {
		include: ['src/**/*.test.ts'],
		environment: 'node'
	}
});
