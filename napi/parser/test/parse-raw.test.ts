import { readdir, readFile, writeFile } from 'node:fs/promises';
import { basename, join as pathJoin } from 'node:path';
import { describe, expect, it } from 'vitest';

import { parseAsync, parseSync } from '../index.js';

const ROOT_DIR = pathJoin(import.meta.dirname, '../../..');
const TARGET_DIR_PATH = pathJoin(ROOT_DIR, 'target');
const TEST262_SHORT_DIR_PATH = 'tasks/coverage/test262/test';
const TEST262_DIR_PATH = pathJoin(ROOT_DIR, TEST262_SHORT_DIR_PATH);
const ACORN_TEST262_DIR_PATH = pathJoin(ROOT_DIR, 'tasks/coverage/acorn-test262/tests/test262/test');
const JSX_SHORT_DIR_PATH = 'tasks/coverage/acorn-test262/tests/acorn-jsx/pass';
const JSX_DIR_PATH = pathJoin(ROOT_DIR, JSX_SHORT_DIR_PATH);
const TEST262_SNAPSHOT_PATH = pathJoin(ROOT_DIR, 'tasks/coverage/snapshots/estree_test262.snap');
const JSX_SNAPSHOT_PATH = pathJoin(ROOT_DIR, 'tasks/coverage/snapshots/estree_acorn_jsx.snap');

const INFINITY_PLACEHOLDER = '__INFINITY__INFINITY__INFINITY__';
const INFINITY_REGEXP = new RegExp(`"${INFINITY_PLACEHOLDER}"`, 'g');

