import { runInNewContext } from "node:vm";
import { expect, it } from "vitest";

import { minifySync } from "../index";

const cases = [
  'globalThis.result = `${z}$` + "{a}\\uD800";',
  'globalThis.result = `$${"{a}\\uD800"}${z}`;',
  'globalThis.result = `$${""}{a}${z}`;',
  "globalThis.result = `${z}$` + `{a}${w}`;",
  'globalThis.result = `${z}\\0` + "1\\uD800";',
  "globalThis.result = `${z}\\0` + `1${w}`;",
  'globalThis.result = `${z}\\0${""}1${w}`;',
  'globalThis.result = `${z}\\\\0` + "1\\uD800";',
  'globalThis.result = `${z}\\$` + "{a}\\uD800";',
  'var a=["before"]; function b(){a[0]="after";return "!"} globalThis.result="\\uD800".concat(a,b());',
  'var a=["before"]; function b(){a[0]="after";return "!"} globalThis.result="\\uD800".concat(a,b()).concat("tail");',
  'var a=["before"]; function b(){a[0]="after";return "!"} globalThis.result="\\uD800".concat(a).concat(b());',
  'var a=["before"]; Object.defineProperty(globalThis,"b",{get(){a[0]="after";return "!"}}); globalThis.result="\\uD800".concat(a,b);',
  'var a=["before"]; var p={x:a,get y(){a[0]="after";return "!"}}; globalThis.result="translate(".concat(p.x,", ").concat(p.y,")");',
  'var a=["before"]; Object.defineProperty(globalThis,"b",{get(){a[0]="after";return "!"}}); globalThis.result="\\uD800".concat(a).concat(b);',
  'var a=["before"]; function b(){a[0]="after";return "!"} globalThis.result="".concat(a).concat(b()).concat(a).concat(a).concat(a);',
  'var a=["before"]; function b(){a[0]="after";return "!"} globalThis.result="".concat(a,b()).concat(a);',
];

function evaluate(source: string) {
  const context = { z: "Z", w: "W", a: "INJECTED", result: undefined };
  runInNewContext(source, context, { timeout: 1000 });
  return context.result;
}

it.each(cases)("preserves string-folding runtime behavior: %s", (source) => {
  const expected = evaluate(source);
  for (const module of [false, true]) {
    for (const mangle of [false, true]) {
      const options = { module, mangle };
      const result = minifySync("test.js", source, options);
      expect(result.errors).toEqual([]);
      expect(evaluate(result.code)).toBe(expected);
      expect(minifySync("test.js", result.code, options).code).toBe(result.code);
    }
  }
});
