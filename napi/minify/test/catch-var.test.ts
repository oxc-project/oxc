import { runInNewContext } from "node:vm";
import { describe, expect, it } from "vitest";

import { minifySync } from "../index";

function captureOutput(code: string): unknown[][] {
  const output: unknown[][] = [];
  runInNewContext(code, { console: { log: (...args: unknown[]) => output.push(args) } });
  return output;
}

describe("catch parameters redeclared by var", () => {
  const cases: [string, string, unknown[][]][] = [
    [
      "enclosing parameter capture",
      `(function(x) {
        function g(y) {
          try { throw 0 } catch (y) { var y = 2; console.log(y) }
          return x;
        }
        console.log(g(5));
      })(1);`,
      [[2], [1]],
    ],
    [
      "separate catch and outer values",
      `function f() {
        try { throw 0 } catch (y) { var y = 2; console.log(y) }
        console.log(y);
      } f();`,
      [[2], [undefined]],
    ],
    [
      "existing outer parameter",
      `function f(y) {
        try { throw 0 } catch (y) { var y = 2; console.log(y) }
        console.log(y);
      } f(5);`,
      [[2], [5]],
    ],
    [
      "nested catches",
      `function f() {
        try { throw 10 } catch (y) {
          try { throw 20 } catch (y) { var y = 3; console.log(y) }
          console.log(y);
        }
        console.log(y);
      } f();`,
      [[3], [10], [undefined]],
    ],
    [
      "repeated declarations in a nested block",
      `function f() {
        try { throw 0 } catch (y) {
          { var y = 1; var y = 2; console.log(y) }
        }
      } f();`,
      [[2]],
    ],
    [
      "shared outer binding",
      `function f() {
        try { throw 1 } catch (y) { var y = 2; console.log(y) }
        try { throw 3 } catch (y) { var y = 4; console.log(y) }
        console.log(y);
      } f();`,
      [[2], [4], [undefined]],
    ],
    [
      "unused catch value with an outer read",
      `function f() { try { throw 0 } catch (y) { var y = 2 } return y }
       console.log(f());`,
      [[undefined]],
    ],
    [
      "declaration hoisted from a dead catch",
      `function f() {
        if (false) { try {} catch (y) { var y = 2 } }
        return y;
      } console.log(f());`,
      [[undefined]],
    ],
    [
      "destructuring var initializer",
      `function f() {
        try { throw 0 } catch (y) { var [y] = [2]; console.log(y) }
        console.log(y);
      } f();`,
      [[2], [undefined]],
    ],
  ];

  for (const module of [false, true]) {
    for (const compress of [false, true]) {
      for (const mangle of [false, true]) {
        const label = `module=${module}, compress=${compress}, mangle=${mangle}`;
        it.each(cases)(`preserves %s with ${label}`, (_name, code, expected) => {
          const result = minifySync("input.js", code, { module, compress, mangle });
          expect(result.errors).toEqual([]);
          const strict = module ? '"use strict";' : "";
          expect(captureOutput(strict + code)).toEqual(expected);
          expect(captureOutput(strict + result.code)).toEqual(expected);
        });
      }
    }
  }
});
