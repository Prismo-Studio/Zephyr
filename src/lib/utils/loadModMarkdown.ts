import type { Mod, MarkdownType } from '$lib/types';
import * as api from '$lib/api';
import { getMarkdown } from '$lib/utils/mod';
import { marked } from 'marked';

async function md(raw: string | null | undefined): Promise<string> {
	if (!raw) return '';
	return await marked(raw);
}

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

		if (mod.uuid.startsWith('zephyrmods:')) {
			const slug = mod.uuid.replace('zephyrmods:', '');
			if (type === 'readme') {
				const desc = await api.sources.getSourceModDescription('zephyrmods', slug);
				return await md(desc ?? mod.description ?? '');
			}
			const cl = await api.sources.getSourceModChangelog('zephyrmods', slug, '');
			return await md(cl ?? '');
		}

		if (mod.source === 'zephyrmods' && mod.externalId) {
			if (type === 'readme') {
				const desc = await api.sources.getSourceModDescription('zephyrmods', mod.externalId);
				return await md(desc ?? mod.description ?? '');
			}
			const cl = await api.sources.getSourceModChangelog('zephyrmods', mod.externalId, '');
			return await md(cl ?? '');
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
