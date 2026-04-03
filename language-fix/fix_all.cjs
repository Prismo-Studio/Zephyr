/**
 * Apply mojibake fix to all locale files using the Windows-1252 map.
 */
const fs = require('fs');
const path = require('path');

const messagesDir = path.join(__dirname, '..', 'messages');

const win1252ToBytes = {
	0x20ac: 0x80,
	0x201a: 0x82,
	0x0192: 0x83,
	0x201e: 0x84,
	0x2026: 0x85,
	0x2020: 0x86,
	0x2021: 0x87,
	0x02c6: 0x88,
	0x2030: 0x89,
	0x0160: 0x8a,
	0x2039: 0x8b,
	0x0152: 0x8c,
	0x017d: 0x8e,
	0x2018: 0x91,
	0x2019: 0x92,
	0x201c: 0x93,
	0x201d: 0x94,
	0x2022: 0x95,
	0x2013: 0x96,
	0x2014: 0x97,
	0x02dc: 0x98,
	0x2122: 0x99,
	0x0161: 0x9a,
	0x203a: 0x9b,
	0x0153: 0x9c,
	0x017e: 0x9e,
	0x0178: 0x9f
};

function decodeMojibake(str) {
	const bytes = [];
	for (const c of str) {
		const cp = c.codePointAt(0);
		if (win1252ToBytes[cp] !== undefined) bytes.push(win1252ToBytes[cp]);
		else if (cp <= 0xff) bytes.push(cp);
		else return null;
	}
	const decoded = Buffer.from(bytes).toString('utf8');
	if (decoded.includes('\uFFFD')) return null;
	return decoded;
}

function hasCJK(str) {
	return /[\u4E00-\u9FFF\u3400-\u4DBF]/.test(str);
}

function looksLikeMojibake(str) {
	// Ã followed by accented char = clear mojibake signal
	return /Ã[^\s]/.test(str) || /â[^\s]/.test(str);
}

function fixFile(filepath, forceAll) {
	const fullPath = path.join(messagesDir, filepath);
	const raw = fs.readFileSync(fullPath, 'utf8');
	const obj = JSON.parse(raw);
	let fixed = 0;

	for (const [key, val] of Object.entries(obj)) {
		if (typeof val !== 'string' || key === '$schema') continue;
		if (!forceAll && !looksLikeMojibake(val)) continue;
		if (forceAll && hasCJK(val)) continue;

		const parts = val.split(/(\{[^}]+\})/);
		const fixedParts = parts.map((part) => {
			if (/^\{[^}]+\}$/.test(part) || part === '') return part;
			const decoded = decodeMojibake(part);
			return decoded !== null && decoded !== part ? decoded : part;
		});
		const result = fixedParts.join('');
		if (result !== val) {
			obj[key] = result;
			fixed++;
		}
	}

	fs.writeFileSync(fullPath, JSON.stringify(obj, null, '\t') + '\n', 'utf8');
	console.log(filepath + ': fixed ' + fixed);
	return fixed;
}

// Latin-based languages: fix strings that obviously have Ã-mojibake
fixFile('fr.json', false);
fixFile('es-ES.json', false);
fixFile('pt-BR.json', false);
// zh-CN already fixed by fix_final.cjs (uses forceAll=true for CJK detection)

// Final verification
const files = ['fr', 'es-ES', 'pt-BR', 'zh-CN'];
console.log('\nVerification:');
for (const f of files) {
	const fullPath = path.join(messagesDir, f + '.json');
	const o = JSON.parse(fs.readFileSync(fullPath, 'utf8'));
	console.log(f + ': ' + o.language_name + ' | prefs_global= ' + o.prefs_global_title);
}
