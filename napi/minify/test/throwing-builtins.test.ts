import { runInNewContext } from "node:vm";
import { expect, it } from "vitest";

import { minifySync } from "../index";

it.each([
  ["new ArrayBuffer(-1)", "RangeError"],
  ["new ArrayBuffer('-1')", "RangeError"],
  ["new ArrayBuffer(Infinity)", "RangeError"],
  ["new ArrayBuffer(-0.5)", "ok"],
  ["new ArrayBuffer(NaN)", "ok"],
  ["decodeURIComponent('%')", "URIError"],
  ["decodeURI('%ED%A0%80')", "URIError"],
  ['encodeURI("\\uD800")', "URIError"],
  ['encodeURIComponent("\\uDC00")', "URIError"],
  ['encodeURI("\\uD83D\\uDE00")', "ok"],
  ["decodeURIComponent('%F0%9F%98%80')", "ok"],
])("preserves exceptions from unused %s", (expression, expected) => {
  const source = `try { const unused = ${expression}; result = 'ok'; }
    catch (e) { result = e.name; }`;
  for (const mangle of [false, true]) {
    const options = { mangle };
    const { code, errors } = minifySync("test.js", source, options);
    expect(errors).toEqual([]);
    const context = { result: undefined };
    runInNewContext(code, context, { timeout: 1000 });
    expect(context.result).toBe(expected);
    expect(minifySync("test.js", code, options).code).toBe(code);
  }
});
