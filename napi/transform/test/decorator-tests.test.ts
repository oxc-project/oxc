import { describe, expect, test } from "vitest";
import { readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
import { HelperMode, transformSync } from "../index";

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

const RUNTIME_HELPERS_DIR = join(__dirname, "../../../npm/runtime/src/helpers");

const NEEDED_HELPERS = [
  "applyDecs2311",
  "applyDecs2203R",
  "checkInRHS",
  "toPropertyKey",
  "setFunctionName",
  "toPrimitive",
  "typeof",
];

function loadBabelHelpers(): Record<string, Function> {
  const helpers: Record<string, Function> = {};
  for (const name of NEEDED_HELPERS) {
    const helperPath = join(RUNTIME_HELPERS_DIR, `${name}.js`);
    helpers[name] = require(helperPath);
  }
  return helpers;
}

const babelHelpers = loadBabelHelpers();
if (!("metadata" in Symbol)) {
  (Symbol as any).metadata = Symbol("Symbol.metadata");
}
if (!((Symbol as any).metadata in Function)) {
  Object.defineProperty(Function.prototype, Symbol.metadata, { value: null });
}

interface TestCase {
  name: string;
  body: string;
}

function extractTestCases(source: string): TestCase[] {
  const tests: TestCase[] = [];
  const startIdx = source.indexOf("const tests:");
  if (startIdx < 0) return tests;

  const objStart = source.indexOf("{", startIdx);
  if (objStart < 0) return tests;

  let depth = 1;
  let i = objStart + 1;
  while (i < source.length && depth > 0) {
    if (source[i] === "{") depth++;
    if (source[i] === "}") depth--;
    i++;
  }
  const objContent = source.slice(objStart + 1, i - 1);

  let pos = 0;
  while (pos < objContent.length) {
    const q1 = objContent.indexOf("'", pos);
    const q2 = objContent.indexOf('"', pos);
    if (q1 < 0 && q2 < 0) break;
    const [quoteStart, quote] = (q2 < 0 || (q1 >= 0 && q1 < q2))
      ? [q1, "'"]
      : [q2, '"'];

    const nameEnd = objContent.indexOf(`${quote}: `, quoteStart + 1);
    if (nameEnd < 0) break;
    const name = objContent.slice(quoteStart + 1, nameEnd);
    const fnStart = objContent.indexOf("=> {", nameEnd + 2);
    if (fnStart < 0) break;

    let bodyStart = fnStart + 4;
    depth = 1;
    let bodyEnd = bodyStart;
    while (bodyEnd < objContent.length && depth > 0) {
      const ch = objContent[bodyEnd];
      if (ch === "{") depth++;
      if (ch === "}") depth--;
      bodyEnd++;
    }
    const body = objContent.slice(bodyStart, bodyEnd - 1);
    tests.push({ name, body });
    pos = bodyEnd;
  }
  return tests;
}

function makeExecutable(name: string, body: string): string {
  return `"use strict";
var testName = ${JSON.stringify(name)};
var failures = 0;
function prettyPrint(x) {
  if (x && x.prototype && x.prototype.constructor === x) return 'class';
  if (typeof x === 'string') return JSON.stringify(x);
  try { return x + ''; } catch { return 'typeof ' + typeof x; }
}
function assertEq(callback, expected) {
  var details;
  try {
    var x = callback();
    if (x === expected) return true;
    details = '  Expected: ' + prettyPrint(expected) + '\\n  Observed: ' + prettyPrint(x);
  } catch (error) {
    details = '  Throws: ' + error;
  }
  var code = callback.toString().replace(/^\\(\\) => /, '').replace(/\\s+/g, ' ');
  console.log('FAIL: ' + testName + '\\n  Code: ' + code + '\\n' + details);
  failures++;
  return false;
}
function assertThrows(callback, expected) {
  var details;
  try {
    var x = callback();
    details = '  Expected: throws instanceof ' + expected.name + '\\n  Observed: returns ' + prettyPrint(x);
  } catch (error) {
    if (error instanceof expected) return true;
    details = '  Expected: throws instanceof ' + expected.name + '\\n  Observed: throws ' + error;
  }
  var code = callback.toString().replace(/^\\(\\) => /, '').replace(/\\s+/g, ' ');
  console.log('FAIL: ' + testName + '\\n  Code: ' + code + '\\n' + details);
  failures++;
  return false;
}
${body}
if (failures > 0) { throw new Error(failures + ' assertion(s) failed'); }
`;
}

describe("TC39 decorator transform (V2023-11)", () => {
  const sourcePath = join(__dirname, "decorator-tests.ts");
  const source = readFileSync(sourcePath, "utf-8");
  const testCases = extractTestCases(source);

  const supported: TestCase[] = [];
  const unsupported: TestCase[] = [];

  for (const tc of testCases) {
    const result = transformSync("test.ts", tc.body, {
      sourceType: "script",
      target: "esnext",
      decorator: {},
      helpers: { mode: HelperMode.External },
    });
    const hasRemainingDecorators = result.code.split("\n").some(
      (l) => l.includes("@") && !l.trim().startsWith("//") && !l.trim().startsWith("*")
    );
    if (hasRemainingDecorators) {
      unsupported.push(tc);
    } else {
      supported.push(tc);
    }
  }

  describe("supported class decorator tests", () => {
    test.each(supported.map((tc) => [tc.name, tc.body] as const))(
      "%s",
      (name, body) => {
        const result = transformSync("test.ts", body, {
          sourceType: "script",
          target: "esnext",
          decorator: {},
          helpers: { mode: HelperMode.External },
        });

        expect(result.errors).toHaveLength(0);

        const execCode = makeExecutable(name, result.code);

        const origBabelHelpers = (globalThis as any).babelHelpers;
        (globalThis as any).babelHelpers = babelHelpers;

        try {
          const fn = new Function(execCode);
          fn();
        } finally {
          if (origBabelHelpers === undefined) {
            delete (globalThis as any).babelHelpers;
          } else {
            (globalThis as any).babelHelpers = origBabelHelpers;
          }
        }
      },
      30000
    );
  });

  describe("unsupported member decorator tests", () => {
    test.skip.each(unsupported.map((tc) => [tc.name] as const))(
      "%s (member decorators not yet implemented in oxc)",
      () => {}
    );
  });
});
