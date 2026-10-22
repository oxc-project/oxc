import { transformSync, HelperMode } from './index.js';
import { readFileSync } from 'fs';
import { createRequire } from 'module';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);
const RUNTIME_HELPERS_DIR = join(__dirname, "../../npm/runtime/src/helpers");
const NEEDED_HELPERS = ["applyDecs2311", "applyDecs2203R", "checkInRHS", "toPropertyKey", "setFunctionName", "toPrimitive", "typeof"];
function loadBabelHelpers() {
  const helpers = {};
  for (const name of NEEDED_HELPERS) {
    helpers[name] = require(join(RUNTIME_HELPERS_DIR, `${name}.js`));
  }
  return helpers;
}
const babelHelpers = loadBabelHelpers();
if (!("metadata" in Symbol)) { Symbol.metadata = Symbol("Symbol.metadata"); }
if (!(Symbol.metadata in Function)) { Object.defineProperty(Function.prototype, Symbol.metadata, { value: null }); }

// Test 1: Class decorator only
const t1 = `var old; const dec = () => (cls, ctx) => { old = cls; }; @dec() class Foo {} assertEq(() => Foo, old);`
const result = transformSync("test.ts", t1, { sourceType: "script", target: "esnext", decorator: {}, helpers: { mode: HelperMode.External } });
console.log("Output:", result.code);

const orig = globalThis.babelHelpers;
globalThis.babelHelpers = babelHelpers;
try {
  const fn = new Function(`"use strict"; function assertEq(cb, expected) { var x = cb(); if (x === expected) return true; console.log('FAIL: x=' + x + ' expected=' + expected); return false; } ${result.code}`);
  fn();
  console.log("Result: OK");
} catch(e) {
  console.log("Error:", e.message);
} finally {
  if (orig === undefined) delete globalThis.babelHelpers; else globalThis.babelHelpers = orig;
}