// Load/download fixtures.
// Save in `target` directory, same as where benchmarks store them.
//
// `checker.ts` and `cal.com.tsx` fixture tests are disabled for now while we work on aligning TS AST
// with TS-ESLint.
// TODO: Enable them again once that work is complete.
const benchFixtureUrls = [
  // TypeScript syntax (2.81MB)
  // 'https://cdn.jsdelivr.net/gh/microsoft/TypeScript@v5.3.3/src/compiler/checker.ts',
  // Real world app tsx (1.0M)
  // 'https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@main/cal.com.tsx',
  // Real world content-heavy app jsx (3K)
  'https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@main/RadixUIAdoptionSection.jsx',
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

// Test raw transfer output matches JSON snapshots for Test262 test cases.
//
// Only test Test262 fixtures which Acorn is able to parse.
// Skip tests which we know we can't pass (listed as failing in `estree_test262.snap` snapshot file),
// and skip tests related to hashbangs (where output is correct, but Acorn doesn't parse hashbangs).
const test262FailPaths = await getTestFailurePaths(TEST262_SNAPSHOT_PATH, TEST262_SHORT_DIR_PATH);
const test262FixturePaths = [];
for (let path of await readdir(ACORN_TEST262_DIR_PATH, { recursive: true })) {
  if (!path.endsWith('.json')) continue;
  path = path.slice(0, -2);
  if (test262FailPaths.has(path) || path.startsWith('language/comments/hashbang/')) continue;
  test262FixturePaths.push(path);
}

describe('test262', () => {
  it.each(test262FixturePaths)('%s', async (path) => {
    const filename = basename(path);
    const [sourceText, acornJson] = await Promise.all([
      readFile(pathJoin(TEST262_DIR_PATH, path), 'utf8'),
      readFile(pathJoin(ACORN_TEST262_DIR_PATH, `${path}on`), 'utf8'),
    ]);

    // Acorn JSON files always end with:
    // ```
    //   "sourceType": "script",
    //   "hashbang": null
    // }
    // ```
    // For speed, extract `sourceType` with a slice, rather than parsing the JSON.
    const sourceType = acornJson.slice(-29, -23);

    // @ts-ignore
    const { program } = parseSync(filename, sourceText, { sourceType, experimentalRawTransfer: true });
    const json = stringifyAcornTest262Style(program);
    expect(json).toEqual(acornJson);
  });
});

// Test raw transfer output matches JSON snapshots for Acorn-JSX test cases.
//
// Only test Acorn-JSX fixtures which Acorn is able to parse.
// Skip tests which we know we can't pass (listed as failing in `estree_acron_jsx.snap` snapshot file).
const jsxFailPaths = await getTestFailurePaths(JSX_SNAPSHOT_PATH, JSX_SHORT_DIR_PATH);
const jsxFixturePaths = (await readdir(JSX_DIR_PATH, { recursive: true }))
  .filter(path => path.endsWith('.jsx') && !jsxFailPaths.has(path));

describe('JSX', () => {
  it.each(jsxFixturePaths)('%s', async (filename) => {
    const sourcePath = pathJoin(JSX_DIR_PATH, filename),
      jsonPath = sourcePath.slice(0, -1) + 'on'; // `.jsx` -> `.json`
    const [sourceText, acornJson] = await Promise.all([
      readFile(sourcePath, 'utf8'),
      readFile(jsonPath, 'utf8'),
    ]);

    // Acorn JSON files always end with:
    // ```
    //   "sourceType": "script",
    //   "hashbang": null
    // }
    // ```
    // For speed, extract `sourceType` with a slice, rather than parsing the JSON.
    const sourceType = acornJson.slice(-29, -23);

    // @ts-ignore
    const { program } = parseSync(filename, sourceText, { sourceType, experimentalRawTransfer: true });
    const json = stringifyAcornTest262Style(program);
    expect(json).toEqual(acornJson);
  });
});

// Test raw transfer output matches standard (via JSON) output for edge cases not covered by Test262
describe('edge cases', () => {
  it.each([
    // ECMA stage 3
    'import defer * as ns from "x";',
    'import source src from "x";',
    'import.defer("x");',
    'import.source("x");',
    // `StringLiteral`s containing lone surrogates and/or lossy replacement characters
    ';"\\uD800\\uDBFF";',
    ';"�\\u{FFFD}";',
    ';"�\\u{FFFD}\\uD800\\uDBFF�\\u{FFFD}";',
    // `TemplateLiteral`s containing lone surrogates and/or lossy replacement characters
    '`\\uD800\\uDBFF${x}\\uD800\\uDBFF`;',
    '`�\\u{FFFD}${x}�\\u{FFFD}`;',
    '`�\\u{FFFD}\\uD800${x}\\uDBFF�\\u{FFFD}`;',
    // Hashbangs
    '#!/usr/bin/env node\nlet x;',
    '#!/usr/bin/env node\nlet x;\n// foo',
  ])('%s', (sourceText) => {
    assertRawAndStandardMatch('dummy.js', sourceText);
  });
});

// Test raw transfer output matches standard (via JSON) output for some large files
describe('fixtures', () => {
  it.each(benchFixtures)('%s', (filename, sourceText) => {
    assertRawAndStandardMatch(filename, sourceText);
  });
});

function assertRawAndStandardMatch(filename, sourceText) {
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

// Get `Set` containing test paths which failed from snapshot file
async function getTestFailurePaths(snapshotPath, pathPrefix) {
  const mismatchPrefix = `Mismatch: ${pathPrefix}/`,
    mismatchPrefixLen = mismatchPrefix.length;

  const snapshot = await readFile(snapshotPath, 'utf8');
  return new Set(
    snapshot.split('\n')
      .filter(line => line.startsWith(mismatchPrefix))
      .map(line => line.slice(mismatchPrefixLen)),
  );
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

// Stringify to JSON, removing values which are invalid in JSON,
// matching `acorn-test262` fixtures.
function stringifyAcornTest262Style(obj) {
  let containsInfinity = false;
  const json = JSON.stringify(obj, (_key, value) => {
    if (typeof value === 'bigint' || (typeof value === 'object' && value instanceof RegExp)) return null;
    if (value === Infinity) {
      containsInfinity = true;
      return INFINITY_PLACEHOLDER;
    }
    return value;
  }, 2);

  return containsInfinity ? json.replace(INFINITY_REGEXP, '1e+400') : json;
}

// Remove `null` values, to match what NAPI-RS does
function clean(obj) {
  return JSON.parse(JSON.stringify(obj, (_key, value) => value === null ? undefined : value));
}

describe('`parseAsync`', () => {
  it('matches `parseSync`', async () => {
    const [filename, sourceText] = benchFixtures[0];
    const programStandard = parseSync(filename, sourceText).program;
    // @ts-ignore
    const programRaw = (await parseAsync(filename, sourceText, { experimentalRawTransfer: true })).program;
    expect(programRaw).toEqual(programStandard);
  });

  it('processes multiple files', async () => {
    testMultiple(4);
  });

  // This is primarily testing the queuing mechanism.
  // At least on Mac OS, this test does not cause out-of-memory without the queue implemented,
  // but the test doesn't complete in a reasonable time (I gave up waiting after 20 minutes).
  it('does not exhaust memory when called huge number of times in succession', async () => {
    testMultiple(100_000);
  });

  async function testMultiple(iterations) {
    const promises = [];
    for (let i = 0; i < iterations; i++) {
      const code = `let x = ${i}`;
      // @ts-ignore
      promises.push(parseAsync('test.js', code, { experimentalRawTransfer: true }));
    }
    const results = await Promise.all(promises);

    for (let i = 0; i < iterations; i++) {
      const { program } = results[i];
      expect(program.body.length).toBe(1);
      expect(program.body[0].declarations[0].init.value).toBe(i);
    }
  }
});

it('checks semantic', async () => {
  const code = 'let x; let x;';

  // @ts-ignore
  let ret = parseSync('test.js', code, { experimentalRawTransfer: true });
  expect(ret.errors.length).toBe(0);

  // @ts-ignore
  ret = parseSync('test.js', code, { experimentalRawTransfer: true, showSemanticErrors: true });
  expect(ret.errors.length).toBe(1);
});
