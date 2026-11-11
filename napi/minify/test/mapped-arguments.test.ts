import { runInNewContext } from "node:vm";
import { describe, expect, it } from "vitest";

import { minifySync } from "../index";

function captureOutput(code: string): unknown[][] {
  const output: unknown[][] = [];
  runInNewContext(code, { console: { log: (...args: unknown[]) => output.push(args) } });
  return output;
}

describe("mapped arguments", () => {
  // https://github.com/oxc-project/oxc/issues/27473
  const cases = [
    ["direct write", "(function(a) { console.log(a = 'foo', arguments[0]); })('bar');"],
    ["var redeclaration", "(function(a) { var a; console.log(a = 'foo', arguments[0]); })('bar');"],
    [
      "initialized var redeclaration",
      "(function(a) { var a = 'baz'; console.log(a = 'foo', arguments[0]); })('bar');",
    ],
    [
      "function redeclaration",
      "(function(a) { function a() {} console.log(a = 'foo', arguments[0]); })('bar');",
    ],
    [
      "arguments alias",
      "(function(a) { var a; const args = arguments; console.log(a = 'foo', args[0]); })('bar');",
    ],
    [
      "captured write",
      "(function(a) { var a; (() => a = 'foo')(); console.log('foo', arguments[0]); })('bar');",
    ],
    [
      "captured write from strict scope",
      "(function(a) { var a; (function() { 'use strict'; a = 'foo'; })(); console.log('foo', arguments[0]); })('bar');",
    ],
    [
      "initializer write",
      "(function(a) { var a = 'foo'; console.log('foo', arguments[0]); })('bar');",
    ],
  ];

  const returnedArgumentsCode = `
    function f(a) {
      var a = function() {};
      return [arguments, a];
    }
    const result = f(1);
    console.log(typeof result[0][0], result[0][0] === result[1]);
  `;

  for (const mangle of [false, true]) {
    for (const filename of ["input.js", "input.cjs"]) {
      // Single-use substitution still runs with `unused: false`.
      for (const compress of [true, { unused: false }]) {
        const label = `${filename}, mangle=${mangle}, compress=${JSON.stringify(compress)}`;

        it.each(cases)(`preserves %s in ${label}`, (_name, code) => {
          const result = minifySync(filename, code, { mangle, compress });
          expect(result.errors).toEqual([]);
          expect(captureOutput(code)).toEqual([["foo", "foo"]]);
          expect(captureOutput(result.code)).toEqual([["foo", "foo"]]);
        });

        it(`preserves returned arguments in ${label}`, () => {
          const result = minifySync(filename, returnedArgumentsCode, { mangle, compress });
          expect(result.errors).toEqual([]);
          expect(captureOutput(returnedArgumentsCode)).toEqual([["function", true]]);
          expect(captureOutput(result.code)).toEqual([["function", true]]);
        });
      }
    }

    it.each(cases)(`preserves strict %s, mangle=${mangle}`, (_name, code) => {
      const result = minifySync("input.js", code, { module: true, mangle });
      expect(result.errors).toEqual([]);
      // These examples have no imports or exports; strict script execution
      // reproduces their module-mode arguments semantics.
      expect(captureOutput(`"use strict"; ${code}`)).toEqual([["foo", "bar"]]);
      expect(captureOutput(`"use strict"; ${result.code}`)).toEqual([["foo", "bar"]]);
    });

    it(`preserves strict returned arguments, mangle=${mangle}`, () => {
      const result = minifySync("input.js", returnedArgumentsCode, { module: true, mangle });
      expect(result.errors).toEqual([]);
      expect(captureOutput(`"use strict"; ${returnedArgumentsCode}`)).toEqual([["number", false]]);
      expect(captureOutput(`"use strict"; ${result.code}`)).toEqual([["number", false]]);
    });
  }
});
