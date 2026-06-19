import { describe, it, expect } from 'vitest';
import { toFixedLengthString, toSkipEntry, tryToSkipEntry } from './settings';

describe('settings model', () => {
    it('accepts a string of the exact length', () => {
        expect(toFixedLengthString('abcd', 4)).toBe('abcd');
    });

    it('throws on the wrong length', () => {
        expect(() => toFixedLengthString('abc', 4)).toThrow();
    });

    it('toSkipEntry requires 16 characters', () => {
        expect(toSkipEntry('0123456789abcdef')).toBe('0123456789abcdef');
        expect(() => toSkipEntry('short')).toThrow();
    });

    it('tryToSkipEntry returns null on the wrong length', () => {
        expect(tryToSkipEntry('short')).toBeNull();
        expect(tryToSkipEntry('0123456789abcdef')).toBe('0123456789abcdef');
    });
});
