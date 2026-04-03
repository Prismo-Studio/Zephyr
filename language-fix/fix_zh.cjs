/**
 * Fix mojibake in zh-CN.json.
 *
 * Chinese chars have UTF-8 bytes in ranges: E4-E9 (3-byte), and those bytes
 * when read as Latin-1 produce sequences containing ä, å, æ, ç, è, é, ê
 * followed by characters in the \x80-\xBF range (which appear as control chars
 * or specific Latin Extended).
 *
 * The fix: for each string value, try decoding as latin1->utf8.
 * If the result contains Chinese (CJK) chars and the original doesn't, use decoded.
 */
const fs = require('fs');
const path = require('path');

const messagesDir = path.join(__dirname, '..', 'messages');

function fixMojibake(str) {
	try {
		const bytes = Buffer.from(str, 'latin1');
		const decoded = bytes.toString('utf8');
		if (decoded.includes('\uFFFD')) return str;
		return decoded;
	} catch (e) {
		return str;
	}
}

function hasCJK(str) {
	return /[\u4E00-\u9FFF\u3400-\u4DBF\u3000-\u303F]/.test(str);
}

function looksLikeMojibakeZh(str) {
	// CJK chars (3-byte UTF-8: E4-E9 xx xx) decoded as latin1 produce 3 latin chars
	// Like: ä» (E4 BB) + third byte, å¼ (E5 BC) etc.
	// Detect by: trying to decode and checking if result has CJK or is more valid
	if (hasCJK(str)) return false; // already correct Chinese
	const decoded = fixMojibake(str);
	// If decoding produced Chinese characters, it was mojibake
	return hasCJK(decoded) || decoded.length < str.length;
}

function fixJsonFile(filepath, forceDecodeAll) {
	const fullPath = path.join(messagesDir, filepath);
	const raw = fs.readFileSync(fullPath, 'utf8');
	const obj = JSON.parse(raw);
	let fixedCount = 0;

	for (const [key, val] of Object.entries(obj)) {
		if (typeof val !== 'string') continue;

		let shouldFix = false;
		if (forceDecodeAll) {
			shouldFix = looksLikeMojibakeZh(val);
		} else {
			// For Latin-based languages, check for obvious mojibake patterns
			shouldFix =
				/Ã[©ª«¬­®¯°±²³´µ¶·¸¹º»¼½¾¿ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞßàáâãäåæçèéêëìíîïðñòóôõö÷øùúûüýþÿ]/.test(
					val
				);
		}

		if (shouldFix) {
			const fixed = fixMojibake(val);
			if (fixed !== val && !fixed.includes('\uFFFD')) {
				obj[key] = fixed;
				fixedCount++;
			}
		}
	}

	if (fixedCount > 0) {
		fs.writeFileSync(fullPath, JSON.stringify(obj, null, '\t') + '\n', 'utf8');
		console.log(`Fixed ${fixedCount} strings in: ${filepath}`);
	} else {
		console.log(`No mojibake found in: ${filepath}`);
	}
}

// zh-CN needs the CJK-aware check
fixJsonFile('zh-CN.json', true);

// Verify
const o = JSON.parse(fs.readFileSync(path.join(messagesDir, 'zh-CN.json'), 'utf8'));
console.log('\nVerification:');
console.log('zh prefs_global_title:', o.prefs_global_title);
console.log('zh language_name:', o.language_name);
console.log('zh dashboard_title:', o.dashboard_title);
console.log('zh navBar_label_settings:', o.navBar_label_settings);
