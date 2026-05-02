import type { Mod, MarkdownType } from '$lib/types';
import * as api from '$lib/api';
import { getMarkdown } from '$lib/utils/mod';

/**
 * Resolves readme/changelog markdown for a mod, regardless of source.
 *
 * Different mod sources need different fetch paths:
 * - CurseForge: backend HTTP via `sources` API
 * - GitHub (zephyr): backend HTTP via `sources` API
 * - Other external (no slug recognized): use the mod's own description as fallback
 * - Native (Thunderstore + local): the standard `getMarkdown` helper
 */
export async function loadModMarkdown(mod: Mod, type: MarkdownType): Promise<string> {
	try {
		if (mod.uuid.startsWith('curseforge:')) {
			const cfId = mod.uuid.replace('curseforge:', '');
			if (type === 'readme') {
				const desc = await api.sources.getSourceModDescription('curseforge', cfId);
				return desc ?? mod.description ?? '';
			}
			if (type === 'changelog' && mod.versions.length > 0) {
				const fileId = mod.versions[0].uuid;
				const cl = await api.sources.getSourceModChangelog('curseforge', cfId, fileId);
				return cl ?? '';
			}
			return '';
		}

		if (mod.uuid.startsWith('zephyr:')) {
			const slug = mod.uuid.replace('zephyr:', '');
			if (type === 'readme') {
				const desc = await api.sources.getSourceModDescription('github', slug);
				return desc ?? mod.description ?? '';
			}
			const cl = await api.sources.getSourceModChangelog('github', slug, '');
			return cl ?? '';
		}

		if (mod.uuid.includes(':')) {
			return type === 'readme' ? (mod.description ?? '') : '';
		}

		const result = await getMarkdown(mod, type);
		return result ?? '';
	} catch {
		return '';
	}
}
