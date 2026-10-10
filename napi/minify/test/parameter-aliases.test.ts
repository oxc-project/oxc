import { runInNewContext } from "node:vm";
import { describe, expect, it } from "vitest";

import { minifySync } from "../index";

function captureOutput(code: string): unknown[][] {
  const output: unknown[][] = [];
  runInNewContext(code, { console: { log: (...args: unknown[]) => output.push(args) } });
  return output;
}

describe("parameter alias observability", () => {
  const cases: [name: string, code: string, expected: unknown[][]][] = [
    [
      "parameter type",
      `function h(a, b) { var a = b; return typeof a; }
       const values = [0, 'x', undefined, {}, 1n, Symbol(), function() {},
         { [Symbol.toPrimitive]() { throw new Error('unexpected coercion'); } }];
       for (const value of values) console.log(h(1, value), typeof value);
       console.log(h(1), 'undefined');`,
      [
        ["number", "number"],
        ["string", "string"],
        ["undefined", "undefined"],
        ["object", "object"],
        ["bigint", "bigint"],
        ["symbol", "symbol"],
        ["function", "function"],
        ["object", "object"],
        ["undefined", "undefined"],
      ],
    ],
    [
      "parameter value",
      `function h(a, b) { var a = b; return a; }
       const values = [undefined, Symbol(), {}, function() {}, 1n];
       console.log(values.every(value => h(1, value) === value), h(1) === undefined);`,
      [[true, true]],
    ],
    [
      "duplicate parameters",
      `function h(a, b, b) { var a = b; return typeof a; }
       console.log(h(1, 0, 'x'), h(1, 'x', 0));`,
      [["string", "number"]],
    ],
    [
      "previously escaped arguments",
      `let args;
       function h(a, b) { args = arguments; var a = b; return typeof a; }
       console.log(h(1, 'x'), args[0]);`,
      [["string", "x"]],
    ],
    [
      "initializer call",
      `let args;
       function inspect(value) { args = value; return 'x'; }
       function h(a, b) { var a = inspect(arguments); return typeof a; }
       console.log(h(1, 'unused'), args[0]);`,
      [["string", "x"]],
    ],
    [
      "initializer closure",
      `function h(a, b) { var a = () => arguments[0]; return a; }
       const result = h(1, 'unused');
       console.log(result() === result);`,
      [[true]],
    ],
    [
      "escaped closure",
      `function h(a, b) { var a = b; return () => [arguments[0], a]; }
       const result = h(1, 'x')();
       console.log(result[0], result[1]);`,
      [["x", "x"]],
    ],
    [
      "write through arguments",
      `function h(a) { var a = 1; arguments[0] = 2; return a; }
       function k(a, b) { var args = arguments; var a = b; args[0] = 'y'; return a; }
       console.log(h(0), k(1, 'x'));`,
      [[2, "y"]],
    ],
    [
      "nested arrow",
      `function h(a, b) { var a = b; return (() => arguments[0])(); }
       function k(a, b) { var a = b; return () => () => arguments[0]; }
       console.log(h(1, 'x'), k(1, 'y')()());`,
      [["x", "y"]],
    ],
    [
      "nested function",
      `function h(a, b) { var a = b; return [function () { return arguments[0]; }, typeof a]; }
       const result = h(1, 'x');
       console.log(result[0]('y'), result[1]);`,
      [["y", "string"]],
    ],
    [
      "var arguments",
      `function h(a) { var arguments; var a = 1; return arguments[0]; }
       console.log(h(0));`,
      [[1]],
    ],
    [
      "outer arguments binding",
      `function h(arguments) { return function (a) { var a = 2; return arguments[0]; }; }
       console.log(h('outer')(1));`,
      [[2]],
    ],
    [
      "direct eval",
      `function h(a) { var a = 1; return eval('arguments[0]'); }
       function k(a) { var a = 1; return (() => eval('arguments[0]'))(); }
       console.log(h(0), k(0));`,
      [[1, 1]],
    ],
    [
      "dead arguments reference",
      `function f(a) { var a = 1; if (false) g(arguments); return h(a); }
       function h(a) { var a = a + 1; return [a, arguments[0]]; }
       const result = f(0);
       console.log(result[0], result[1]);`,
      [[2, 2]],
    ],
  ];

  for (const mangle of [false, true]) {
    for (const filename of ["input.js", "input.cjs"]) {
      it.each(cases)(`preserves %s in ${filename}, mangle=${mangle}`, (_name, code, expected) => {
        const result = minifySync(filename, code, { mangle });
        expect(result.errors).toEqual([]);
        expect(captureOutput(code)).toEqual(expected);
        expect(captureOutput(result.code)).toEqual(expected);
      });
    }
  }
});
