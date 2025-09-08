// Worker for raw transfer tests.

import { readFile } from 'node:fs/promises';
import { basename, join as pathJoin } from 'node:path';

import { parseSync } from '../index.mjs';
import {
  ACORN_TEST262_DIR_PATH,
  JSX_DIR_PATH,
  ROOT_DIR_PATH,
  TEST262_DIR_PATH,
  TEST_TYPE_FIXTURE,
  TEST_TYPE_INLINE_FIXTURE,
  TEST_TYPE_JSX,
  TEST_TYPE_LAZY,
  TEST_TYPE_MAIN_MASK,
  TEST_TYPE_PRETTY,
  TEST_TYPE_TEST262,
  TEST_TYPE_TS,
  TS_DIR_PATH,
  TS_ESTREE_DIR_PATH,
} from './parse-raw-common.mjs';
import { makeUnitsFromTest } from './typescript-make-units-from-test.mjs';

// Run test case and return whether it passes.
// This is the entry point when run as a worker.
export default async function(data) {
  try {
    await runCase(data, simpleExpect);
    return true;
  } catch {
    return false;
  }
}

// Run test case with specified `expect` implementation.
// If test fails, will throw an error.
// Can be called from main thread.
export async function runCase({ type, props }, expect) {
  const lazy = (type & TEST_TYPE_LAZY) !== 0,
    pretty = (type & TEST_TYPE_PRETTY) !== 0;
  type &= TEST_TYPE_MAIN_MASK;

  switch (type) {
    case TEST_TYPE_TEST262:
      await runTest262Case(props, lazy, expect);
      break;
    case TEST_TYPE_JSX:
      await runJsxCase(props, lazy, expect);
      break;
    case TEST_TYPE_TS:
      await runTsCase(props, lazy, expect);
      break;
    case TEST_TYPE_FIXTURE:
      await runFixture(props, lazy, pretty, expect);
      break;
    case TEST_TYPE_INLINE_FIXTURE:
      await runInlineFixture(props, lazy, pretty, expect);
      break;
    default:
      throw new Error('Unexpected test type');
  }
}

// Run Test262 test case
async function runTest262Case(path, lazy, expect) {
  const filename = basename(path);
  const [sourceText, acornJson] = await Promise.all([
    readFile(pathJoin(TEST262_DIR_PATH, path), 'utf8'),
    readFile(pathJoin(ACORN_TEST262_DIR_PATH, `${path}on`), 'utf8'),
  ]);

  const sourceType = getSourceTypeFromJSON(acornJson);

  if (lazy) {
    testLazy(filename, sourceText, { sourceType });
    return;
  }

  // @ts-ignore
  const { program } = parseSync(filename, sourceText, { sourceType, experimentalRawTransfer: true });
  const json = stringifyAcornTest262Style(program);
  expect(json).toEqual(acornJson);
}

// Run JSX test case
async function runJsxCase(filename, lazy, expect) {
  const sourcePath = pathJoin(JSX_DIR_PATH, filename),
    jsonPath = sourcePath.slice(0, -1) + 'on'; // `.jsx` -> `.json`
  const [sourceText, acornJson] = await Promise.all([
    readFile(sourcePath, 'utf8'),
    readFile(jsonPath, 'utf8'),
  ]);

  const sourceType = getSourceTypeFromJSON(acornJson);

  if (lazy) {
    testLazy(filename, sourceText, { sourceType });
    return;
  }

  // @ts-ignore
  const { program } = parseSync(filename, sourceText, { sourceType, experimentalRawTransfer: true });
  const json = stringifyAcornTest262Style(program);
  expect(json).toEqual(acornJson);
}

// Run TypeScript test case
const TS_CASE_HEADER = '__ESTREE_TEST__:PASS:\n```json\n';
const TS_CASE_FOOTER = '\n```\n';
const TS_CASE_FOOTER_LEN = TS_CASE_FOOTER.length;

async function runTsCase(path, lazy, expect) {
  const tsPath = path.slice(0, -3); // Trim off `.md`
  let [sourceText, casesJson] = await Promise.all([
    readFile(pathJoin(TS_DIR_PATH, tsPath), 'utf8'),
    readFile(pathJoin(TS_ESTREE_DIR_PATH, path), 'utf8'),
  ]);

  // Trim off UTF-8 BOM
  if (sourceText.charCodeAt(0) === 0xFEFF) sourceText = sourceText.slice(1);

  const { tests } = makeUnitsFromTest(tsPath, sourceText);
  const estreeJsons = casesJson.split(TS_CASE_HEADER)
    .slice(1)
    .map(part => part.slice(0, -TS_CASE_FOOTER_LEN));
  expect(estreeJsons.length).toEqual(tests.length);

  for (let i = 0; i < tests.length; i++) {
    const { name: filename, content: code, sourceType } = tests[i];

    const options = {
      sourceType: sourceType.module ? 'module' : 'unambiguous',
      astType: 'ts',
      preserveParens: false,
      experimentalRawTransfer: true,
    };

    if (lazy) {
      testLazy(filename, sourceText, options);
      continue;
    }

    // @ts-ignore
    const { program, errors } = parseSync(filename, code, options);
    const oxcJson = stringifyAcornTest262Style(program);

    const estreeJson = estreeJsons[i];

    try {
      expect(oxcJson).toEqual(estreeJson);
    } catch {
      // Fall back to comparing to AST parsed via JSON transfer.
      // We can fail to match the TS-ESLint snapshots where there are syntax errors,
      // because our parser is not recoverable.
      // @ts-ignore
      const standard = parseSync(filename, code, { ...options, experimentalRawTransfer: false });
      const standardJson = stringifyAcornTest262Style(standard.program);
      const errorsStandard = standard.errors;

      expect(oxcJson).toEqual(standardJson);

      const errorsRawJson = JSON.stringify(removeNullProperties(errors), null, 2);
      const errorsStandardJson = JSON.stringify(errorsStandard, null, 2);
      expect(errorsRawJson).toEqual(errorsStandardJson);
    }
  }
}

