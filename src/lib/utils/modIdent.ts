/**
 * Extracts the mod name from a Thunderstore VersionIdent ("Owner-Name-1.0.0" → "Name").
 * Returns the input unchanged if it does not contain at least two dashes.
 */
export function modNameFromIdent(ident: string): string {
	const lastDash = ident.lastIndexOf('-');
	if (lastDash === -1) return ident;
	const secondLastDash = ident.lastIndexOf('-', lastDash - 1);
	if (secondLastDash === -1) return ident;
	return ident.substring(secondLastDash + 1, lastDash);
}
