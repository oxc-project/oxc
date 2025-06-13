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
  expect(program.body).toHaveLength(1);

  const declaration = program.body[0];
  expect(declaration.type).toBe('VariableDeclaration');
  expect(declaration.kind).toBe('let');
  expect(Array.isArray(declaration.declarations)).toBe(true);
  expect(declaration.declarations).toHaveLength(1);

  const declarator = declaration.declarations[0];
  expect(declarator.type).toBe('VariableDeclarator');
});

it('returns same node object on each access', () => {
  const { program } = parseSyncLazy('test.js', 'let x = y + z;');
  const id1 = program.body[0].declarations[0].id;
  const id2 = program.body[0].declarations[0].id;
  expect(id2).toBe(id1);
});