// Test raw transfer output matches standard (via JSON) output for a fixture file
async function runFixture(path, lazy, pretty, expect) {
  const filename = basename(path);
  const sourceText = await readFile(pathJoin(ROOT_DIR_PATH, path), 'utf8');

  if (lazy) {
    testLazy(filename, sourceText);
  } else {
    assertRawAndStandardMatch(filename, sourceText, pretty, expect);
  }
}

// Test raw transfer output matches standard (via JSON) output for a fixture, with provided source text
async function runInlineFixture({ filename, sourceText }, lazy, pretty, expect) {
  if (lazy) {
    testLazy(filename, sourceText);
  } else {
    assertRawAndStandardMatch(filename, sourceText, pretty, expect);
  }
}

// Test lazy deserialization does not throw an error.
// We don't test the correctness of the output.
function testLazy(filename, sourceText, options) {
  // @ts-ignore
  const ret = parseSync(filename, sourceText, {
    ...options,
    experimentalRawTransfer: false,
    experimentalLazy: true,
  });
  JSON.stringify(ret.program);
  JSON.stringify(ret.comments);
  JSON.stringify(ret.errors);
  JSON.stringify(ret.module);
}

// Assert raw transfer output matches standard (via JSON) output
function assertRawAndStandardMatch(filename, sourceText, pretty, expect) {
  const retStandard = parseSync(filename, sourceText);
  const { program: programStandard, comments: commentsStandard, module: moduleStandard, errors: errorsStandard } =
    retStandard;

  // Re-arrange fields to match raw transfer.
  // We don't want to change field order of the Rust structs, but want `start` and `end` last.
  // Field order doesn't matter much anyway for module record.
  moveStartAndEndToLast(moduleStandard.staticImports, true);
  moveStartAndEndToLast(moduleStandard.staticExports, true);
  moveStartAndEndToLast(moduleStandard.dynamicImports, false);

  // @ts-ignore
  const retRaw = parseSync(filename, sourceText, { experimentalRawTransfer: true });
  const { program: programRaw, comments: commentsRaw } = retRaw;
  // Remove `null` values, to match what NAPI-RS does
  const moduleRaw = removeNullProperties(retRaw.module);
  const errorsRaw = removeNullProperties(retRaw.errors);

  // Compare as JSON (to ensure same field order)
  const jsonStandard = stringify(
    { program: programStandard, comments: commentsStandard, module: moduleStandard, errors: errorsStandard },
    pretty,
  );
  const jsonRaw = stringify(
    { program: programRaw, comments: commentsRaw, module: moduleRaw, errors: errorsRaw },
    pretty,
  );
  expect(jsonRaw).toEqual(jsonStandard);
}

function moveStartAndEndToLast(arr, reorderEntries) {
  for (const obj of arr) {
    const { start, end } = obj;
    delete obj.start;
    delete obj.end;
    obj.start = start;
    obj.end = end;
    if (reorderEntries) moveStartAndEndToLast(obj.entries, false);
  }
}

// Acorn JSON files always end with:
// ```
//   "sourceType": "script",
//   "hashbang": null,
//   "start": 0,
//   "end": <some integer>,
// }
// ```
// For speed, extract `sourceType` with a slice, rather than parsing the JSON.
function getSourceTypeFromJSON(json) {
  const index = json.lastIndexOf('"sourceType": "');
  return json.slice(index + 15, index + 21);
}

// Stringify to JSON, replacing values which are invalid in JSON.
// If `pretty === true`, JSON is pretty-printed.
function stringify(obj, pretty) {
  return JSON.stringify(obj, (_key, value) => {
    if (typeof value === 'bigint') return `__BIGINT__: ${value}`;
    if (typeof value === 'object' && value instanceof RegExp) return `__REGEXP__: ${value}`;
    if (value === Infinity) return `__INFINITY__`;
    return value;
  }, pretty ? 2 : undefined);
}

// Stringify to JSON, removing values which are invalid in JSON,
// matching `acorn-test262` fixtures.
const INFINITY_PLACEHOLDER = '__INFINITY__INFINITY__INFINITY__';
const INFINITY_REGEXP = new RegExp(`"${INFINITY_PLACEHOLDER}"`, 'g');

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
function removeNullProperties(obj) {
  return JSON.parse(JSON.stringify(obj, (_key, value) => value === null ? undefined : value));
}

// Very simple `expect` implementation.
// Only supports `expect(x).toEqual(y)`, and uses only a simple `===` comparison.
// Therefore, only works for primitive values e.g. strings.
function simpleExpect(value) {
  return {
    toEqual(expected) {
      if (value !== expected) throw new Error('Mismatch');
    },
  };
}
