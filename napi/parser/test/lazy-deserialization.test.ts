// Tests for lazy deserialization.

// @ts-nocheck

import { describe, expect, it } from 'vitest';

import { parseSync } from '../index.js';

function parseSyncLazy(filename, code, options = null) {
  return parseSync(filename, code, { ...options, experimentalLazy: true });
}

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
      expect(body.join()).toBe('[object Object],[object Object]');
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

    it('slice', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      const slice = body.slice(1);
      expect(Array.isArray(slice)).toBe(true);
      expect(Object.getPrototypeOf(slice)).toBe(Array.prototype);
      expect(slice).toHaveLength(1);
      expect(slice[0]).toBe(body[1]);
    });

    it('some', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
      expect(body.some(stmt => stmt.type === 'VariableDeclaration')).toBe(true);
      expect(body.some(stmt => stmt.type === 'ExpressionStatement')).toBe(true);
      expect(body.some(stmt => stmt.type === 'Donkey')).toBe(false);
    });

    it('sort (throws)', () => {
      const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
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

  it('set length (throws)', () => {
    const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
    expect(() => body.length = 0)
      .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property 'length'"));
    expect(() => body.length = 2)
      .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property 'length'"));
    expect(() => body.length = 3)
      .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property 'length'"));
  });

  it('set element (throws)', () => {
    const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;
    expect(() => body[0] = {}).toThrow(new TypeError('Cannot redefine property: 0'));
    expect(() => body[1] = {}).toThrow(new TypeError('Cannot redefine property: 1'));
    expect(() => body[2] = {})
      .toThrow(new TypeError("'defineProperty' on proxy: trap returned falsish for property '2'"));
  });

  it('set properties', () => {
    const { body } = parseSyncLazy('test.js', 'let x = 1; x = 2;').program;

    const keys = [
      'foo',
      'bar',
      Symbol('yeah'),
      -1,
      '01',
      '0x1',
      '1 ',
      ' 1',
      '1e1',
    ];

    for (const [i, key] of keys.entries()) {
      body[key] = i + 100;
    }

    for (const [i, key] of keys.entries()) {
      expect(body[key]).toBe(i + 100);
    }
  });
});
