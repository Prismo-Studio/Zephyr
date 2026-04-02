import { quadOut } from 'svelte/easing';
import {
	TRANSITION_DROP_IN_DURATION_MS,
	TRANSITION_DROP_OUT_DURATION_MS
} from './constants/ui.constants';

export const dropIn = dropInTo({ y: -5 });
export const dropOut = dropOutFrom({ y: -5 });

export function dropInTo({ x, y }: { x?: number; y?: number }) {
	return { duration: TRANSITION_DROP_IN_DURATION_MS, easing: quadOut, x, y };
}

export function dropOutFrom({}: { x?: number; y?: number }) {
	return { duration: TRANSITION_DROP_OUT_DURATION_MS };
}
