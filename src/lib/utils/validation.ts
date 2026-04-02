export function isValidHex(str: string) {
	return /^#?([0-9A-Fa-f]{6})$/.test(str);
}

export function emptyOrUndefined(str: string) {
	if (str.length === 0) return undefined;
	return str;
}
