import type { SourceId } from '$lib/api/sources';
import { PersistedState } from 'runed';

export const activeSourceState = new PersistedState<SourceId>('activeSource', 'thunderstore');
