/**
 * Fix all mojibake in zh-CN.json and other locale files.
 *
 * The strings were encoded with Windows-1252/Latin-1 control chars
 * that got converted to their Unicode equivalents during a copy-paste or save operation:
 * - 0x85 (NEL) → U+2026 …
 * - 0x96 (en-dash) → U+2013 –
 * - 0x97 (em-dash) → U+2014 —
 * - 0x93 (left quote) → U+201C "
 * - 0x94 (right quote) → U+201D "
 * - 0x91 → U+2018 '
 * - 0x92 → U+2019 '
 *
 * Then the whole thing was treated as Latin-1 bytes and stored in UTF-8.
 */
const fs = require('fs');
const path = require('path');

const messagesDir = path.join(__dirname, '..', 'messages');

// Map Windows-1252 special chars that were converted to Unicode
const win1252Map = {
	0x2026: 0x85, // … → NEL
	0x2013: 0x96, // – → en-dash in cp1252
	0x2014: 0x97, // — → em-dash in cp1252
	0x201c: 0x93, // " → left double quote
	0x201d: 0x94, // " → right double quote
	0x2018: 0x91, // ' → left single quote
	0x2019: 0x92, // ' → right single quote
	0x2022: 0x95, // • → bullet
	0x20ac: 0x80, // € → euro
	0x2122: 0x99 // ™ → trademark
};

function hasCJK(str) {
	return /[\u4E00-\u9FFF\u3400-\u4DBF]/.test(str);
}

function fixMojibake(str) {
	// Convert the string to "bytes" by undoing Windows-1252 -> Unicode replacements
	// then treating each remaining char as its latin-1 byte value
	const bytes = [];
	for (const c of str) {
		const cp = c.codePointAt(0);
		if (win1252Map[cp] !== undefined) {
			bytes.push(win1252Map[cp]);
		} else if (cp <= 0xff) {
			bytes.push(cp);
		} else {
			// Character outside latin-1 range that we can't map
			// This might be already-correct Unicode (like proper Chinese)
			// Return null to indicate we can't fix this string
			return null;
		}
	}

	const buf = Buffer.from(bytes);
	const decoded = buf.toString('utf8');
	if (decoded.includes('\uFFFD')) return null;
	return decoded;
}

function processFile(filepath, checkFn) {
	const fullPath = path.join(messagesDir, filepath);
	const raw = fs.readFileSync(fullPath, 'utf8');
	const obj = JSON.parse(raw);
	let fixed = 0;

	for (const [key, val] of Object.entries(obj)) {
		if (typeof val !== 'string' || key === '$schema') continue;
		if (hasCJK(val)) continue; // already correct

		if (!checkFn(val)) continue;

		const decoded = fixMojibake(val);
		if (decoded !== null && decoded !== val) {
			obj[key] = decoded;
			fixed++;
		}
	}

	if (fixed > 0) {
		fs.writeFileSync(fullPath, JSON.stringify(obj, null, '\t') + '\n', 'utf8');
	}
	console.log(filepath + ': fixed ' + fixed + ' strings');
	return fixed;
}

// For zh-CN: fix strings that have no CJK but aren't English
const en = JSON.parse(fs.readFileSync(path.join(messagesDir, 'en.json'), 'utf8'));
processFile('zh-CN.json', (val) => {
	if (hasCJK(val)) return false;
	return true; // Try to fix any non-CJK string
});

// Verify
const v = JSON.parse(fs.readFileSync(path.join(messagesDir, 'zh-CN.json'), 'utf8'));
console.log('\nVerification:');
console.log('prefs_global_title:', v.prefs_global_title);
console.log('menuBar_file_title:', v.menuBar_file_title);
console.log('language_name:', v.language_name);
console.log('navBar_label_home:', v.navBar_label_home);
