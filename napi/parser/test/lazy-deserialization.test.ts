// Tests for lazy deserialization.

// @ts-nocheck

import { describe, expect, it } from 'vitest';

import { parseSync } from '../src-js/index.js';

function parseSyncLazy(filename, code, options = null) {
  return parseSync(filename, code, { ...options, experimentalLazy: true });
}

// Get `NodeArray` constructor
const NodeArray = Object.getPrototypeOf(parseSyncLazy('test.js', '').program.body).constructor;
// oxlint-disable eslint-plugin-jest/no-standalone-expect
expect(NodeArray).not.toBe(Array);
expect(NodeArray.toString().startsWith('class NodeArray extends Array {')).toBe(true);
// oxlint-enable eslint-plugin-jest/no-standalone-expect

it('parses', () => {
  const { program } = parseSyncLazy('test.js', 'let x = y + z;');
  expect(program.type).toBe('Program');
  expect(Array.isArray(program.body)).toBe(true);
  expect(program.body.length).toBe(1);

  const declaration = program.body[0];
  expect(declaration.type).toBe('VariableDeclaration');
  expect(declaration.kind).toBe('let');
  expect(Array.isArray(declaration.declarations)).toBe(true);
  expect(declaration.declarations.length).toBe(1);

  const declarator = declaration.declarations[0];
  expect(declarator.type).toBe('VariableDeclarator');
});

it('returns same node objects and node arrays on each access', () => {
  const data = parseSyncLazy('test.js', 'let x = y + z;');
  const { program } = data;
  expect(data.program).toBe(program);
  const { body } = program;
  expect(program.body).toBe(body);
  const stmt = body[0];
  expect(body[0]).toBe(stmt);
  const { declarations } = stmt;
  expect(stmt.declarations).toBe(declarations);
  const declaration = declarations[0];
  expect(declarations[0]).toBe(declaration);
  const { id } = declaration;
  expect(declaration.id).toBe(id);

  expect(program.body[0].declarations[0].id).toBe(id);
});

