// Mirror of src-tauri/src/randomizer/types.rs

export type Value = boolean | number | string | Value[] | { [key: string]: Value };

export type Choice = {
	value: string;
	label: string;
	description?: string;
};

export type OptionType =
	| { kind: 'toggle'; default: boolean }
	| { kind: 'range'; min: number; max: number; step: number; default: number }
	| { kind: 'select'; choices: Choice[]; default: string }
	| { kind: 'multi_select'; choices: Choice[]; defaults: string[] }
	| { kind: 'text'; default: string; placeholder?: string };

export type Condition =
	| { op: 'equals'; value: Value }
	| { op: 'not_equals'; value: Value }
	| { op: 'in'; values: Value[] };

export type Dependency = {
	option_id: string;
	condition: Condition;
};

export type OptionDef = {
	id: string;
	label: string;
	description: string;
	category: string;
	type: OptionType;
	dependencies?: Dependency[];
	advanced: boolean;
};

export type Preset = {
	id: string;
	name: string;
	description: string;
	values: Record<string, Value>;
};

export type GameMeta = {
	rom_required: boolean;
	supported_versions: string[];
	wiki_url?: string;
	icon?: string;
	/** True for worlds bundled with Archipelago (served on archipelago.gg).
	 *  False for user-installed .apworld files whose tutorials live only in
	 *  the apworld itself. Backward-compat default is true. */
	is_official?: boolean;
	/** Author-defined English tutorial link (e.g. "setup/en"). Absent when the
	 *  world defines no tutorial. In which case the UI must not render an
	 *  archipelago.gg link. */
	tutorial_path?: string;
};

export type GameSchema = {
	id: string;
	name: string;
	version: string;
	description: string;
	options: OptionDef[];
	presets: Preset[];
	meta: GameMeta;
	/** Item names for start_inventory autocomplete. Empty on old schemas. */
	items: string[];
};

export type GameSummary = {
	id: string;
	name: string;
	version: string;
	description: string;
	option_count: number;
	preset_count: number;
	icon: string | null;
};

export type RandomizerConfig = {
	game_id: string;
	seed?: string;
	values: Record<string, Value>;
	preset_id?: string;
	player_name?: string;
};

export type ValidationError = {
	option_id: string;
	message: string;
};

export type LintIssue = {
	level: 'error' | 'warning' | 'info';
	message: string;
};

export type PythonStatus = {
	available: boolean;
	executable: string | null;
	version: string | null;
	ap_dir: string;
	ap_present: boolean;
};

export type PlayerFile = {
	name: string;
	path: string;
	size: number;
};

export type SeedFile = {
	name: string;
	path: string;
	size: number;
	modified: number;
};

export type GenerateOutcome = {
	success: boolean;
	zip_path: string | null;
	stdout: string;
	stderr: string;
};

export type ServerStatus = {
	running: boolean;
	port: number | null;
	password: string | null;
	multidata: string | null;
	pid: number | null;
	recent_log: string[];
	local_ip: string | null;
	public_ip: string | null;
	port_reachable: boolean;
};

export type CustomApworld = {
	file_name: string;
	path: string;
	size: number;
	world_id: string | null;
	display_name: string | null;
	world_version: string | null;
	has_schema: boolean;
};

export type ApworldRefreshResult = {
	success: boolean;
	stdout: string;
	stderr: string;
	out_dir: string;
};

export type RuntimeStatus = {
	installed: boolean;
	venv_ready: boolean;
	path: string;
	bytes_on_disk: number;
	world_count: number;
	default_download_url: string;
};

export type PatchFile = {
	path: string;
	file_name: string;
	extension: string;
	size: number;
	modified: number;
	seed_stem: string | null;
	player_label: string | null;
	has_rom_registered: boolean;
	output_rom_path: string | null;
};

export type RuntimeProgress =
	| { stage: 'downloading'; received: number; total: number | null }
	| { stage: 'extracting'; entry: string; done: number; total: number }
	| { stage: 'provisioning_venv'; message: string }
	| { stage: 'installing_deps'; message: string }
	| { stage: 'installed'; path: string }
	| { stage: 'failed'; error: string };

export const CATEGORY_ORDER = [
	'general',
	'goals',
	'items',
	'entrances',
	'logic',
	'cosmetic',
	'advanced'
] as const;

export const CATEGORY_LABELS: Record<string, string> = {
	general: 'General',
	goals: 'Goals',
	items: 'Items',
	entrances: 'Entrances',
	logic: 'Logic',
	cosmetic: 'Cosmetic',
	advanced: 'Advanced'
};

export const CATEGORY_ICONS: Record<string, string> = {
	general: 'mdi:cog',
	goals: 'mdi:flag-checkered',
	items: 'mdi:treasure-chest',
	entrances: 'mdi:door',
	logic: 'mdi:brain',
	cosmetic: 'mdi:palette',
	advanced: 'mdi:tune-variant'
};
