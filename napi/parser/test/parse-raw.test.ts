// Tests for raw transfer.

import { mkdir, readdir, readFile, stat, writeFile } from 'node:fs/promises';
import { basename, join as pathJoin } from 'node:path';
import Tinypool from 'tinypool';
import { describe, expect, it } from 'vitest';

import { parseAsync, parseSync, type TSTypeAliasDeclaration, type VariableDeclaration } from '../src-js/index.js';

import {
  ACORN_TEST262_DIR_PATH,
  JSX_DIR_PATH,
  JSX_SHORT_DIR_PATH,
  JSX_SNAPSHOT_PATH,
  ROOT_DIR_PATH,
  TARGET_DIR_PATH,
  TEST262_SHORT_DIR_PATH,
  TEST262_SNAPSHOT_PATH,
  TEST_TYPE_FIXTURE,
  TEST_TYPE_INLINE_FIXTURE,
  TEST_TYPE_JSX,
  TEST_TYPE_LAZY,
  TEST_TYPE_PRETTY,
  TEST_TYPE_RANGE_PARENT,
  TEST_TYPE_TEST262,
  TEST_TYPE_TS,
  TS_ESTREE_DIR_PATH,
  TS_SHORT_DIR_PATH,
  TS_SNAPSHOT_PATH,
} from './parse-raw-common';

const { env } = process;
const isEnabled = envValue => envValue === 'true' || envValue === '1';

const [describeRangeParent, itRangeParent] = isEnabled(env.RUN_RAW_RANGE_TESTS)
  ? [describe, it]
  : (noop => [noop, noop])(Object.assign(() => {}, { concurrent() {} }));

const [describeLazy, itLazy] = isEnabled(env.RUN_LAZY_TESTS)
  ? [describe, it]
  : (noop => [noop, noop])(Object.assign(() => {}, { concurrent() {} }));

// Worker pool for running test cases.
// Vitest provides parallelism across test files, but not across cases within a single test file.
// So we run each case in a worker to achieve parallelism.
const pool = new Tinypool({ filename: new URL('./parse-raw-worker.ts', import.meta.url).href });

let runCase;

// Run test case in a worker
async function runCaseInWorker(type, props) {
  const success = await pool.run({ type, props });

  // If test failed in worker, run it again in main thread with Vitest's `expect`,
  // to get a nice diff and stack trace
  if (!success) {
    if (!runCase) ({ runCase } = await import('./parse-raw-worker.ts'));

    type |= TEST_TYPE_PRETTY;
    await runCase({ type, props }, expect);
    throw new Error('Failed on worker but unexpectedly passed on main thread');
  }
}

// Download fixtures.
// Save in `target` directory, same as where benchmarks store them.
let benchFixtureUrls = [
  // TypeScript syntax (2.81MB)
  'https://cdn.jsdelivr.net/gh/microsoft/TypeScript@v5.3.3/src/compiler/checker.ts',
  // Real world app tsx (1.0M)
  'https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@main/cal.com.tsx',
  // Real world content-heavy app jsx (3K)
  'https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@main/RadixUIAdoptionSection.jsx',
  // Heavy with classes (554K)
  'https://cdn.jsdelivr.net/npm/pdfjs-dist@4.0.269/build/pdf.mjs',
  // ES5 (3.9M)
  'https://cdn.jsdelivr.net/npm/antd@4.16.1/dist/antd.js',
];

// `antd.js` tests sometimes take longer than 5 secs on CI, so skip that fixture in CI.
// Tried setting `{ timeout: 10_000 }` option, but it didn't work for some reason.
// TODO: Get longer timeout working and re-enable these tests in CI.
if (process.env.CI) benchFixtureUrls = benchFixtureUrls.filter(url => !url.endsWith('/antd.js'));

await mkdir(TARGET_DIR_PATH, { recursive: true });

