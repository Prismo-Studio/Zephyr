import type { Mod } from './mod.types';

export type ContextItem = {
	label: string;
	icon?: string;
	onclick: () => void;
	children?: ContextItem[];
};

export type ModContextItem = {
	label: string;
	icon?: string;
	showFor?: (mod: Mod, locked: boolean) => boolean;
	onclick: (mod: Mod) => void;
	children?: (mod: Mod) => ModContextItem[];
};
