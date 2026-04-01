const fs = require('fs');

const zh = JSON.parse(fs.readFileSync('messages/zh-CN.json', 'utf8'));
const en = JSON.parse(fs.readFileSync('messages/en.json', 'utf8'));

function hasCJK(str) {
	return /[\u4E00-\u9FFF\u3400-\u4DBF]/.test(str);
}

const broken = [];
for (const [key, val] of Object.entries(zh)) {
	if (typeof val !== 'string' || key === '$schema') continue;
	if (!hasCJK(val) && val !== en[key]) {
		broken.push({ key, val, en: en[key] });
	}
}

console.log('Total broken keys:', broken.length);
console.log('\nSample broken keys:');
broken.slice(0, 30).forEach(({ key, val, en }) => {
	console.log(`  "${key}": "${val}" (EN: "${en}")`);
});
