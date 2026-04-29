/**
 * Tokenizer for Server (`/`) and Client (`!`) command lines.
 *
 * Supports quoted args ("two words" or 'two words') with `\"` / `\'` escapes
 * inside. Unquoted tokens split on whitespace. Leading prefix is mandatory ,
 * a raw line without `/` or `!` is treated as chat text, returned with
 * `kind: 'chat'`.
 */

export type ParsedCommand =
	| { kind: 'command'; prefix: '/' | '!'; name: string; args: string[]; raw: string }
	| { kind: 'chat'; text: string; raw: string }
	| { kind: 'empty' };

export function parseLine(raw: string): ParsedCommand {
	const trimmed = raw.trim();
	if (!trimmed) return { kind: 'empty' };

	const first = trimmed[0];
	if (first !== '/' && first !== '!') {
		return { kind: 'chat', text: trimmed, raw };
	}

	const rest = trimmed.slice(1);
	const tokens = tokenize(rest);
	if (tokens.length === 0) {
		return { kind: 'chat', text: trimmed, raw };
	}

	const [name, ...args] = tokens;
	return { kind: 'command', prefix: first as '/' | '!', name, args, raw };
}

export function tokenize(s: string): string[] {
	const out: string[] = [];
	let buf = '';
	let quote: '"' | "'" | null = null;
	let escape = false;

	for (let i = 0; i < s.length; i++) {
		const c = s[i];

		if (escape) {
			buf += c;
			escape = false;
			continue;
		}

		if (c === '\\') {
			escape = true;
			continue;
		}

		if (quote) {
			if (c === quote) {
				quote = null;
			} else {
				buf += c;
			}
			continue;
		}

		if (c === '"' || c === "'") {
			quote = c;
			continue;
		}

		if (/\s/.test(c)) {
			if (buf.length > 0) {
				out.push(buf);
				buf = '';
			}
			continue;
		}

		buf += c;
	}

	if (buf.length > 0) out.push(buf);
	return out;
}
