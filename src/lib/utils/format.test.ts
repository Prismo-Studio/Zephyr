import { describe, it, expect } from 'vitest';
import { shortenFileSize, shortenNum, formatModName } from './format';

describe('shortenFileSize', () => {
	it('formats zero bytes', () => {
		expect(shortenFileSize(0)).toBe('0.0B');
	});

	it('formats bytes', () => {
		expect(shortenFileSize(500)).toBe('500.0B');
	});

	it('formats kilobytes', () => {
		expect(shortenFileSize(1024)).toBe('1.0kB');
	});

	it('formats megabytes', () => {
		expect(shortenFileSize(1048576)).toBe('1.0MB');
	});

	it('formats gigabytes', () => {
		expect(shortenFileSize(1073741824)).toBe('1.0GB');
	});
});

describe('shortenNum', () => {
	it('formats zero', () => {
		expect(shortenNum(0)).toBe('0');
	});

	it('formats small numbers as-is', () => {
		expect(shortenNum(999)).toBe('999');
	});

	it('formats thousands with k suffix', () => {
		expect(shortenNum(1500)).toBe('1.5k');
	});

	it('formats millions with M suffix', () => {
		expect(shortenNum(2500000)).toBe('2.5M');
	});
});

describe('formatModName', () => {
	it('replaces underscores with spaces', () => {
		expect(formatModName('my_cool_mod')).toBe('my cool mod');
	});

	it('leaves names without underscores unchanged', () => {
		expect(formatModName('MyMod')).toBe('MyMod');
	});
});
