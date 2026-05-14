export type PluginType = 'feature' | 'theme' | 'game' | 'mod';

export type Plugin = {
	id: string;
	name: string;
	description: string;
	author: string;
	version: string;
	icon: string;
	kind: PluginType;
	builtIn: boolean;
	removable: boolean;
	enabled: boolean;
	dev?: boolean;
	devPath?: string;
	sidebarLabel?: string;
	sidebarIcon?: string;
};
