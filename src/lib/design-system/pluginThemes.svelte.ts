type PluginTheme = { id: string; name: string };

const STYLE_PREFIX = 'zephyr-plugin-theme-';

class PluginThemesState {
	list: PluginTheme[] = $state([]);

	register(theme: PluginTheme, css: string) {
		if (typeof document === 'undefined') return;
		const elementId = `${STYLE_PREFIX}${theme.id}`;
		let style = document.getElementById(elementId) as HTMLStyleElement | null;
		if (!style) {
			style = document.createElement('style');
			style.id = elementId;
			document.head.appendChild(style);
		}
		style.textContent = css;
		if (!this.list.some((t) => t.id === theme.id)) {
			this.list = [...this.list, theme];
		}
	}

	unregister(id: string) {
		if (typeof document !== 'undefined') {
			document.getElementById(`${STYLE_PREFIX}${id}`)?.remove();
		}
		this.list = this.list.filter((t) => t.id !== id);
	}

	has(id: string): boolean {
		return this.list.some((t) => t.id === id);
	}
}

const pluginThemes = new PluginThemesState();
export default pluginThemes;
