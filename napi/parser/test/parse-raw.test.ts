import { readdir, readFile, writeFile } from 'node:fs/promises';
import { basename, join as pathJoin } from 'node:path';
import { describe, expect, it } from 'vitest';

import { parseSync } from '../index.js';

const TARGET_DIR_PATH = pathJoin(import.meta.dirname, '../../../target');
const TEST262_DIR_PATH = pathJoin(import.meta.dirname, '../../../tasks/coverage/test262/test');
const ACORN_TEST262_DIR_PATH = pathJoin(import.meta.dirname, '../../../tasks/coverage/acorn-test262/test');

// Load/download fixtures.
// Save in `target` directory, same as where benchmarks store them.
const benchFixtureUrls = [
  // TypeScript syntax (2.81MB)
  'https://raw.githubusercontent.com/microsoft/TypeScript/v5.3.3/src/compiler/checker.ts',
  // Real world app tsx (1.0M)
  'https://raw.githubusercontent.com/oxc-project/benchmark-files/main/cal.com.tsx',
  // Real world content-heavy app jsx (3K)
  'https://raw.githubusercontent.com/oxc-project/benchmark-files/main/RadixUIAdoptionSection.jsx',
  // Heavy with classes (554K)
  'https://cdn.jsdelivr.net/npm/pdfjs-dist@4.0.269/build/pdf.mjs',
  // ES5 (3.9M)
  'https://cdn.jsdelivr.net/npm/antd@5.12.5/dist/antd.js',
];

const benchFixtures = await Promise.all(benchFixtureUrls.map(async (url) => {
  const filename = url.split('/').at(-1),
    path = pathJoin(TARGET_DIR_PATH, filename);
  let sourceText;
  try {
    sourceText = await readFile(path, 'utf8');
  } catch {
    const res = await fetch(url);
    sourceText = await res.text();
    await writeFile(path, sourceText);
  }

  return [filename, sourceText];
}));

// Only test Test262 fixtures which Acorn is able to parse
const test262FixturePaths = (await readdir(ACORN_TEST262_DIR_PATH, { recursive: true }))
  .filter(path => path.endsWith('.json'))
  .map(path => path.slice(0, -2));

// Test raw transfer output matches standard (via JSON) output for some large files
describe('fixtures', () => {
  it.each(benchFixtures)('%s', testRaw, { timeout: 10000 });
});

// Test raw transfer output matches standard (via JSON) output for Test262 test cases
describe('test262', () => {
  it.each(test262FixturePaths)('%s', async (path) => {
    const filename = basename(path);
    const sourceText = await readFile(pathJoin(TEST262_DIR_PATH, path), 'utf8');
    testRaw(filename, sourceText);
  });
});

function testRaw(filename, sourceText) {
  const retStandard = parseSync(filename, sourceText);
  const { program: programStandard, comments: commentsStandard, module: moduleStandard, errors: errorsStandard } =
    retStandard;

  // @ts-ignore
  const retRaw = parseSync(filename, sourceText, { experimentalRawTransfer: true });
  const { program: programRaw, comments: commentsRaw } = retRaw;
  // Remove `null` values, to match what NAPI-RS does
  const moduleRaw = clean(retRaw.module);
  const errorsRaw = clean(retRaw.errors);

  // Compare as objects
  expect(programRaw).toEqual(programStandard);
  expect(commentsRaw).toEqual(commentsStandard);
  expect(moduleRaw).toEqual(moduleStandard);
  expect(errorsRaw).toEqual(errorsStandard);

  // Compare as JSON (to ensure same field order)
  const jsonStandard = stringify({
    program: programStandard,
    comments: commentsStandard,
    module: moduleStandard,
    errors: errorsStandard,
  });
  const jsonRaw = stringify({ program: programRaw, comments: commentsRaw, module: moduleRaw, errors: errorsRaw });
  expect(jsonRaw).toEqual(jsonStandard);
}

// Stringify to JSON, removing values which are invalid in JSON
function stringify(obj) {
  return JSON.stringify(obj, (_key, value) => {
    if (typeof value === 'bigint') return `__BIGINT__: ${value}`;
    if (typeof value === 'object' && value instanceof RegExp) return `__REGEXP__: ${value}`;
    if (value === Infinity) return `__INFINITY__`;
    return value;
  });
}

// Remove `null` values, to match what NAPI-RS does
function clean(obj) {
  return JSON.parse(JSON.stringify(obj, (_key, value) => value === null ? undefined : value));
}
