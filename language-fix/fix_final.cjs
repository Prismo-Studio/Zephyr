/**
 * Complete Windows-1252 to byte mapping for mojibake fix.
 * Windows-1252 maps 0x80-0x9F to special Unicode chars.
 * When these were stored in a UTF-8 file as Latin-1, they appear as their Unicode equivalents.
 */
const fs = require('fs');
const path = require('path');

const messagesDir = path.join(__dirname, '..', 'messages');

// Complete Windows-1252 special char map (0x80-0x9F range)
const win1252ToBytes = {
	0x20ac: 0x80, // €
	0x201a: 0x82, // ‚
	0x0192: 0x83, // ƒ
	0x201e: 0x84, // „
	0x2026: 0x85, // …
	0x2020: 0x86, // †
	0x2021: 0x87, // ‡
	0x02c6: 0x88, // ˆ
	0x2030: 0x89, // ‰
	0x0160: 0x8a, // Š
	0x2039: 0x8b, // ‹
	0x0152: 0x8c, // Œ
	0x017d: 0x8e, // Ž
	0x2018: 0x91, // '
	0x2019: 0x92, // '
	0x201c: 0x93, // "
	0x201d: 0x94, // "
	0x2022: 0x95, // •
	0x2013: 0x96, // –
	0x2014: 0x97, // —
	0x02dc: 0x98, // ˜
	0x2122: 0x99, // ™
	0x0161: 0x9a, // š
	0x203a: 0x9b, // ›
	0x0153: 0x9c, // œ
	0x017e: 0x9e, // ž
	0x0178: 0x9f // Ÿ
};

function hasCJK(str) {
	return /[\u4E00-\u9FFF\u3400-\u4DBF]/.test(str);
}

function decodeMojibake(str) {
	const bytes = [];
	for (const c of str) {
		const cp = c.codePointAt(0);
		if (win1252ToBytes[cp] !== undefined) {
			bytes.push(win1252ToBytes[cp]);
		} else if (cp <= 0xff) {
			bytes.push(cp);
		} else {
			return null; // Can't decode this char
		}
	}
	const buf = Buffer.from(bytes);
	const decoded = buf.toString('utf8');
	if (decoded.includes('\uFFFD')) return null;
	return decoded;
}

function fixFile(filepath) {
	const fullPath = path.join(messagesDir, filepath);
	const raw = fs.readFileSync(fullPath, 'utf8');
	const obj = JSON.parse(raw);
	let fixed = 0;
	let failed = 0;

	for (const [key, val] of Object.entries(obj)) {
		if (typeof val !== 'string' || key === '$schema') continue;
		if (hasCJK(val)) continue;

		// Split on template variables to preserve them
		const parts = val.split(/(\{[^}]+\})/);
		const fixedParts = [];
		let anyFixed = false;
		let anyFailed = false;

		for (const part of parts) {
			if (/^\{[^}]+\}$/.test(part) || part === '') {
				fixedParts.push(part);
				continue;
			}

			const decoded = decodeMojibake(part);
			if (decoded !== null && decoded !== part) {
				fixedParts.push(decoded);
				anyFixed = true;
			} else if (
				decoded === null &&
				part.trim() !== '' &&
				part.trim() !== '.' &&
				part.trim() !== ',' &&
				part.trim() !== ':' &&
				part.trim() !== '%s'
			) {
				fixedParts.push(part);
				// Don't mark as failed if it's punctuation or placeholder
			} else {
				fixedParts.push(part);
			}
		}

		if (anyFixed) {
			obj[key] = fixedParts.join('');
			fixed++;
		}
	}

	fs.writeFileSync(fullPath, JSON.stringify(obj, null, '\t') + '\n', 'utf8');
	console.log(filepath + ': fixed=' + fixed + ' failed=' + failed);
	return { fixed, failed };
}

// Fix zh-CN
fixFile('zh-CN.json');

// Verify
const v = JSON.parse(fs.readFileSync(path.join(messagesDir, 'zh-CN.json'), 'utf8'));
const en = JSON.parse(fs.readFileSync(path.join(messagesDir, 'en.json'), 'utf8'));

function hasCJKCheck(str) {
	return /[\u4E00-\u9FFF]/.test(str);
}

let stillBroken = 0;
for (const [key, val] of Object.entries(v)) {
	if (typeof val !== 'string' || key === '$schema') continue;
	if (!hasCJKCheck(val) && val !== en[key] && val !== en[key]) {
		stillBroken++;
	}
}

console.log('\nVerification:');
console.log('prefs_global_title:', v.prefs_global_title);
console.log('menuBar_file_title:', v.menuBar_file_title);
console.log('language_name:', v.language_name);
console.log('navBar_label_home:', v.navBar_label_home);
console.log('navBar_label_settings:', v.navBar_label_settings);
console.log('Still broken:', stillBroken);