describe('NodeArray', () => {
  describe('methods', () => {
    it('at', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(body.at(0)).toBe(body[0]);
      expect(body.at(1)).toBe(body[1]);
      expect(body.at(2)).toBeUndefined();
    });

    it('concat', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const concat = body.concat(body);
      expect(Array.isArray(concat)).toBe(true);
      expect(Object.getPrototypeOf(concat)).toBe(Array.prototype);
      expect(concat).toHaveLength(4);
      expect(concat[0]).toBe(body[0]);
      expect(concat[1]).toBe(body[1]);
      expect(concat[2]).toBe(body[0]);
      expect(concat[3]).toBe(body[1]);
    });

    it('copyWithin (throws)', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(() => body.copyWithin(0, 1, 2)).toThrow(new TypeError('Cannot redefine property: 0'));
    });

    it('entries', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const entries = [...body.entries()];
      expect(entries).toStrictEqual([[0, body[0]], [1, body[1]]]);
      expect(entries[0][1]).toBe(body[0]);
      expect(entries[1][1]).toBe(body[1]);
    });

    it('every', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(body.every(stmt => typeof stmt.type === 'string')).toBe(true);
      expect(body.every(stmt => stmt.type === 'VariableDeclaration')).toBe(false);
    });

    it('fill (throws)', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(() => body.fill(0)).toThrow(new TypeError('Cannot redefine property: 0'));
    });

    it('filter', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const filter = body.filter(stmt => stmt.type === 'VariableDeclaration');
      expect(Array.isArray(filter)).toBe(true);
      expect(Object.getPrototypeOf(filter)).toBe(Array.prototype);
      expect(filter).toHaveLength(1);
      expect(filter[0]).toBe(body[0]);
    });

    it('find', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(body.find(stmt => stmt.type === 'VariableDeclaration')).toBe(body[0]);
      expect(body.find(stmt => stmt.type === 'ExpressionStatement')).toBe(body[1]);
    });

    it('findIndex', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(body.findIndex(stmt => stmt.type === 'VariableDeclaration')).toBe(0);
      expect(body.findIndex(stmt => stmt.type === 'ExpressionStatement')).toBe(1);
    });

    it('findLast', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(body.findLast(() => true)).toBe(body[1]);
      expect(body.findLast(stmt => stmt.type === 'VariableDeclaration')).toBe(body[0]);
    });

    it('findLastIndex', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(body.findLastIndex(() => true)).toBe(1);
      expect(body.findLastIndex(stmt => stmt.type === 'VariableDeclaration')).toBe(0);
    });

    it('flat', () => {
      // Can't test flattening of nested arrays, as we don't have `Vec<Vec>` in AST
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const flat = body.flat();
      expect(Array.isArray(flat)).toBe(true);
      expect(Object.getPrototypeOf(flat)).toBe(Array.prototype);
      expect(flat).toHaveLength(2);
      expect(flat[0]).toBe(body[0]);
      expect(flat[1]).toBe(body[1]);
    });

    it('flatMap', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const flat = body.flatMap(stmt => [stmt, stmt]);
      expect(Array.isArray(flat)).toBe(true);
      expect(Object.getPrototypeOf(flat)).toBe(Array.prototype);
      expect(flat).toHaveLength(4);
      expect(flat[0]).toBe(body[0]);
      expect(flat[1]).toBe(body[0]);
      expect(flat[2]).toBe(body[1]);
      expect(flat[3]).toBe(body[1]);
    });

    it('forEach', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const stmts = [];
      body.forEach(stmt => stmts.push(stmt));
      expect(stmts).toHaveLength(2);
      expect(stmts[0]).toBe(body[0]);
      expect(stmts[1]).toBe(body[1]);
    });

    it('includes', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(body.includes(body[0])).toBe(true);
      expect(body.includes(body[1])).toBe(true);
      expect(body.includes(undefined)).toBe(false);
      expect(body.includes({ ...body[0] })).toBe(false);
    });

    it('indexOf', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(body.indexOf(body[0])).toBe(0);
      expect(body.indexOf(body[1])).toBe(1);
      expect(body.indexOf(undefined)).toBe(-1);
      expect(body.indexOf({ ...body[0] })).toBe(-1);
    });

    it('join', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      // oxlint-disable-next-line typescript-eslint/no-base-to-string
      expect(body.join()).toBe('[object Object],[object Object]');
      // oxlint-disable-next-line typescript-eslint/no-base-to-string
      expect(body.join(' x ')).toBe('[object Object] x [object Object]');
    });

    it('keys', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const keys = [...body.keys()];
      expect(keys).toStrictEqual([0, 1]);
    });

    it('lastIndexOf', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(body.lastIndexOf(body[0])).toBe(0);
      expect(body.lastIndexOf(body[1])).toBe(1);
      expect(body.lastIndexOf(undefined)).toBe(-1);
      expect(body.lastIndexOf({ ...body[0] })).toBe(-1);
    });

    it('map', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const map = body.map(stmt => stmt.type === 'VariableDeclaration');
      expect(Array.isArray(map)).toBe(true);
      expect(Object.getPrototypeOf(map)).toBe(Array.prototype);
      expect(map).toStrictEqual([true, false]);
    });

    it('pop (throws)', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(() => body.pop())
        .toThrow(new TypeError("'deleteProperty' on proxy: trap returned falsish for property '1'"));
    });

    it('push (throws)', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(() => body.push())
        .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property 'length'"));
      expect(() => body.push({}))
        .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property '2'"));
      expect(() => body.push({}, {}))
        .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property '2'"));
    });

    it('reduce', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const stmts = body.reduce(
        (stmts, stmt) => {
          stmts.push(stmt);
          return stmts;
        },
        [],
      );
      expect(stmts).toHaveLength(2);
      expect(stmts[0]).toBe(body[0]);
      expect(stmts[1]).toBe(body[1]);
    });

    it('reduceRight', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const stmts = body.reduceRight(
        (stmts, stmt) => {
          stmts.push(stmt);
          return stmts;
        },
        [],
      );
      expect(stmts).toHaveLength(2);
      expect(stmts[0]).toBe(body[1]);
      expect(stmts[1]).toBe(body[0]);
    });

    it('reverse (throws)', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(() => body.reverse()).toThrow(new TypeError('Cannot redefine property: 0'));
    });

    it('shift (throws)', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(() => body.shift()).toThrow(new TypeError('Cannot redefine property: 0'));
    });

    describe('slice', () => {
      class Case {
        constructor(args, expected) {
          this.args = args;
          this.expected = expected;
        }

        toString() {
          return `(${this.args.map(arg => JSON.stringify(arg) || 'undefined').join(', ')})`;
        }
      }

      it.each([
        // No args
        new Case([], { start: 0, length: 3 }),
        // 1 arg integer
        new Case([1], { start: 1, length: 2 }),
        new Case([2], { start: 2, length: 1 }),
        new Case([3], { start: 0, length: 0 }),
        new Case([4], { start: 0, length: 0 }),
        // 1 arg negative integer
        new Case([-0], { start: 0, length: 3 }),
        new Case([-1], { start: 2, length: 1 }),
        new Case([-2], { start: 1, length: 2 }),
        new Case([-3], { start: 0, length: 3 }),
        new Case([-4], { start: 0, length: 3 }),
        // 1 arg undefined/null
        new Case([undefined], { start: 0, length: 3 }),
        new Case([null], { start: 0, length: 3 }),
        // 1 arg non-integer
        new Case(['0'], { start: 0, length: 3 }),
        new Case(['2'], { start: 2, length: 1 }),
        new Case(['-2'], { start: 1, length: 2 }),
        new Case(['0x2'], { start: 2, length: 1 }),
        new Case(['oops'], { start: 0, length: 3 }),
        new Case([false], { start: 0, length: 3 }),
        new Case([true], { start: 1, length: 2 }),
        new Case([{ valueOf: () => 1 }], { start: 1, length: 2 }),
        // 2 args integers
        new Case([0, 0], { start: 0, length: 0 }),
        new Case([0, 1], { start: 0, length: 1 }),
        new Case([0, 2], { start: 0, length: 2 }),
        new Case([0, 3], { start: 0, length: 3 }),
        new Case([0, 4], { start: 0, length: 3 }),
        new Case([1, 2], { start: 1, length: 1 }),
        new Case([1, 3], { start: 1, length: 2 }),
        new Case([1, 4], { start: 1, length: 2 }),
        new Case([3, 3], { start: 0, length: 0 }),
        new Case([3, 5], { start: 0, length: 0 }),
        new Case([3, 0], { start: 0, length: 0 }),
        // 2 args negative integers
        new Case([-1, 3], { start: 2, length: 1 }),
        new Case([-2, 3], { start: 1, length: 2 }),
        new Case([-3, 3], { start: 0, length: 3 }),
        new Case([-4, 3], { start: 0, length: 3 }),
        new Case([-2, 5], { start: 1, length: 2 }),
        new Case([0, -1], { start: 0, length: 2 }),
        new Case([1, -1], { start: 1, length: 1 }),
        new Case([2, -1], { start: 0, length: 0 }),
        new Case([3, -1], { start: 0, length: 0 }),
        new Case([0, -0], { start: 0, length: 0 }),
        new Case([-2, -1], { start: 1, length: 1 }),
        new Case([-3, -1], { start: 0, length: 2 }),
        new Case([-3, -4], { start: 0, length: 0 }),
        // 2 args undefined/null
        new Case([undefined, undefined], { start: 0, length: 3 }),
        new Case([null, null], { start: 0, length: 0 }),
        new Case([null, undefined], { start: 0, length: 3 }),
        new Case([undefined, null], { start: 0, length: 0 }),
        new Case([undefined, 1], { start: 0, length: 1 }),
        new Case([undefined, 3], { start: 0, length: 3 }),
        new Case([undefined, 4], { start: 0, length: 3 }),
        new Case([undefined, -1], { start: 0, length: 2 }),
        new Case([undefined, 0], { start: 0, length: 0 }),
        new Case([null, 1], { start: 0, length: 1 }),
        new Case([null, 3], { start: 0, length: 3 }),
        new Case([null, 4], { start: 0, length: 3 }),
        new Case([null, -1], { start: 0, length: 2 }),
        new Case([null, 0], { start: 0, length: 0 }),
        new Case([0, undefined], { start: 0, length: 3 }),
        new Case([1, undefined], { start: 1, length: 2 }),
        new Case([2, undefined], { start: 2, length: 1 }),
        new Case([3, undefined], { start: 0, length: 0 }),
        new Case([4, undefined], { start: 0, length: 0 }),
        new Case([-2, undefined], { start: 1, length: 2 }),
        new Case([-3, undefined], { start: 0, length: 3 }),
        new Case([-5, undefined], { start: 0, length: 3 }),
        new Case([0, null], { start: 0, length: 0 }),
        new Case([1, null], { start: 0, length: 0 }),
        new Case([3, null], { start: 0, length: 0 }),
        new Case([-2, null], { start: 0, length: 0 }),
        new Case([-3, null], { start: 0, length: 0 }),
        new Case([-5, null], { start: 0, length: 0 }),
        // 2 args non-integers
        new Case(['0', '2'], { start: 0, length: 2 }),
        new Case(['0', '-2'], { start: 0, length: 1 }),
        new Case(['1', '-1'], { start: 1, length: 1 }),
        new Case(['0x1', '0x2'], { start: 1, length: 1 }),
        new Case([0, 'oops'], { start: 0, length: 0 }),
        new Case([false, true], { start: 0, length: 1 }),
        new Case([true, true], { start: 0, length: 0 }),
        new Case([1, { valueOf: () => 2 }], { start: 1, length: 1 }),
      ])('%s', ({ args, expected }) => {
        const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2; x = 3;').program;
        const slice = body.slice(...args);
        expect(Array.isArray(slice)).toBe(true);
        expect(Object.getPrototypeOf(slice)).toBe(NodeArray.prototype);
        expect(slice.length).toBe(expected.length);

        for (let i = 0; i < expected.length; i++) {
          expect(slice[i]).toBe(body[i + expected.start]);
        }

        // Check `Array.prototype.slice` behaves the same
        const arr = [11, 22, 33];
        const arrSlice = arr.slice(...args);
        expect(arrSlice.length).toBe(expected.length);

        for (let i = 0; i < expected.length; i++) {
          expect(arrSlice[i]).toBe(arr[i + expected.start]);
        }
      });
    });

    it('some', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(body.some(stmt => stmt.type === 'VariableDeclaration')).toBe(true);
      expect(body.some(stmt => stmt.type === 'ExpressionStatement')).toBe(true);
      expect(body.some(stmt => stmt.type === 'Donkey')).toBe(false);
    });

    it('sort (throws)', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      // oxlint-disable-next-line typescript-eslint/require-array-sort-compare
      expect(() => body.sort()).toThrow(new TypeError('Cannot redefine property: 0'));
    });

    it('splice (throws)', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(() => body.splice(0, 0))
        .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property 'length'"));
      expect(() => body.splice(0, 1)).toThrow(new TypeError('Cannot redefine property: 0'));
      expect(() => body.splice(0, 0, {}))
        .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property '2'"));
    });

    it('toLocaleString', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      // oxlint-disable-next-line typescript-eslint/no-base-to-string
      expect(body.toLocaleString()).toBe('[object Object],[object Object]');
    });

    it('toReversed', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const reversed = body.toReversed();
      expect(Array.isArray(reversed)).toBe(true);
      expect(Object.getPrototypeOf(reversed)).toBe(Array.prototype);
      expect(reversed).toHaveLength(2);
      expect(reversed[0]).toBe(body[1]);
      expect(reversed[1]).toBe(body[0]);
    });

    it('toSorted', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const sorted = body.toSorted((a, b) => {
        if (a.type === b.type) return 0;
        return a.type > b.type ? 1 : -1;
      });
      expect(Array.isArray(sorted)).toBe(true);
      expect(Object.getPrototypeOf(sorted)).toBe(Array.prototype);
      expect(sorted).toHaveLength(2);
      expect(sorted[0]).toBe(body[1]);
      expect(sorted[1]).toBe(body[0]);
    });

    it('toSpliced', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const spliced = body.toSpliced(1, 0, { x: 1 }, { y: 2 });
      expect(Array.isArray(spliced)).toBe(true);
      expect(Object.getPrototypeOf(spliced)).toBe(Array.prototype);
      expect(spliced).toStrictEqual([body[0], { x: 1 }, { y: 2 }, body[1]]);
      expect(spliced[0]).toBe(body[0]);
      expect(spliced[3]).toBe(body[1]);
    });

    it('toString', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      // oxlint-disable-next-line typescript-eslint/no-base-to-string
      expect(body.toString()).toBe('[object Object],[object Object]');
    });

    it('unshift (throws)', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(() => body.unshift({}))
        .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property '2'"));
    });

    it('values', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const values = [...body.values()];
      expect(values).toHaveLength(2);
      expect(values[0]).toBe(body[0]);
      expect(values[1]).toBe(body[1]);
    });

    it('with', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const withed = body.with(0, { x: 1 });
      expect(Array.isArray(withed)).toBe(true);
      expect(Object.getPrototypeOf(withed)).toBe(Array.prototype);
      expect(withed).toStrictEqual([{ x: 1 }, body[1]]);
      expect(withed[1]).toBe(body[1]);
    });
  });

  it('iteration', () => {
    const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
    const stmts = [];
    for (const stmt of body) {
      stmts.push(stmt);
    }
    expect(stmts).toHaveLength(2);
    expect(stmts[0]).toBe(body[0]);
    expect(stmts[1]).toBe(body[1]);
  });

  it('spread', () => {
    const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
    const stmts = [...body];
    expect(stmts).toHaveLength(2);
    expect(stmts[0]).toBe(body[0]);
    expect(stmts[1]).toBe(body[1]);

    const stmtsTwice = [...body, ...body];
    expect(stmtsTwice).toHaveLength(4);
    expect(stmtsTwice[0]).toBe(body[0]);
    expect(stmtsTwice[1]).toBe(body[1]);
    expect(stmtsTwice[2]).toBe(body[0]);
    expect(stmtsTwice[3]).toBe(body[1]);
  });

  it('get length', () => {
    const body0 = parseSyncLazy('test.js', '').program.body;
    expect(body0.length).toBe(0);
    const body2 = parseSyncLazy('test.js', 'let x = 1; x = 2;').program.body;
    expect(body2.length).toBe(2);
    const body4 = parseSyncLazy('test.js', 'x; y; z; 123;').program.body;
    expect(body4.length).toBe(4);
  });

  it('set length (throws)', () => {
    const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
    expect(() => body.length = 0)
      .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property 'length'"));
    expect(() => body.length = 2)
      .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property 'length'"));
    expect(() => body.length = 3)
      .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property 'length'"));
  });

  it('set length via `defineProperty` (throws)', () => {
    const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
    expect(() => Object.defineProperty(body, 'length', { value: 0 }))
      .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property 'length'"));
    expect(() => Object.defineProperty(body, 'length', { value: 2 }))
      .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property 'length'"));
    expect(() => Object.defineProperty(body, 'length', { value: 3 }))
      .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property 'length'"));
  });

  it('set element (throws)', () => {
    const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
    expect(() => body[0] = {}).toThrow(new TypeError('Cannot redefine property: 0'));
    expect(() => body[1] = {}).toThrow(new TypeError('Cannot redefine property: 1'));
    expect(() => body[2] = {})
      .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property '2'"));
    expect(() => body[4294967294] = {})
      .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property '4294967294'"));
  });

  it('set element via `defineProperty` (throws)', () => {
    const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
    expect(() => Object.defineProperty(body, 0, { value: {} }))
      .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property '0'"));
    expect(() => Object.defineProperty(body, 1, { value: {} }))
      .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property '1'"));
    expect(() => Object.defineProperty(body, 2, { value: {} }))
      .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property '2'"));
    expect(() => Object.defineProperty(body, 4294967294, { value: {} }))
      .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property '4294967294'"));
  });

  const propertyKeys = [
    'foo',
    'bar',
    Symbol('yeah'),
    -1,
    '01',
    '0x1',
    '1 ',
    ' 1',
    '1e1',
    '4294967295',
    '4294967296',
    '10000000000000',
  ];

  it('set properties', () => {
    const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;

    for (const [i, key] of propertyKeys.entries()) {
      body[key] = i + 100;
    }

    for (const [i, key] of propertyKeys.entries()) {
      expect(body[key]).toBe(i + 100);
    }
  });

  it('set properties via `defineProperty`', () => {
    const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;

    for (const [i, key] of propertyKeys.entries()) {
      Object.defineProperty(body, key, { value: i + 100 });
    }

    for (const [i, key] of propertyKeys.entries()) {
      expect(body[key]).toBe(i + 100);
    }
  });

  describe('reflection', () => {
    it('Object.keys', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const keys = Object.keys(body);
      expect(keys).toStrictEqual(['0', '1']);
      // Check same as array
      expect(Object.keys([...body])).toStrictEqual(keys);
    });

    it('Object.values', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const values = Object.values(body);
      expect(values).toStrictEqual([body[0], body[1]]);
      expect(values[0]).toBe(body[0]);
      expect(values[1]).toBe(body[1]);
      // Check same as array
      expect(Object.values([...body])).toStrictEqual(values);
    });

    it('Object.entries', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const entries = Object.entries(body);
      expect(entries).toStrictEqual([['0', body[0]], ['1', body[1]]]);
      expect(entries[0][1]).toBe(body[0]);
      expect(entries[1][1]).toBe(body[1]);
      // Check same as array
      expect(Object.entries([...body])).toStrictEqual(entries);
    });

    it('Object.getOwnPropertyNames', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const propNames = Object.getOwnPropertyNames(body);
      expect(propNames).toStrictEqual(['0', '1', 'length']);
      // Check same as array
      expect(Object.getOwnPropertyNames([...body])).toStrictEqual(propNames);
    });

    it('Object.getOwnPropertySymbols', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const symbols = Object.getOwnPropertySymbols(body);
      expect(symbols).toStrictEqual([]);
      // Check same as array
      expect(Object.getOwnPropertySymbols([...body])).toStrictEqual(symbols);
    });

    it('Object.getOwnPropertyDescriptor', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;

      const descriptor0 = Object.getOwnPropertyDescriptor(body, 0);
      expect(descriptor0).toStrictEqual({
        value: body[0],
        writable: false,
        enumerable: true,
        configurable: true,
      });
      expect(descriptor0.value).toBe(body[0]);
      // Check same as array, except for `writable`
      expect(Object.getOwnPropertyDescriptor([...body], 0)).toStrictEqual({ ...descriptor0, writable: true });

      const descriptor1 = Object.getOwnPropertyDescriptor(body, 1);
      expect(descriptor1).toStrictEqual({
        value: body[1],
        writable: false,
        enumerable: true,
        configurable: true,
      });
      expect(descriptor1.value).toBe(body[1]);
      // Check same as array, except for `writable`
      expect(Object.getOwnPropertyDescriptor([...body], 1)).toStrictEqual({ ...descriptor1, writable: true });

      const descriptorLength = Object.getOwnPropertyDescriptor(body, 'length');
      expect(descriptorLength).toStrictEqual({
        value: 2,
        writable: true,
        enumerable: false,
        configurable: false,
      });
      // Check same as array
      expect(Object.getOwnPropertyDescriptor([...body], 'length')).toStrictEqual(descriptorLength);
    });

    it('Object.getOwnPropertyDescriptors', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const descriptors = Object.getOwnPropertyDescriptors(body);
      expect(descriptors).toStrictEqual({
        0: {
          value: body[0],
          writable: false,
          enumerable: true,
          configurable: true,
        },
        1: {
          value: body[1],
          writable: false,
          enumerable: true,
          configurable: true,
        },
        length: {
          value: 2,
          writable: true,
          enumerable: false,
          configurable: false,
        },
      });
      expect(descriptors[0].value).toBe(body[0]);
      expect(descriptors[1].value).toBe(body[1]);
      // Check same as array, except for `writable` on elements
      descriptors[0].writable = true;
      descriptors[1].writable = true;
      expect(Object.getOwnPropertyDescriptors([...body])).toStrictEqual(descriptors);
    });

    it('Object.isExtensible', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(Object.isExtensible(body)).toBe(true);
      // Check same as array
      expect(Object.isExtensible([...body])).toBe(true);
    });

    it('Object.isFrozen', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(Object.isFrozen(body)).toBe(false);
      // Check same as array
      expect(Object.isFrozen([...body])).toBe(false);
    });

    it('Object.isSealed', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(Object.isSealed(body)).toBe(false);
      // Check same as array
      expect(Object.isSealed([...body])).toBe(false);
    });
  });
});
