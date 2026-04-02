export type ConfigValueType<T extends string, C> = { type: T; content: C };

export type ConfigValue =
	| ConfigValueType<'bool', boolean>
	| ConfigValueType<'string', string>
	| ConfigValueType<'int', ConfigNum>
	| ConfigValueType<'float', ConfigNum>
	| ConfigValueType<'enum', { index: number; options: string[] }>
	| ConfigValueType<'flags', { indicies: number[]; options: string[] }>;

export type ConfigEntry = {
	name: string;
	description: string | null;
	default: ConfigValue | null;
	value: ConfigValue;
};

export type ConfigSection = {
	name: string;
	entries: ConfigEntry[];
};

export type ConfigFileData = {
	displayName: string;
	relativePath: string;
	sections: ConfigSection[];
	metadata: ConfigFileMetadata | null;
};

export type ConfigFileMetadata = {
	modName: string;
	modVersion: string;
};

export type ConfigNum = {
	value: number;
	range: ConfigRange | null;
};

export type ConfigRange = {
	start: number;
	end: number;
};

export type ConfigFileType<T extends string, C = {}> = { type: T } & C;

export type ConfigFile = { relativePath: string; displayName: string | null } & (
	| ConfigFileType<'ok', ConfigFileData>
	| ConfigFileType<'err', { error: string }>
	| ConfigFileType<'unsupported'>
);

export type ConfigEntryId = {
	file: { relativePath: string };
	section: ConfigSection;
	entry: ConfigEntry;
};
