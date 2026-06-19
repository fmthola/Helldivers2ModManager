import { describe, it, expect } from 'vitest';
import { format } from './stringExtensions';

describe('format', () => {
    it('fills named placeholders from an object', () => {
        expect(format('hi {name}', { name: 'Bob' })).toBe('hi Bob');
    });

    it('fills indexed placeholders from an array', () => {
        expect(format('{0}-{1}', ['a', 'b'])).toBe('a-b');
    });

    it('keeps the placeholder when the value is missing or null', () => {
        expect(format('{x}', {})).toBe('{x}');
        expect(format('{x}', { x: null })).toBe('{x}');
    });

    it('stringifies numbers, booleans, bigint, and objects', () => {
        expect(format('{n} {b}', { n: 3, b: true })).toBe('3 true');
        expect(format('{g}', { g: 10n })).toBe('10');
        expect(format('{o}', { o: { a: 1 } })).toBe('{"a":1}');
    });
});
