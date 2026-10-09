import { describe, expect, it } from "vitest";
import { parseSync } from "oxc-parser";
import { codegen as rustPrint } from "oxc-codegen-conformance";
import { printSync } from "../dist/index.js";

const fixtures = [
  "const foo /* #__PURE__ */ = pureOperation();",
  "const foo /* @__NO_SIDE_EFFECTS__ */ = () => effect();",
  "a + (// c\n b ? c : d);",
  "a && (/* c */ b ? c : d);",
  "a || (// c\n b = c);",
  "f = () => (/* c */ { x: a });",
  "(/* c */ { x: a });",
  "(/* c */ class {});",
  "a && (b || c // c\n);",
  "a + (b ? c : d // c\n);",
  "a || (b = c // c\n);",
  "x = (a = 1, a // c\n);",
  "x = (a || b // c\n) && d;",
  "function f() { return a && (b || c && (d || e // c\n)); }",
  "x =\n/*a*/ (\n/*b*/ a\n).b;",
  "function f() { do; while (x); }",
  "function f() { return (\n// comment\na as any\n)``; }",
  "function *f() { yield (\n// comment\na as any\n)``; }",
  "const values = [/* sequence */ (a, b), /* function */ function () {}, /* arrow */ () => {}];",
  "const o = { [/* key */ (a, b)]: value }; class C { [/* key */ (a, b)]() {} }",
  "class C { #field; method(obj) { return /* binary */ (#field in obj) && obj; } }",
  "const f = /* @__NO_SIDE_EFFECTS__ */ (() => {}); new (/* @__PURE__ */ call())();",
  "const f = (/* function */ function () {})(); const a = (/* arrow */ () => {})();",
  "call(/* binary */ (a + b) * c, /* sequence */ (a, b));",
  "const view = <C cb={(value) => value}>{(value) => value}</C>;",
  "const { a, b // last\n} = obj; type M = { [K in keyof T]: T[K] // last\n};",
  "import {} from 'm'\n/* source end */;",
  "import { x } from 'm'\n/* source end */;",
  "import * as ns from 'm'\n/* source end */;",
  "import x from 'm'\n/* source end */;",
  "// only\n/* comments */",
  "/* leading */ f(); // tail\n/* eof */",
  "(/* string */ 'value' /* string end */);",
  "const a = [/* array */], b = { /* object */ }; new C(/* new */); f(/* call */);",
  "function f(/* param */ x /* params end */) { /* body */ }",
  "const o = { /* shorthand */ a, m(/* param */) { /* method */ } };",
  "const { /* binding */ a } = o; const { b: /* alias */ b } = o;",
  "import { /* import */ a as /* local */ b } from /* source */ 'mod'; export { /* export */ b /* end */ };",
  "import { /* empty */ } from 'mod'; export { /* export empty */ };",
  "import 'mod' with { /* attributes */ };",
  "import(/* webpackChunkName: 'chunk' */ 'mod' /* end */);",
  "class C extends /* super */ Base { /* member */ m(/* param */) { /* body */ } }",
  "try { /* try */ } catch (/* error */ err) { /* catch */ }",
  "const t = `a${/* before */ value /* after */}b`;",
  "const view = <C /* attr */ a={/* value */ x}>{/* child */}</C>;",
  "type T = { /* empty */ }; type U = [/* tuple */]; enum E { /* enum */ }",
  "const f = (/* empty */) => x; value /* postfix */ ++;",
  "interface I { /* body */ } type F = (/* params */ p: T) /* signature */ => T;",
  "const { /* key */ a = /* default */ 1 } = obj;",
  "const fragment = < /* open */ ></ /* close */>;",
  "switch (value) { case 1: /* empty case */ default: /* final case */ }",
  "for (/* init */ let i = 0; /* test */ i < 1; /* update */ i++) { /* body */ }",
  "if (/* test */ value /* test end */) first(); /* then */ else second();",
  "const o = { [/* key */ value /* key end */]: item /* property end */ };",
  "const o = { ...value /* spread end */ }; [ ...items /* spread end */ ];",
  "type T = (/* parenthesized */ U /* end */); type M = T[/* key */ K /* end */];",
  "type T = import(/* import type */ 'mod' /* end */).T;",
  "type M = { [K in keyof T /* key end */]: T[K] /* mapped end */ };",
  "function f() { return /* return */; } debugger /* debugger */;",
  "class C { [/* key */ value /* end */]() {} static { /* static */ } }",
  "function f(/* parameters */) { return /*\nleading\n*/ value; }",
  "function *f() { yield /*\nleading\n*/ value; throw (/*\nleading\n*/ error); }",
  "/* @__PURE__ */ call(); /* @__NO_SIDE_EFFECTS__ */ function f() {}",
  "const object = {/* @__KEY__ */ 'key': value};",
  "// v8 ignore file\n/* 🦀 */ function f() {\n /* multiline\n  * comment </ScRiPt>\n  */ value;\n}",
  "#!/usr/bin/env node\n// head\ncall(); // tail",
  "a /* tail */ + /* operand */ b + /* operand */ c;",
  "@dec class C { @dec method(/* params */) {} }",
];

/** Freeze metadata too: printing must never clear buckets or mark caller-owned objects. */
function freeze(value: unknown, seen = new Set<object>()): void {
  if (value === null || typeof value !== "object" || seen.has(value)) return;
  seen.add(value);
  for (const child of Object.values(value)) freeze(child, seen);
  Object.freeze(value);
}

describe("attached comments", () => {
  for (const source of fixtures) {
    it(`${source}`, () => {
      for (const preserveParens of [false, true]) {
        const expected = rustPrint("test.tsx", source, {
          lang: "tsx",
          preserveParens,
          comments: true,
        });
        expect(expected).not.toBeNull();
        if (expected === null) throw new Error("Invalid fixture");
        for (const experimentalRawTransfer of [false, true]) {
          const options = { preserveParens, attachComments: true, experimentalRawTransfer };
          const { program, errors } = parseSync("test.tsx", source, options);
          expect(errors).toEqual([]);
          freeze(program);
          expect(printSync(program, { ts: true, jsx: true, comments: true }).code).toBe(
            expected.code,
          );
          const mapped = printSync(program, {
            ts: true,
            jsx: true,
            comments: true,
            sourcemap: true,
            sourceFilename: "test.tsx",
            sourceText: source,
          });
          expect(mapped).toEqual(expected);
          expect(printSync(program, { ts: true, jsx: true, comments: true }).code).toBe(
            expected.code,
          );
        }
      }
    });
  }

  it.for([false, true])("prints moved owners in JS and TS builds (ts=%s)", (ts) => {
    const source = "/* first */ first();\n/* second */ second();";
    for (const experimentalRawTransfer of [false, true]) {
      const options = {
        lang: ts ? "ts" : "js",
        attachComments: true,
        experimentalRawTransfer,
      } as const;
      const { program } = parseSync("test.js", source, options);
      program.body.reverse();
      freeze(program);
      const expected = "/* second */ second();\n/* first */ first();\n";
      expect(printSync(program, { ts, comments: true }).code).toBe(expected);
      expect(
        printSync(program, { ts, comments: true, sourcemap: true, sourceText: source }).code,
      ).toBe(expected);
    }
  });

  it("accepts ASTs without comments and keeps comments opt-in", () => {
    const { program } = parseSync("test.js", "/* before */ call();", { attachComments: true });
    expect(printSync(program).code).toBe("call();\n");
    const plain = parseSync("test.js", "call();").program;
    expect(printSync(plain, { comments: true }).code).toBe("call();\n");
  });
});
