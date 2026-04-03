/**
 * Fix mojibake in JSON locale files.
 *
 * Mojibake pattern: UTF-8 bytes of a character were read/stored as Latin-1,
 * so e.g. "é" (U+00E9, UTF-8: C3 A9) was stored as "Ã©" (C3=Ã, A9=©).
 *
 * To fix: re-encode the string from Latin-1 bytes back to proper UTF-8.
 */
const fs = require('fs');
const path = require('path');

const messagesDir = path.join(__dirname, '..', 'messages');

/**
 * Attempt to decode a mojibake string.
 * Converts the string by treating each char as a Latin-1 byte,
 * then decoding those bytes as UTF-8.
 */
function fixMojibake(str) {
	try {
		// Encode string as Latin-1 (each char code is a byte), then decode as UTF-8
		const bytes = Buffer.from(str, 'latin1');
		const decoded = bytes.toString('utf8');
		// Simple heuristic: if decoded has fewer chars and contains expected script chars, use it
		// Check for replacement character - if decoding failed, keep original
		if (decoded.includes('\uFFFD')) {
			return str; // Invalid UTF-8, keep original
		}
		return decoded;
	} catch (e) {
		return str;
	}
}

/**
 * Determine if a string looks like mojibake.
 * Signs: contains Ã, â, Å, Ã, etc. before other latin chars,
 * or contains sequences like Ã© Ã  Ã¨ Ã´ Ã¼ etc.
 */
function looksLikeMojibake(str) {
	// Check for common mojibake patterns from Latin-1 double encoding
	// These are multi-byte UTF-8 sequences read as Latin-1
	return (
		/[\xC0-\xC5\xC3][\x80-\xBF]/.test(str) || // Latin extended + continuation bytes as latin1
		/Ã[©ª«¬­®¯°±²³´µ¶·¸¹º»¼½¾¿ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞßàáâãäåæçèéêëìíîïðñòóôõö÷øùúûüýþÿ]/.test(
			str
		)
	);
}

function fixJsonFile(filepath) {
	const fullPath = path.join(messagesDir, filepath);
	if (!fs.existsSync(fullPath)) {
		console.log(`Skipped (not found): ${filepath}`);
		return;
	}
	const raw = fs.readFileSync(fullPath, 'utf8');
	const obj = JSON.parse(raw);
	let fixedCount = 0;

	for (const [key, val] of Object.entries(obj)) {
		if (typeof val !== 'string') continue;
		if (looksLikeMojibake(val)) {
			const fixed = fixMojibake(val);
			if (fixed !== val) {
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

const files = ['fr.json', 'es-ES.json', 'pt-BR.json', 'zh-CN.json'];

for (const f of files) {
	try {
		fixJsonFile(f);
	} catch (e) {
		console.error(`Error processing ${f}:`, e.message);
	}
}

// Verify
console.log('\n=== Verification ===');
for (const f of files) {
	try {
		const fullPath = path.join(messagesDir, f);
		if (!fs.existsSync(fullPath)) {
			console.log(f, '-> (file not found)');
			continue;
		}
		const obj = JSON.parse(fs.readFileSync(fullPath, 'utf8'));
		console.log(f, '-> language_name:', obj.language_name);
	} catch (e) {
		console.log(f, '-> PARSE ERROR:', e.message);
	}
}
