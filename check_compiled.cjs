const fs = require('fs');

function checkKey(file, key) {
	const c = fs.readFileSync(file, 'utf8');
	const re = new RegExp(key.replace(/_/g, '_') + ' =.*\\n.*return `([^`]+)`');
	const m = c.match(re);
	return m ? m[1] : 'NOT FOUND';
}

console.log('=== fr.js ===');
console.log(
	'prefs_global_title:',
	checkKey('src/lib/paraglide/messages/fr.js', 'prefs_global_title')
);
console.log('language_name:', checkKey('src/lib/paraglide/messages/fr.js', 'language_name'));

console.log('\n=== zh-CN.js ===');
console.log(
	'prefs_global_title:',
	checkKey('src/lib/paraglide/messages/zh-CN.js', 'prefs_global_title')
);
console.log('dashboard_title:', checkKey('src/lib/paraglide/messages/zh-CN.js', 'dashboard_title'));
console.log('language_name:', checkKey('src/lib/paraglide/messages/zh-CN.js', 'language_name'));
