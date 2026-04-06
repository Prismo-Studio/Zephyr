import { describe, it, expect } from 'vitest';
import { isValidHex, emptyOrUndefined } from './validation';

describe('isValidHex', () => {
	it('accepts 6-char hex with hash', () => {
		expect(isValidHex('#1afffa')).toBe(true);
		expect(isValidHex('#0A0E1A')).toBe(true);
	});

	it('accepts 6-char hex without hash', () => {
		expect(isValidHex('1afffa')).toBe(true);
		expect(isValidHex('FF00AA')).toBe(true);
	});

	it('rejects invalid hex strings', () => {
		expect(isValidHex('#GGG000')).toBe(false);
		expect(isValidHex('#fff')).toBe(false);
		expect(isValidHex('')).toBe(false);
		expect(isValidHex('hello')).toBe(false);
	});
});

describe('emptyOrUndefined', () => {
	it('returns undefined for empty string', () => {
		expect(emptyOrUndefined('')).toBeUndefined();
	});

	it('returns the string for non-empty', () => {
		expect(emptyOrUndefined('hello')).toBe('hello');
		expect(emptyOrUndefined(' ')).toBe(' ');
	});
});
