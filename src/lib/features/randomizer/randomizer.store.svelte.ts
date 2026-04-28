import * as api from './api';
import type {
	Condition,
	Dependency,
	GameSchema,
	GameSummary,
	LintIssue,
	OptionDef,
	RandomizerConfig,
	Value,
	ValidationError
} from './types';

function defaultFor(opt: OptionDef): Value {
	switch (opt.type.kind) {
		case 'toggle':
			return opt.type.default;
		case 'range':
			return opt.type.default;
		case 'select':
			return opt.type.default;
		case 'multi_select':
			return [...opt.type.defaults];
		case 'text':
			return opt.type.default;
	}
}

function valuesEqual(a: Value, b: Value): boolean {
	if (Array.isArray(a) && Array.isArray(b)) {
		return a.length === b.length && a.every((v, i) => valuesEqual(v, b[i]));
	}
	return a === b;
}

function matchesCondition(actual: Value | undefined, condition: Condition): boolean {
	if (actual === undefined) return false;
	switch (condition.op) {
		case 'equals':
			return valuesEqual(actual, condition.value);
		case 'not_equals':
			return !valuesEqual(actual, condition.value);
		case 'in':
			return condition.values.some((v) => valuesEqual(actual, v));
	}
}

export function dependenciesSatisfied(
	deps: Dependency[] | undefined,
	values: Record<string, Value>
): boolean {
	if (!deps || deps.length === 0) return true;
	return deps.every((d) => matchesCondition(values[d.option_id], d.condition));
}

class RandomizerStore {
	catalog: GameSummary[] = $state([]);
	catalogLoading = $state(false);

	currentSchema: GameSchema | null = $state(null);
	loadingSchema = $state(false);

	values: Record<string, Value> = $state({});
	presetId: string | undefined = $state(undefined);
	seed: string = $state('');
	playerName: string = $state('Player1');

	showAdvanced = $state(false);

	generatedYaml = $state('');
	validationErrors: ValidationError[] = $state([]);
	lintIssues: LintIssue[] = $state([]);

	/** Most recently changed option id, used to highlight & describe the change. */
	lastChangedId: string | null = $state(null);
	/** Dependent option ids that just became visible/hidden because of the last change. */
	lastImpact: { newlyVisible: string[]; newlyHidden: string[] } = $state({
		newlyVisible: [],
		newlyHidden: []
	});

	get config(): RandomizerConfig | null {
		if (!this.currentSchema) return null;
		return {
			game_id: this.currentSchema.id,
			seed: this.seed || undefined,
			values: { ...this.values },
			preset_id: this.presetId,
			player_name: this.playerName || undefined
		};
	}

	async loadCatalog() {
		this.catalogLoading = true;
		try {
			this.catalog = await api.listSupportedGames();
		} finally {
			this.catalogLoading = false;
		}
	}

	async selectGame(gameId: string) {
		this.loadingSchema = true;
		try {
			const schema = await api.getGameSchema(gameId);
			this.currentSchema = schema;
			this.resetToDefaults();
		} finally {
			this.loadingSchema = false;
		}
	}

	async reloadCurrentSchema() {
		const id = this.currentSchema?.id;
		if (!id) return;
		this.loadingSchema = true;
		try {
			this.currentSchema = await api.getGameSchema(id);
		} finally {
			this.loadingSchema = false;
		}
	}

	clearGame() {
		this.currentSchema = null;
		this.values = {};
		this.presetId = undefined;
		this.generatedYaml = '';
		this.validationErrors = [];
		this.lintIssues = [];
		this.lastChangedId = null;
		this.lastImpact = { newlyVisible: [], newlyHidden: [] };
	}

	/** Compute which options depend on a given option id. */
	dependentsOf(optionId: string): OptionDef[] {
		if (!this.currentSchema) return [];
		return this.currentSchema.options.filter((o) =>
			(o.dependencies ?? []).some((d) => d.option_id === optionId)
		);
	}

	resetToDefaults() {
		if (!this.currentSchema) return;
		const next: Record<string, Value> = {};
		for (const opt of this.currentSchema.options) {
			next[opt.id] = defaultFor(opt);
		}
		this.values = next;
		this.presetId = undefined;
	}

	applyPreset(presetId: string) {
		if (!this.currentSchema) return;
		const preset = this.currentSchema.presets.find((p) => p.id === presetId);
		if (!preset) return;
		// Start from defaults so missing keys fall back sensibly
		const next: Record<string, Value> = {};
		for (const opt of this.currentSchema.options) {
			next[opt.id] = preset.values[opt.id] ?? defaultFor(opt);
		}
		this.values = next;
		this.presetId = presetId;
	}

	setValue(id: string, value: Value) {
		// Snapshot the visibility of dependents BEFORE the change
		const deps = this.dependentsOf(id);
		const wasVisible = new Map(
			deps.map((d) => [d.id, dependenciesSatisfied(d.dependencies, this.values)])
		);

		this.values = { ...this.values, [id]: value };
		// Changing a value invalidates the preset selection
		this.presetId = undefined;

		const newlyVisible: string[] = [];
		const newlyHidden: string[] = [];
		for (const d of deps) {
			const now = dependenciesSatisfied(d.dependencies, this.values);
			const before = wasVisible.get(d.id) ?? false;
			if (now && !before) newlyVisible.push(d.id);
			if (!now && before) newlyHidden.push(d.id);
		}

		this.lastChangedId = id;
		this.lastImpact = { newlyVisible, newlyHidden };
	}

	async refreshGenerated() {
		const cfg = this.config;
		if (!cfg) {
			this.generatedYaml = '';
			this.validationErrors = [];
			this.lintIssues = [];
			return;
		}
		try {
			const [yaml, errors] = await Promise.all([api.generateYaml(cfg), api.validateConfig(cfg)]);
			this.generatedYaml = yaml;
			this.validationErrors = errors;
			// lint depends on the generated yaml, run after
			this.lintIssues = await api.lintYaml(cfg.game_id, yaml);
		} catch {
			// errors are already toasted by invoke()
		}
	}
}

export const randomizerStore = new RandomizerStore();
