/**
 * Fix zh-CN.json: re-decode all strings that are mojibake of Chinese.
 *
 * Strategy: for every value that contains no CJK but is not the same as English,
 * attempt latin1->utf8 re-decode. Accept if result has CJK or is otherwise valid.
 */
const fs = require('fs');

const o = JSON.parse(fs.readFileSync('messages/zh-CN.json', 'utf8'));
const en = JSON.parse(fs.readFileSync('messages/en.json', 'utf8'));

function hasCJK(str) {
	return /[\u4E00-\u9FFF\u3400-\u4DBF\u3000-\u303F\uFF00-\uFFEF]/.test(str);
}

function tryDecode(str) {
	const bytes = Buffer.from(str, 'latin1');
	const decoded = bytes.toString('utf8');
	return decoded;
}

let fixed = 0;
let skipped = 0;
let failed = 0;

for (const [key, val] of Object.entries(o)) {
	if (typeof val !== 'string') continue;
	if (hasCJK(val)) continue; // already correct Chinese
	if (key === '$schema') continue;

	// This looks like it might be mojibake - try decode
	const decoded = tryDecode(val);

	// Check if decoded has replacement chars
	if (decoded.includes('\uFFFD')) {
		// Partial decode - try character by character approach
		// Some strings mix Chinese mojibake with latin text or interpolation vars
		// Try to fix only the mojibake portions

		// Split on template vars like {name}, {count}, etc.
		const parts = val.split(/(\{[^}]+\}|\s+)/);
		let allFixed = true;
		const fixedParts = parts.map((part) => {
			if (/^\{[^}]+\}$/.test(part) || /^\s+$/.test(part) || part === '') {
				return part; // Keep interpolation vars and whitespace as-is
			}
			const d = tryDecode(part);
			if (d.includes('\uFFFD')) {
				// Try fixing byte by byte - some chars may be HTML-escaped
				// The & in the hex was JSON escaping \u0026
				// Try with the actual bytes
				const bytes = Buffer.from(part, 'latin1');
				// Replace common issue: 0x26 (&) between two high bytes may indicate
				// JSON unicode escaping was applied. Let's check if this fixes it
				const d2 = bytes.toString('utf8');
				if (!d2.includes('\uFFFD') && (hasCJK(d2) || d2.length <= part.length)) {
					return d2;
				}
				allFixed = false;
				return part;
			}
			return d;
		});

		if (allFixed) {
			const result = fixedParts.join('');
			if (result !== val) {
				o[key] = result;
				fixed++;
			}
		} else {
			console.log('SKIP (partial decode issue):', key, '->', JSON.stringify(val).substring(0, 80));
			skipped++;
		}
	} else if (hasCJK(decoded) || decoded.length < val.length) {
		o[key] = decoded;
		fixed++;
	} else {
		// Decoded but no obvious improvement - check if it's actually a legitimate non-CJK value
		// e.g. template vars only, latin names, etc.
		const isTemplateOnly = /^[\s\{][^}]*\}[\s,]*$/.test(val.trim());
		if (!isTemplateOnly && val !== en[key]) {
			// Might still be mojibake that happens to decode to Latin
			skipped++;
		}
	}
}

console.log('Fixed:', fixed, '| Skipped:', skipped, '| Failed:', failed);
fs.writeFileSync('messages/zh-CN.json', JSON.stringify(o, null, '\t') + '\n', 'utf8');

// Verify key strings
const v = JSON.parse(fs.readFileSync('messages/zh-CN.json', 'utf8'));
console.log('\nVerification:');
console.log('prefs_global_title:', v.prefs_global_title);
console.log('language_name:', v.language_name);
console.log('dashboard_title:', v.dashboard_title);
console.log('navBar_label_settings:', v.navBar_label_settings);
console.log('menuBar_file_title:', v.menuBar_file_title);
