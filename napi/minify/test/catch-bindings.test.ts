import { runInNewContext } from "node:vm";
import { describe, expect, it } from "vitest";

import { minifySync } from "../index";

function captureOutput(code: string): unknown[][] {
  const output: unknown[][] = [];
  runInNewContext(code, { console: { log: (...args: unknown[]) => output.push(args) } });
  return output;
}

describe("catch bindings", () => {
  // https://github.com/oxc-project/oxc/issues/29076
  const cases = [
    [
      "nested catch with a same-named var",
      `try { throw "o"; } catch (x) {
        try { throw "i"; } catch (x) {
          var x = "v";
          console.log(x);
        }
      }`,
      [["v"]],
    ],
    [
      "references before and after the declaration and outside the catches",
      `try { throw "o"; } catch (x) {
        try { throw "i"; } catch (x) {
          console.log(x);
          var x = "v";
          console.log(x);
        }
        console.log(x);
      }
      console.log(x);`,
      [["i"], ["v"], ["o"], [undefined]],
    ],
    [
      "repeated var declarations",
      `try { throw "o"; } catch (x) {
        try { throw "i"; } catch (x) {
          var x = "v";
          console.log(x);
          var x = "w";
          console.log(x);
        }
        console.log(x);
      }`,
      [["v"], ["w"], ["o"]],
    ],
    [
      "destructuring in a nested block with a captured reference",
      `try { throw "o"; } catch (x) {
        try { throw "i"; } catch (x) {
          const read = () => x;
          { var { value: x } = { value: "v" }; }
          console.log(read(), x);
        }
        console.log(x);
      }`,
      [["v", "v"], ["o"]],
    ],
    [
      "three nested catches",
      `try { throw "o"; } catch (x) {
        try { throw "i"; } catch (x) {
          try { throw "j"; } catch (x) {
            var x = "v";
            console.log(x);
          }
          console.log(x);
        }
        console.log(x);
      }`,
      [["v"], ["i"], ["o"]],
    ],
    [
      "existing var binding outside the catches",
      `var x = "top";
      try { throw "o"; } catch (x) {
        try { throw "i"; } catch (x) {
          var x = "v";
          console.log(x);
        }
        console.log(x);
      }
      console.log(x);`,
      [["v"], ["o"], ["top"]],
    ],
    [
      "function scope inside the nested catch",
      `try { throw "o"; } catch (x) {
        try { throw "i"; } catch (x) {
          var x = "v";
          console.log((function () { var x = "f"; return x; })(), x);
        }
        console.log(x);
      }`,
      [["f", "v"], ["o"]],
    ],
  ] as const;

  for (const module of [false, true]) {
    for (const compress of [false, true]) {
      it.each(cases)(
        `preserves %s, module=${module}, compress=${compress}`,
        (_name, code, expected) => {
          const result = minifySync("input.js", code, {
            module,
            compress,
            mangle: true,
            codegen: { removeWhitespace: false },
          });
          expect(result.errors).toEqual([]);
          const directive = module ? '"use strict";' : "";
          expect(captureOutput(directive + code)).toEqual(expected);
          expect(captureOutput(directive + result.code)).toEqual(expected);
        },
      );
    }
  }
});