const benchFixturePaths = await Promise.all(benchFixtureUrls.map(async (url) => {
  const filename = url.split('/').at(-1),
    path = pathJoin(TARGET_DIR_PATH, filename);
  try {
    await stat(path);
  } catch {
    const res = await fetch(url);
    const sourceText = await res.text();
    await writeFile(path, sourceText);
  }
  return path.slice(ROOT_DIR_PATH.length + 1);
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

describe.concurrent('test262', () => {
  // oxlint-disable-next-line jest/expect-expect
  it.each(test262FixturePaths)('%s', path => runCaseInWorker(TEST_TYPE_TEST262, path));
});

describeRangeParent.concurrent('range & parent test262', () => {
  // oxlint-disable-next-line jest/expect-expect
  it.each(test262FixturePaths)('%s', path => runCaseInWorker(TEST_TYPE_TEST262 | TEST_TYPE_RANGE_PARENT, path));
});

// Check lazy deserialization doesn't throw
describeLazy.concurrent('lazy test262', () => {
  // oxlint-disable-next-line jest/expect-expect
  it.each(test262FixturePaths)('%s', path => runCaseInWorker(TEST_TYPE_TEST262 | TEST_TYPE_LAZY, path));
});

// Test raw transfer output matches JSON snapshots for Acorn-JSX test cases.
//
// Only test Acorn-JSX fixtures which Acorn is able to parse.
// Skip tests which we know we can't pass (listed as failing in `estree_acorn_jsx.snap` snapshot file).
const jsxFailPaths = await getTestFailurePaths(JSX_SNAPSHOT_PATH, JSX_SHORT_DIR_PATH);
const jsxFixturePaths = (await readdir(JSX_DIR_PATH, { recursive: true }))
  .filter(path => path.endsWith('.jsx') && !jsxFailPaths.has(path));

describe.concurrent('JSX', () => {
  // oxlint-disable-next-line jest/expect-expect
  it.each(jsxFixturePaths)('%s', filename => runCaseInWorker(TEST_TYPE_JSX, filename));
});

describeRangeParent.concurrent('range & parent JSX', () => {
  // oxlint-disable-next-line jest/expect-expect
  it.each(jsxFixturePaths)('%s', filename => runCaseInWorker(TEST_TYPE_JSX | TEST_TYPE_RANGE_PARENT, filename));
});

// Check lazy deserialization doesn't throw
describeLazy.concurrent('lazy JSX', () => {
  // oxlint-disable-next-line jest/expect-expect
  it.each(jsxFixturePaths)('%s', filename => runCaseInWorker(TEST_TYPE_JSX | TEST_TYPE_LAZY, filename));
});

// Test raw transfer output matches JSON snapshots for TypeScript test cases.
//
// Only test TypeScript fixtures which TS-ESLint is able to parse.
// Skip tests which we know we can't pass (listed as failing in `estree_typescript.snap` snapshot file).
//
// Where output does not match snapshot, fallback to comparing to "standard" transfer method instead.
// We can fail to match the TS-ESLint snapshots where there are syntax errors, because our parser
// is not recoverable.
const tsFailPaths = await getTestFailurePaths(TS_SNAPSHOT_PATH, TS_SHORT_DIR_PATH);
const tsFixturePaths = (await readdir(TS_ESTREE_DIR_PATH, { recursive: true }))
  .filter(path => path.endsWith('.md') && !tsFailPaths.has(path.slice(0, -3)));

describe.concurrent('TypeScript', () => {
  // oxlint-disable-next-line jest/expect-expect
  it.each(tsFixturePaths)('%s', path => runCaseInWorker(TEST_TYPE_TS, path));
});

describeRangeParent.concurrent('range & parent TypeScript', () => {
  // oxlint-disable-next-line jest/expect-expect
  it.each(tsFixturePaths)('%s', path => runCaseInWorker(TEST_TYPE_TS | TEST_TYPE_RANGE_PARENT, path));
});

// Check lazy deserialization doesn't throw
describeLazy.concurrent('lazy TypeScript', () => {
  // oxlint-disable-next-line jest/expect-expect
  it.each(tsFixturePaths)('%s', path => runCaseInWorker(TEST_TYPE_TS | TEST_TYPE_LAZY, path));
});

// Test raw transfer output matches standard (via JSON) output for edge cases not covered by Test262
describe.concurrent('edge cases', () => {
  describe.each([
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
    // oxlint-disable-next-line jest/expect-expect
    it('JS', () => runCaseInWorker(TEST_TYPE_INLINE_FIXTURE, { filename: 'dummy.js', sourceText }));
    // oxlint-disable-next-line jest/expect-expect
    it('TS', () => runCaseInWorker(TEST_TYPE_INLINE_FIXTURE, { filename: 'dummy.ts', sourceText }));

    itRangeParent(
      'JS range & parent',
      () => runCaseInWorker(TEST_TYPE_INLINE_FIXTURE | TEST_TYPE_RANGE_PARENT, { filename: 'dummy.js', sourceText }),
    );
    itRangeParent(
      'TS range & parent',
      () => runCaseInWorker(TEST_TYPE_INLINE_FIXTURE | TEST_TYPE_RANGE_PARENT, { filename: 'dummy.ts', sourceText }),
    );

    itLazy(
      'JS lazy',
      () => runCaseInWorker(TEST_TYPE_INLINE_FIXTURE | TEST_TYPE_LAZY, { filename: 'dummy.js', sourceText }),
    );
    itLazy(
      'TS lazy',
      () => runCaseInWorker(TEST_TYPE_INLINE_FIXTURE | TEST_TYPE_LAZY, { filename: 'dummy.ts', sourceText }),
    );
  });
});

// Test raw transfer output matches standard (via JSON) output for some large files
describe.concurrent('fixtures', () => {
  // oxlint-disable-next-line jest/expect-expect
  it.each(benchFixturePaths)('%s', path => runCaseInWorker(TEST_TYPE_FIXTURE, path));
});

describeRangeParent.concurrent('range & parent fixtures', () => {
  // oxlint-disable-next-line jest/expect-expect
  it.each(benchFixturePaths)('%s', path => runCaseInWorker(TEST_TYPE_FIXTURE | TEST_TYPE_RANGE_PARENT, path));
});

// Check lazy deserialization doesn't throw
describeLazy.concurrent('lazy fixtures', () => {
  // oxlint-disable-next-line jest/expect-expect
  it.each(benchFixturePaths)('%s', path => runCaseInWorker(TEST_TYPE_FIXTURE | TEST_TYPE_LAZY, path));
});

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

describe.concurrent('`parseAsync`', () => {
  it('matches `parseSync`', async () => {
    const path = benchFixturePaths[0],
      filename = basename(path),
      sourceText = await readFile(pathJoin(ROOT_DIR_PATH, path), 'utf8');
    const programStandard = parseSync(filename, sourceText).program;
    // @ts-ignore
    const programRaw = (await parseAsync(filename, sourceText, { experimentalRawTransfer: true })).program;
    expect(programRaw).toEqual(programStandard);
  });

  // oxlint-disable-next-line jest/expect-expect
  it('processes multiple files', async () => {
    await testMultiple(4);
  });

  // This is primarily testing the queuing mechanism.
  // At least on Mac OS, this test does not cause out-of-memory without the queue implemented,
  // but the test doesn't complete in a reasonable time (I gave up waiting after 20 minutes).
  // oxlint-disable-next-line jest/expect-expect
  it('does not exhaust memory when called huge number of times in succession', async () => {
    await testMultiple(10_000);
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

it.concurrent('checks semantic', async () => {
  const code = 'let x; let x;';

  // @ts-ignore
  let ret = parseSync('test.js', code, { experimentalRawTransfer: true });
  expect(ret.errors.length).toBe(0);

  // @ts-ignore
  ret = parseSync('test.js', code, { experimentalRawTransfer: true, showSemanticErrors: true });
  expect(ret.errors.length).toBe(1);
});

describe.concurrent('`preserveParens` option', () => {
  describe.concurrent('should not include parens when false', () => {
    it.concurrent('JS', async () => {
      const code = 'let x = (1 + 2);';

      // @ts-ignore
      let ret = parseSync('test.js', code, { experimentalRawTransfer: true, preserveParens: false });
      expect(ret.errors.length).toBe(0);
      const firstStatement = ret.program.body[0] as VariableDeclaration;
      expect(firstStatement.declarations[0].init.type).toBe('BinaryExpression');
    });

    it.concurrent('TS', async () => {
      const code = 'let x = (1 + 2); type T = (string);';

      // @ts-ignore
      let ret = parseSync('test.ts', code, { experimentalRawTransfer: true, preserveParens: false });
      expect(ret.errors.length).toBe(0);
      const firstStatement = ret.program.body[0] as VariableDeclaration;
      expect(firstStatement.declarations[0].init.type).toBe('BinaryExpression');
      const secondStatement = ret.program.body[1] as TSTypeAliasDeclaration;
      expect(secondStatement.typeAnnotation.type).toBe('TSStringKeyword');
    });
  });

  describe.concurrent('should include parens when true', () => {
    it.concurrent('JS', async () => {
      const code = 'let x = (1 + 2);';

      // @ts-ignore
      let ret = parseSync('test.js', code, { experimentalRawTransfer: true, preserveParens: true });
      expect(ret.errors.length).toBe(0);
      const firstStatement = ret.program.body[0] as VariableDeclaration;
      expect(firstStatement.declarations[0].init.type).toBe('ParenthesizedExpression');
    });

    it.concurrent('TS', async () => {
      const code = 'let x = (1 + 2); type T = (string);';

      // @ts-ignore
      let ret = parseSync('test.ts', code, { experimentalRawTransfer: true, preserveParens: true });
      expect(ret.errors.length).toBe(0);
      const firstStatement = ret.program.body[0] as VariableDeclaration;
      expect(firstStatement.declarations[0].init.type).toBe('ParenthesizedExpression');
      const secondStatement = ret.program.body[1] as TSTypeAliasDeclaration;
      expect(secondStatement.typeAnnotation.type).toBe('TSParenthesizedType');
    });
  });
});

describe.concurrent('`range` option', () => {
  describe.concurrent('should not include `range` fields when false', () => {
    it.concurrent('JS', async () => {
      const code = 'let x = 1;';

      // @ts-ignore
      let ret = parseSync('test.js', code, { experimentalRawTransfer: true, range: false });
      expect(ret.errors.length).toBe(0);
      const { program } = ret;
      expect(program).not.toHaveProperty('range');
      const stmt = ret.program.body[0] as VariableDeclaration;
      expect(stmt).not.toHaveProperty('range');
      const declaration = stmt.declarations[0];
      expect(declaration).not.toHaveProperty('range');
      expect(declaration.id).not.toHaveProperty('range');
      expect(declaration.init).not.toHaveProperty('range');
    });

    it.concurrent('TS', async () => {
      const code = 'let x = 1;';

      // @ts-ignore
      let ret = parseSync('test.ts', code, { experimentalRawTransfer: true, range: false });
      expect(ret.errors.length).toBe(0);
      const { program } = ret;
      expect(program).not.toHaveProperty('range');
      const stmt = ret.program.body[0] as VariableDeclaration;
      expect(stmt).not.toHaveProperty('range');
      const declaration = stmt.declarations[0];
      expect(declaration).not.toHaveProperty('range');
      expect(declaration.id).not.toHaveProperty('range');
      expect(declaration.init).not.toHaveProperty('range');
    });
  });

  describe.concurrent('should include `range` fields when true', () => {
    it.concurrent('JS', async () => {
      const code = 'let x = 1;';

      // @ts-ignore
      let ret = parseSync('test.js', code, { experimentalRawTransfer: true, range: true });
      expect(ret.errors.length).toBe(0);
      const { program } = ret;
      expect(program.range).toEqual([0, 10]);
      const stmt = ret.program.body[0] as VariableDeclaration;
      expect(stmt.range).toEqual([0, 10]);
      const declaration = stmt.declarations[0];
      expect(declaration.range).toEqual([4, 9]);
      expect(declaration.id.range).toEqual([4, 5]);
      expect(declaration.init.range).toEqual([8, 9]);
    });

    it.concurrent('TS', async () => {
      const code = 'let x = 1;';

      // @ts-ignore
      let ret = parseSync('test.ts', code, { experimentalRawTransfer: true, range: true });
      expect(ret.errors.length).toBe(0);
      const { program } = ret;
      expect(program.range).toEqual([0, 10]);
      const stmt = ret.program.body[0] as VariableDeclaration;
      expect(stmt.range).toEqual([0, 10]);
      const declaration = stmt.declarations[0];
      expect(declaration.range).toEqual([4, 9]);
      expect(declaration.id.range).toEqual([4, 5]);
      expect(declaration.init.range).toEqual([8, 9]);
    });
  });
});

describe.concurrent('`experimentalParent` option', () => {
  describe.concurrent('should not include `parent` fields when false', () => {
    it.concurrent('JS', async () => {
      const code = 'let x = 1;';

      // @ts-ignore
      let ret = parseSync('test.js', code, { experimentalRawTransfer: true, experimentalParent: false });
      expect(ret.errors.length).toBe(0);
      const { program } = ret;
      expect(program).not.toHaveProperty('parent');
      const stmt = ret.program.body[0] as VariableDeclaration;
      expect(stmt).not.toHaveProperty('parent');
      const declaration = stmt.declarations[0];
      expect(declaration).not.toHaveProperty('parent');
      expect(declaration.id).not.toHaveProperty('parent');
      expect(declaration.init).not.toHaveProperty('parent');
    });

    it.concurrent('TS', async () => {
      const code = 'let x = 1;';

      // @ts-ignore
      let ret = parseSync('test.ts', code, { experimentalRawTransfer: true, experimentalParent: false });
      expect(ret.errors.length).toBe(0);
      const { program } = ret;
      expect(program).not.toHaveProperty('parent');
      const stmt = ret.program.body[0] as VariableDeclaration;
      expect(stmt).not.toHaveProperty('parent');
      const declaration = stmt.declarations[0];
      expect(declaration).not.toHaveProperty('parent');
      expect(declaration.id).not.toHaveProperty('parent');
      expect(declaration.init).not.toHaveProperty('parent');
    });
  });

  describe.concurrent('should include `parent` fields when true', () => {
    it.concurrent('JS', async () => {
      const code = 'let x = 1;';

      // @ts-ignore
      let ret = parseSync('test.js', code, { experimentalRawTransfer: true, experimentalParent: true });
      expect(ret.errors.length).toBe(0);
      const { program } = ret;
      expect(program.parent).toBe(null);
      const stmt = ret.program.body[0] as VariableDeclaration;
      expect(stmt.parent).toBe(program);
      const declaration = stmt.declarations[0];
      expect(declaration.parent).toBe(stmt);
      expect(declaration.id.parent).toBe(declaration);
      expect(declaration.init.parent).toBe(declaration);
    });

    it.concurrent('TS', async () => {
      const code = 'let x = 1;';

      // @ts-ignore
      let ret = parseSync('test.ts', code, { experimentalRawTransfer: true, experimentalParent: true });
      expect(ret.errors.length).toBe(0);
      const { program } = ret;
      expect(program.parent).toBe(null);
      const stmt = ret.program.body[0] as VariableDeclaration;
      expect(stmt.parent).toBe(program);
      const declaration = stmt.declarations[0];
      expect(declaration.parent).toBe(stmt);
      expect(declaration.id.parent).toBe(declaration);
      expect(declaration.init.parent).toBe(declaration);
    });
  });
});
