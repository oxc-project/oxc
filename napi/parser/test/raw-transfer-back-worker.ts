// Worker for raw transfer back conformance tests.
//
// Mirror of `parse-raw-worker.ts`, with the round-trip oracle instead of JSON comparison:
// parse via raw transfer -> ESTree program -> encode back into arena -> native side asserts
// `ContentEq` + codegen equality against an independent direct parse of the same source.

import { readFile } from "node:fs/promises";
import { basename, join as pathJoin } from "node:path";

import { parseSync } from "./parser.ts";
import {
  ACORN_TEST262_DIR_PATH,
  JSX_DIR_PATH,
  ROOT_DIR_PATH,
  TEST262_DIR_PATH,
  TS_DIR_PATH,
} from "./parse-raw-common.ts";
import {
  TEST_TYPE_FIXTURE,
  TEST_TYPE_INLINE_FIXTURE,
  TEST_TYPE_JSX,
  TEST_TYPE_MAIN_MASK,
  TEST_TYPE_NO_SOURCE,
  TEST_TYPE_PRESERVE_PARENS,
  TEST_TYPE_TEST262,
  TEST_TYPE_TS,
} from "./raw-transfer-back-common.ts";
import { roundtrip } from "./raw-transfer-back-api.ts";
import { makeUnitsFromTest } from "./typescript-make-units-from-test.ts";

import type { ParserOptions, Program } from "./parser.ts";

type TestCaseProps = string | { filename: string; sourceText: string };

// Run test case and return whether it passes.
// This is the entry point when run as a worker.
export default async function (data: { type: number; props: TestCaseProps }): Promise<boolean> {
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
export async function runCase(
  { type, props }: { type: number; props: TestCaseProps },
  expect: ExpectFunction,
): Promise<void> {
  const noSource = (type & TEST_TYPE_NO_SOURCE) !== 0,
    preserveParens = (type & TEST_TYPE_PRESERVE_PARENS) !== 0;
  type &= TEST_TYPE_MAIN_MASK;

  switch (type) {
    case TEST_TYPE_TEST262:
      await runTest262Case(props as string, noSource, preserveParens, expect);
      break;
    case TEST_TYPE_JSX:
      await runJsxCase(props as string, noSource, preserveParens, expect);
      break;
    case TEST_TYPE_TS:
      await runTsCase(props as string, noSource, preserveParens, expect);
      break;
    case TEST_TYPE_FIXTURE: {
      const path = props as string;
      const sourceText = await readFile(pathJoin(ROOT_DIR_PATH, path), "utf8");
      roundtripCheck(basename(path), sourceText, null, noSource, preserveParens, expect);
      break;
    }
    case TEST_TYPE_INLINE_FIXTURE: {
      const { filename, sourceText } = props as { filename: string; sourceText: string };
      roundtripCheck(filename, sourceText, null, noSource, preserveParens, expect);
      break;
    }
    default:
      throw new Error("Unexpected test type");
  }
}

// Run Test262 test case
async function runTest262Case(
  path: string,
  noSource: boolean,
  preserveParens: boolean,
  expect: ExpectFunction,
): Promise<void> {
  const filename = basename(path);
  const [sourceText, acornJson] = await Promise.all([
    readFile(pathJoin(TEST262_DIR_PATH, path), "utf8"),
    readFile(pathJoin(ACORN_TEST262_DIR_PATH, `${path}on`), "utf8"),
  ]);
  const sourceType = getSourceTypeFromJSON(acornJson);
  roundtripCheck(filename, sourceText, { sourceType }, noSource, preserveParens, expect);
}

// Run JSX test case
async function runJsxCase(
  filename: string,
  noSource: boolean,
  preserveParens: boolean,
  expect: ExpectFunction,
): Promise<void> {
  const sourcePath = pathJoin(JSX_DIR_PATH, filename),
    jsonPath = sourcePath.slice(0, -1) + "on"; // `.jsx` -> `.json`
  const [sourceText, acornJson] = await Promise.all([
    readFile(sourcePath, "utf8"),
    readFile(jsonPath, "utf8"),
  ]);
  const sourceType = getSourceTypeFromJSON(acornJson);
  roundtripCheck(filename, sourceText, { sourceType }, noSource, preserveParens, expect);
}

// Run TypeScript test case.
// Same sub-case extraction as `parse-raw-worker.ts`. The TS-ESLint expected JSON is not needed;
// the oracle is the round trip itself.
async function runTsCase(
  path: string,
  noSource: boolean,
  preserveParens: boolean,
  expect: ExpectFunction,
): Promise<void> {
  const tsPath = path.slice(0, -3); // Trim off `.md`
  let sourceText = await readFile(pathJoin(TS_DIR_PATH, tsPath), "utf8");

  // Trim off UTF-8 BOM
  if (sourceText.charCodeAt(0) === 0xfeff) sourceText = sourceText.slice(1);

  const { tests } = makeUnitsFromTest(tsPath, sourceText);

  for (const { name: filename, content: code, sourceType } of tests) {
    const options: ParserOptions = {
      sourceType: sourceType.module ? "module" : "unambiguous",
      astType: "ts",
    };
    roundtripCheck(filename, code, options, noSource, preserveParens, expect);
  }
}

// Parse source via raw transfer, encode the ESTree program back into the arena,
// and assert codegen equality + `ContentEq` against an independent direct parse.
function roundtripCheck(
  filename: string,
  sourceText: string,
  options: ParserOptions | null,
  noSource: boolean,
  preserveParens: boolean,
  expect: ExpectFunction,
): void {
  const parseOptions: ParserOptions = {
    ...options,
    preserveParens,
    experimentalRawTransfer: true,
  };
  const { program, errors } = parseSync(filename, sourceText, parseOptions);

  // Our parser is not recoverable. Skip fixtures which fail to parse (fatal error
  // produces an empty program) - there is nothing meaningful to round-trip.
  if (errors.length > 0 && isEmptyProgram(program)) return;

  const astType = options?.astType === "ts" || /\.[mc]?tsx?$/.test(filename) ? "ts" : "js";
  const result = roundtrip(program, {
    filename,
    sourceText: noSource ? null : sourceText,
    astType,
    preserveParens,
  });

  // Codegen comparison first: on failure, the printed diff is the readable artifact.
  expect(result.printed).toEqual(result.expected);
  expect(result.contentEq).toBe(true);
}

function isEmptyProgram(program: Program): boolean {
  return program.start === 0 && program.end === 0 && program.body.length === 0;
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
function getSourceTypeFromJSON(json: string): "script" | "module" {
  const index = json.lastIndexOf('"sourceType": "');
  return json.slice(index + 15, index + 21) as "script" | "module";
}

// Type for expect function
interface ExpectFunction {
  (value: any): {
    toEqual: (expected: any) => void;
    toBe: (expected: any) => void;
  };
}

// Very simple `expect` implementation.
// Only supports `expect(x).toEqual(y)` and `expect(x).toBe(y)`, and both use only a simple `===` comparison.
// Therefore, only works for primitive values e.g. strings and booleans.
const simpleExpect: ExpectFunction = (value: any) => {
  const toBe = (expected: any): void => {
    if (value !== expected) throw new Error("Mismatch");
  };
  return { toEqual: toBe, toBe };
};
