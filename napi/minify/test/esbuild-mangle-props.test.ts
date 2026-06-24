import { describe, expect, it } from "vitest";

import { minifySync } from "../index";

// Conformance tests ported from esbuild's property-mangling bundler tests:
//   esbuild/internal/bundler_tests/bundler_default_test.go
//
// These do NOT assert esbuild's literal output. esbuild's mangled names start at
// `a`/`b`/... and its codegen formatting differs from oxc's (oxc's base54 names
// start at `e`/`t`/`n`/...). Instead, each ported test feeds esbuild's INPUT to
// oxc-minify with the equivalent options and asserts oxc's ACTUAL output, which
// verifies that the CORRECT properties were renamed consistently and the right
// ones were reserved.
//
// Option mapping:
//   esbuild MangleProps: regexp("re")   -> oxc mangleProps: "re"
//   esbuild ReserveProps: regexp("re")  -> oxc reserveProps: "re"
//   esbuild MinifySyntax: true          -> oxc compress: true
//   esbuild MinifySyntax: false/unset   -> oxc compress: false (isolate pure renaming)
//   esbuild MinifyIdentifiers: true     -> oxc mangle: true
//
// Scope notes (oxc-minify v1):
//   * Single self-contained JS program. Multi-file/bundler & import/export tests
//     are out of scope -> it.skip with reason.
//   * JS-only. TypeScript-feature tests are out of scope -> it.skip.
//   * `mangleQuoted` is supported (`mangleQuoted: true`). By DEFAULT (the flag
//     off), a quoted property access (`o['_x']`) RESERVES the unquoted name `_x`
//     program-wide rather than mangling it (the safe terser-"strict" behavior).
//     With `mangleQuoted: true`, quoted keys in key/index positions (including
//     the wrapped conditional / comma forms esbuild handles) become candidates
//     and are renamed consistently with their unquoted siblings.
//   * `/* @__KEY__ */` / `/* #__KEY__ */` annotation comments ARE supported: a
//     string / no-substitution-template directly preceded by one is treated as a
//     property NAME to mangle even outside a key position (the regex still gates
//     it). A bare `/* __KEY__ */` (no `@`/`#`) is NOT an annotation.
//   * Syntax lowering (optional-chain / class-field lowering) is a transformer
//     concern, not minify; the non-lowered essence is covered separately.

describe("esbuild mangle-props conformance", () => {
  // ────────────────────────────────────────────────────────────────────────
  // Ported tests
  // ────────────────────────────────────────────────────────────────────────

  // Ported from bundler_default_test.go:6882 TestMangleProps
  // esbuild input (/entry2.js):
  //   export default { bar_: 0, 'baz_': 1 }
  // (entry1.js — the `shouldMangle`/`shouldNotMangle` pair — exercises the same
  // rule on a richer body; see the dedicated cases below.)
  // oxc: `bar_` matches `_$` -> renamed to `e`; the quoted `'baz_'` is a quoted
  // property, which oxc reserves (no mangleQuoted), so it is kept.
  it("TestMangleProps (entry2 default object)", () => {
    const r = minifySync(
      "entry2.js",
      `export default { bar_: 0, 'baz_': 1 }`,
      { mangleProps: "_$", compress: false, module: true },
    );
    expect(r.code).toBe(`export default{e:0,"baz_":1};`);
    expect(r.mangleCache).toEqual({ bar_: "e" });
  });

  // Ported from bundler_default_test.go:6882 TestMangleProps (entry1.js).
  // esbuild input — the `shouldMangle` function only (the original file also has
  // a `shouldNotMangle` twin that quotes every property; in oxc those quoted
  // names would reserve their unquoted forms program-wide, suppressing all
  // mangling, so we isolate the unquoted `shouldMangle` half):
  //   export function shouldMangle() {
  //     let foo = { bar_: 0, baz_() {} };
  //     let { bar_ } = foo;
  //     ({ bar_ } = foo);
  //     class foo_ { bar_ = 0; baz_() {}; static bar_ = 0; static baz_() {} }
  //     return { bar_, foo_ }
  //   }
  // oxc renames `baz_`->`e` and `foo_`->`t` (member, object-literal, class-field
  // and class-method positions all consistently). `bar_` is NOT mangled here:
  // it appears as a shorthand destructuring-ASSIGNMENT target `({ bar_ } = foo)`,
  // which oxc cannot rename without un-shorthanding an assignment pattern, so it
  // conservatively reserves `bar_` program-wide. The result is still consistent
  // (every `bar_` is preserved). This is an intentional, safe difference from
  // esbuild, which mangles these by tracking the bound local. See the dedicated
  // "shorthand assignment target reserves" case below.
  it("TestMangleProps (entry1 shouldMangle body)", () => {
    const r = minifySync(
      "entry1.js",
      `export function shouldMangle() {
  let foo = {
    bar_: 0,
    baz_() {},
  };
  let { bar_ } = foo;
  ({ bar_ } = foo);
  class foo_ {
    bar_ = 0
    baz_() {}
    static bar_ = 0
    static baz_() {}
  }
  return { bar_, foo_ }
}`,
      { mangleProps: "_$", compress: false, module: true },
    );
    expect(r.code).toBe(
      `export function shouldMangle(){let e={bar_:0,e(){}};let{bar_:t}=e;({bar_:t}=e);class n{bar_=0;e(){}static bar_=0;static e(){}}return{bar_:t,t:n}}`,
    );
    // baz_ and foo_ are mangled; bar_ is reserved (shorthand assignment target).
    expect(r.mangleCache).toEqual({ baz_: "e", foo_: "t" });
  });

  // Ported from bundler_default_test.go:6938 TestManglePropsMinify
  // Same input shape as TestMangleProps but with MinifySyntax + MinifyIdentifiers.
  // esbuild input (/entry1.js, shouldMangle half, with the long XXXX name for
  // frequency analysis):
  //   export function shouldMangle_XXXX...() {
  //     let foo = { bar_: 0, baz_() {} };
  //     let { bar_ } = foo;
  //     ({ bar_ } = foo);
  //     class foo_ { bar_ = 0; baz_() {}; static bar_ = 0; static baz_() {} }
  //     return { bar_, foo_ }
  //   }
  // Under compress, `baz_`->`e` and `foo_`->`t` are mangled consistently across
  // object-literal, class-field and class-method positions, while `bar_` stays
  // reserved (shorthand assignment target, as above). Locals get joined/renamed.
  it("TestManglePropsMinify", () => {
    const r = minifySync(
      "entry1.js",
      `export function shouldMangle_XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX() {
  let foo = {
    bar_: 0,
    baz_() {},
  };
  let { bar_ } = foo;
  ({ bar_ } = foo);
  class foo_ {
    bar_ = 0
    baz_() {}
    static bar_ = 0
    static baz_() {}
  }
  return { bar_, foo_ }
}`,
      { mangleProps: "_$", compress: true, mangle: true, module: true },
    );
    expect(r.code).toBe(
      `export function shouldMangle_XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX(){let e={bar_:0,e(){}},{bar_:t}=e;({bar_:t}=e);class n{bar_=0;e(){}static bar_=0;static e(){}}return{bar_:t,t:n}}`,
    );
    // baz_ and foo_ mangled; bar_ reserved (shorthand assignment target).
    expect(r.mangleCache).toEqual({ baz_: "e", foo_: "t" });
  });

  // Ported from bundler_default_test.go:6998 TestManglePropsKeywordPropertyMinify
  // esbuild input:
  //   class Foo { static bar = { get baz() { return 123 } } }
  // esbuild uses MangleProps: regexp(".") (mangle EVERYTHING) to verify mangled
  // names never collide with reserved keywords. oxc's base54 names start at `e`
  // and skip keywords by construction. We assert the substantive behavior: every
  // property (`bar`, and the GETTER `baz`) is renamed. compress:false so the
  // unreferenced class is not tree-shaken away.
  it("TestManglePropsKeywordPropertyMinify (getter prop mangled)", () => {
    const r = minifySync(
      "entry.js",
      `class Foo {
  static bar = { get baz() { return 123 } }
}`,
      { mangleProps: ".", compress: false },
    );
    expect(r.code).toBe(`class Foo{static e={get t(){return 123}}}`);
    expect(r.mangleCache).toEqual({ bar: "e", baz: "t" });
  });

  // Ported from bundler_default_test.go:7019 TestManglePropsOptionalChain
  // esbuild input:
  //   export default function(x) {
  //     x.foo_;
  //     x.foo_?.();
  //     x?.foo_;
  //     x?.foo_();
  //     x?.foo_.bar_;
  //     x?.foo_.bar_();
  //     x?.['foo_'].bar_;
  //     x?.foo_['bar_'];
  //   }
  // oxc renames `foo_`/`bar_` consistently across dot, optional-dot, and call
  // positions. The bracketed `['foo_']`/`['bar_']` are quoted accesses -> oxc
  // reserves them (kept as computed string keys, here printed as templates),
  // so those two sites are intentionally NOT renamed.
  it("TestManglePropsOptionalChain", () => {
    const r = minifySync(
      "entry.js",
      `export default function(x) {
  x.foo_;
  x.foo_?.();
  x?.foo_;
  x?.foo_();
  x?.foo_.bar_;
  x?.foo_.bar_();
  x?.['foo_'].bar_;
  x?.foo_['bar_'];
}`,
      { mangleProps: "_$", compress: false, module: true },
    );
    expect(r.code).toBe(
      "export default function(e){e.foo_;e.foo_?.();e?.foo_;e?.foo_();e?.foo_.bar_;e?.foo_.bar_();e?.[`foo_`].bar_;e?.foo_[`bar_`]}",
    );
    // foo_ and bar_ appear only in dot positions plus reserving quoted positions,
    // so nothing is mangled (the quoted occurrences reserve both names).
    expect(r.mangleCache).toEqual({});
  });

  // Ported from bundler_default_test.go:7070 TestReserveProps
  // esbuild input:
  //   export default { foo_: 0, _bar_: 1 }
  // MangleProps: "_$", ReserveProps: "^_.*_$". `foo_` matches mangle and not
  // reserve -> renamed. `_bar_` matches mangle but ALSO matches reserve -> kept.
  it("TestReserveProps", () => {
    const r = minifySync(
      "entry.js",
      `export default { foo_: 0, _bar_: 1 }`,
      { mangleProps: "_$", reserveProps: "^_.*_$", compress: false, module: true },
    );
    expect(r.code).toBe(`export default{e:0,_bar_:1};`);
    expect(r.mangleCache).toEqual({ foo_: "e" });
  });

  // Ported from bundler_default_test.go:7239 TestManglePropsAvoidCollisions
  // esbuild input:
  //   export default {
  //     foo_: 0, // Must not be named "a"
  //     bar_: 1, // Must not be named "b"
  //     a: 2,
  //     b: 3,
  //     __proto__: {}, // Always avoid mangling this
  //   }
  // Mangled names must not collide with the existing `a`/`b` props, and
  // `__proto__` must never be mangled. oxc's names start at `e`/`t`, so no
  // collision is possible, and `__proto__` is left intact.
  it("TestManglePropsAvoidCollisions", () => {
    const r = minifySync(
      "entry.js",
      `export default {
  foo_: 0,
  bar_: 1,
  a: 2,
  b: 3,
  __proto__: {},
}`,
      { mangleProps: "_$", compress: false, module: true },
    );
    expect(r.code).toBe(`export default{t:0,e:1,a:2,b:3,__proto__:{}};`);
    expect(r.mangleCache).toEqual({ foo_: "t", bar_: "e" });
    // Renamed names never collide with the pre-existing `a`/`b`:
    expect(r.code).toContain("a:2");
    expect(r.code).toContain("b:3");
    expect(r.code).toContain("__proto__:{}");
  });

  // Ported from bundler_default_test.go:7369 TestManglePropsShorthand
  // esbuild input:
  //   export let yyyyy = ({ xxxxx }) => ({ xxxxx })
  // MangleProps: "x" + MinifyIdentifiers. Because the local binding and the
  // property are renamed to the same base54 name, the object/parameter shorthand
  // is preserved: `({ e }) => ({ e })` rather than `({ e: e }) => ({ e: e })`.
  it("TestManglePropsShorthand (shorthand preserved)", () => {
    const r = minifySync(
      "entry.js",
      `export let yyyyy = ({ xxxxx }) => ({ xxxxx })`,
      { mangleProps: "x", compress: false, mangle: true, module: true },
    );
    expect(r.code).toBe(`export let yyyyy=({e})=>({e});`);
    expect(r.mangleCache).toEqual({ xxxxx: "e" });
  });

  // Ported from bundler_default_test.go:7431 TestManglePropsSuperCall
  // esbuild input:
  //   class Foo {}
  //   class Bar extends Foo { constructor() { super(); } }
  // Regression test (esbuild#1976): `constructor` must NEVER be mangled even with
  // MangleProps: regexp(".") (mangle everything), otherwise the method stops
  // being a constructor and `super()` becomes a parse error. oxc correctly leaves
  // `constructor` intact (nothing here is renamed).
  it("TestManglePropsSuperCall (constructor not mangled)", () => {
    const r = minifySync(
      "entry.js",
      `class Foo {}
class Bar extends Foo {
  constructor() {
    super();
  }
}`,
      { mangleProps: ".", compress: false },
    );
    expect(r.code).toBe(`class Foo{}class Bar extends Foo{constructor(){super()}}`);
    expect(r.code).toContain("constructor");
    expect(r.mangleCache).toEqual({});
  });

  // Ported from bundler_default_test.go:7452 TestMangleNoQuotedProps
  // esbuild input (MangleProps: "_", MangleQuoted: false):
  //   x['_doNotMangleThis'];
  //   x?.['_doNotMangleThis'];
  //   x[y ? '_doNotMangleThis' : z];
  //   x?.[y ? '_doNotMangleThis' : z];
  //   x[y ? z : '_doNotMangleThis'];
  //   x?.[y ? z : '_doNotMangleThis'];
  //   ({ '_doNotMangleThis': x });
  //   (class { '_doNotMangleThis' = x });
  //   var { '_doNotMangleThis': x } = y;
  //   '_doNotMangleThis' in x;
  //   (y ? '_doNotMangleThis' : z) in x;
  //   (y ? z : '_doNotMangleThis') in x;
  // All occurrences are QUOTED. With no mangleQuoted (oxc v1's only mode), every
  // `_doNotMangleThis` is preserved. (oxc prints bracket string keys as
  // templates and keeps object/class quoted keys as strings.)
  it("TestMangleNoQuotedProps (quoted never mangled)", () => {
    const r = minifySync(
      "entry.js",
      `x['_doNotMangleThis'];
x?.['_doNotMangleThis'];
x[y ? '_doNotMangleThis' : z];
x?.[y ? '_doNotMangleThis' : z];
x[y ? z : '_doNotMangleThis'];
x?.[y ? z : '_doNotMangleThis'];
({ '_doNotMangleThis': x });
(class { '_doNotMangleThis' = x });
var { '_doNotMangleThis': x } = y;
'_doNotMangleThis' in x;
(y ? '_doNotMangleThis' : z) in x;
(y ? z : '_doNotMangleThis') in x;`,
      { mangleProps: "_", compress: false },
    );
    expect(r.code).toBe(
      "x[`_doNotMangleThis`];x?.[`_doNotMangleThis`];x[y?`_doNotMangleThis`:z];x?.[y?`_doNotMangleThis`:z];x[y?z:`_doNotMangleThis`];x?.[y?z:`_doNotMangleThis`];({\"_doNotMangleThis\":x});(class{\"_doNotMangleThis\"=x});var{\"_doNotMangleThis\":x}=y;`_doNotMangleThis`in x;(y?`_doNotMangleThis`:z)in x;(y?z:`_doNotMangleThis`)in x;",
    );
    expect(r.mangleCache).toEqual({});
    // The property name survives everywhere (never renamed to a base54 name):
    expect(r.code).not.toContain(".e");
  });

  // Ported from bundler_default_test.go:7480 TestMangleNoQuotedPropsMinifySyntax
  // Same input as TestMangleNoQuotedProps but with MinifySyntax (compress: true).
  // `_doNotMangleThis` is still never mangled. compress additionally normalizes
  // some quoted bracket accesses back into dot/identifier form and joins
  // statements, but the property NAME is preserved in every position.
  it("TestMangleNoQuotedPropsMinifySyntax (quoted never mangled, compressed)", () => {
    const r = minifySync(
      "entry.js",
      `x['_doNotMangleThis'];
x?.['_doNotMangleThis'];
x[y ? '_doNotMangleThis' : z];
x?.[y ? '_doNotMangleThis' : z];
x[y ? z : '_doNotMangleThis'];
x?.[y ? z : '_doNotMangleThis'];
({ '_doNotMangleThis': x });
(class { '_doNotMangleThis' = x });
var { '_doNotMangleThis': x } = y;
'_doNotMangleThis' in x;
(y ? '_doNotMangleThis' : z) in x;
(y ? z : '_doNotMangleThis') in x;`,
      { mangleProps: "_", compress: true },
    );
    expect(r.code).toBe(
      "x._doNotMangleThis,x?._doNotMangleThis,x[y?`_doNotMangleThis`:z],x?.[y?`_doNotMangleThis`:z],x[y?z:`_doNotMangleThis`],x?.[y?z:`_doNotMangleThis`];var{_doNotMangleThis:x}=y;`_doNotMangleThis`in x,(y?`_doNotMangleThis`:z)in x,(y?z:`_doNotMangleThis`)in x;",
    );
    expect(r.mangleCache).toEqual({});
    // Never renamed to a base54 name:
    expect(r.code).not.toContain(".e");
    expect(r.code).toContain("_doNotMangleThis");
  });

  // ────────────────────────────────────────────────────────────────────────
  // Behavioral observations (oxc-specific, derived from porting the above)
  // ────────────────────────────────────────────────────────────────────────

  // Documents oxc's conservative handling of a shorthand destructuring
  // ASSIGNMENT target, which is why `bar_` is reserved in TestMangleProps/
  // TestManglePropsMinify above. Compare:
  //   ({ bar_ } = foo)        -> `bar_` reserved program-wide (kept), `baz_` still mangled
  //   ({ bar_: v } = foo)     -> `bar_` mangled normally
  // The output is always CONSISTENT (no half-renamed property), so this is a
  // safe choice, not a bug. esbuild instead un-shorthands and mangles these.
  it("oxc: shorthand assignment target reserves the property (safe, consistent)", () => {
    const shorthand = minifySync("e.js", `({ bar_ } = foo); x.bar_; x.baz_;`, {
      mangleProps: "_$",
      compress: false,
    });
    // bar_ kept everywhere (consistent); baz_ still mangled.
    expect(shorthand.code).toBe("({bar_}=foo);x.bar_;x.e;");
    expect(shorthand.mangleCache).toEqual({ baz_: "e" });

    const explicit = minifySync("e.js", `({ bar_: v } = foo); x.bar_;`, {
      mangleProps: "_$",
      compress: false,
    });
    // With an explicit target key, bar_ is mangled normally.
    expect(explicit.code).toBe("({e:v}=foo);x.e;");
    expect(explicit.mangleCache).toEqual({ bar_: "e" });
  });

  // ────────────────────────────────────────────────────────────────────────
  // Out-of-scope tests (skipped, with reason). Listed so every one of the 23
  // esbuild property-mangling tests is accounted for.
  // ────────────────────────────────────────────────────────────────────────

  // bundler_default_test.go:7044 TestManglePropsLoweredOptionalChain
  // SKIP: identical input to TestManglePropsOptionalChain but with
  // UnsupportedJSFeatures: OptionalChain (syntax lowering). Lowering `?.` is a
  // transformer concern, not minify; oxc-minify keeps `?.` regardless of target.
  // The non-lowered property-mangling essence is already covered by
  // TestManglePropsOptionalChain above.
  it.skip("TestManglePropsLoweredOptionalChain (lowering is a transformer concern)", () => {});

  // Ported from bundler_default_test.go:7090 TestManglePropsImportExport
  // esbuild input (two passthrough files):
  //   /esm.js:  export let foo_ = 123       // NOT a property name -> not mangled
  //             import { bar_ } from 'xyz'  // NOT a property name -> not mangled
  //   /cjs.js:  exports.foo_ = 123          // member access -> mangled
  //             let bar_ = require('xyz').bar_  // `.bar_` member -> mangled; `let bar_` binding NOT
  //
  // The single-file-portable essence (the only part that doesn't need a module
  // graph): import/export SPECIFIER names and VARIABLE bindings are not property
  // keys and must be left alone, while MEMBER accesses are mangled. We split the
  // two files into two oxc-minify runs (ESM via `module: true`, CJS plain).
  //
  // Confirmed oxc behaves correctly: it leaves the `export let foo_` binding and
  // the `import { bar_ }` specifier untouched, and mangles the `o.member_` /
  // `exports.foo_` / `require(...).bar_` member accesses. (oxc-minify always
  // renames local bindings to short names — so the import's LOCAL binding becomes
  // `import { bar_ as e }`; the EXTERNAL specifier name `bar_` is still preserved
  // and never enters the mangle cache. That local rename is identifier
  // minification, not property mangling.)
  it("TestManglePropsImportExport (ESM: export/import specifiers not mangled, member is)", () => {
    const r = minifySync(
      "esm.mjs",
      `export let foo_ = 123
import { bar_ } from 'xyz'
console.log(bar_)
o.member_ = 1`,
      { mangleProps: "_$", compress: false, module: true },
    );
    // `export let foo_` binding kept; import specifier `bar_` kept (only its local
    // binding is shortened by identifier minification); `o.member_` member mangled.
    expect(r.code).toBe(
      `export let foo_=123;import{bar_ as e}from"xyz";console.log(e);o.e=1;`,
    );
    expect(r.code).toContain("export let foo_=123"); // export binding NOT mangled
    expect(r.code).toContain("import{bar_ as"); // import specifier NOT mangled
    // Only the member access is a property; the binding/specifier names are absent.
    expect(r.mangleCache).toEqual({ member_: "e" });
  });

  it("TestManglePropsImportExport (CJS: exports./require() members mangled, binding not)", () => {
    const r = minifySync(
      "cjs.js",
      `exports.foo_ = 123
let baz_ = require('xyz').bar_
console.log(baz_)`,
      { mangleProps: "_$", compress: false },
    );
    // `exports.foo_` and `require(...).bar_` are member accesses -> mangled; the
    // `let baz_` variable binding is not a property -> kept.
    expect(r.code).toBe(
      `exports.t=123;let baz_=require("xyz").e;console.log(baz_);`,
    );
    expect(r.code).toContain("let baz_="); // variable binding NOT mangled
    expect(r.mangleCache).toEqual({ bar_: "e", foo_: "t" });
  });

  // bundler_default_test.go:7117 TestManglePropsImportExportBundled
  // SKIP (by design, not a deferred feature): this is a four-file bundler test
  // (config.ModeBundle) whose whole point is cross-module property-name
  // CONSISTENCY — the same `esm_foo_`/`cjs_foo_` must mangle identically across
  // the files that share them. oxc-minify operates on a single self-contained
  // program and is deliberately NOT a bundler, so there is no module graph to make
  // names consistent across; this is permanently out of v1 scope. The
  // single-file-portable essence (import/export specifiers are bindings, not
  // property keys, and so are not mangled) is already covered by the now-ported
  // TestManglePropsImportExport above. Note esbuild's own test comment documents
  // this code as deliberately "broken" — it exists only to pin the behavior, not
  // to assert a useful result.
  it.skip("TestManglePropsImportExportBundled (bundler cross-module consistency; not a bundler by design)", () => {});

  // bundler_default_test.go:7162 TestManglePropsJSXTransform
  // SKIP: input is JSX and relies on esbuild's JSX transform wiring the factory/
  // fragment to mangled members (Foo.createElement_/Foo.Fragment_). oxc-minify
  // runs post-transform on plain JS and does not perform JSX transform, so the
  // test's premise can't be reproduced here.
  it.skip("TestManglePropsJSXTransform (JSX transform wiring)", () => {});

  // bundler_default_test.go:7194 TestManglePropsJSXPreserve
  // SKIP: input is JSX with JSX.Preserve (emit JSX unchanged). oxc-minify input
  // is plain JS (post-transform); preserving raw JSX is not a minify concern.
  it.skip("TestManglePropsJSXPreserve (JSX preserve)", () => {});

  // bundler_default_test.go:7219 TestManglePropsJSXTransformNamespace
  // SKIP: JSX namespaced elements (`<KEEP:THIS_ />`) — JSX-specific parsing/
  // transform behavior, out of scope for plain-JS minify input.
  it.skip("TestManglePropsJSXTransformNamespace (JSX namespaces)", () => {});

  // bundler_default_test.go:7261 TestManglePropsTypeScriptFeatures
  // SKIP (N/A by design, not a deferred feature): every construct this test
  // exercises — TS parameter properties (`constructor(public MANGLE_FIELD_)`), TS
  // `namespace` exports, and `enum` members — is TypeScript-only SYNTAX. The oxc
  // minifier is JS-only and runs AFTER the TypeScript transform has already
  // erased/lowered these forms, so by minify time none of them exist as TS syntax:
  // parameter properties have become plain constructor assignments + class fields,
  // namespaces have become IIFE-wrapped objects, and enums have become objects.
  // There is therefore nothing TS-specific left for the property mangler to act on
  // here; this is permanently N/A, not something to implement later.
  //
  // Additionally, esbuild itself DELIBERATELY does not mangle TS enum members (see
  // its own comment at /enum-values.ts in this test): the TypeScript compiler
  // emits enum values as quoted strings, which a JS-level property mangler can't
  // pick up, so esbuild keeps enum members consistent by never mangling them and
  // recommends enum inlining instead. So even enum-member mangling is not a goal.
  it.skip("TestManglePropsTypeScriptFeatures (TS-only syntax; erased before JS minify by design)", () => {});

  // bundler_default_test.go:7387 TestManglePropsNoShorthand
  // SKIP: relies on UnsupportedJSFeatures: ObjectExtensions to force
  // un-shorthanding (`{ y }` -> `{ y: y }`). That lowering is a transformer
  // concern; oxc-minify doesn't expose an es5-style object-shorthand lowering
  // toggle. The shorthand-PRESERVING essence is covered by
  // TestManglePropsShorthand above.
  it.skip("TestManglePropsNoShorthand (object-extensions lowering)", () => {});

  // bundler_default_test.go:7406 TestManglePropsLoweredClassFields
  // SKIP: relies on UnsupportedJSFeatures: ClassField|ClassStaticField to lower
  // class fields to assignments — a transformer concern, not minify. The
  // non-lowered property-mangling essence (class field `foo_`/`bar_` renamed
  // consistently with `Foo.bar_`/`new Foo().foo_`) is exercised by the entry1
  // class-field positions in TestMangleProps above.
  it.skip("TestManglePropsLoweredClassFields (class-field lowering)", () => {});

  // Ported from bundler_default_test.go:7509 TestMangleQuotedProps
  // esbuild input — two passthrough parts (`/keep.js` and `/mangle.js`) built
  // with MangleProps: "_", MangleQuoted: true, MinifySyntax: false.
  //
  //   /keep.js (the strings here are CALL ARGUMENTS / non-literal computed
  //   indices, i.e. NOT statically-known keys, so they must NOT be mangled):
  //     foo("_keepThisProperty");
  //     foo((x, "_keepThisProperty"));
  //     foo(x ? "_keepThisProperty" : "_keepThisPropertyToo");
  //     x[foo("_keepThisProperty")];
  //     x?.[foo("_keepThisProperty")];
  //     ({ [foo("_keepThisProperty")]: x });
  //     (class { [foo("_keepThisProperty")] = x });
  //     var { [foo("_keepThisProperty")]: x } = y;
  //     foo("_keepThisProperty") in x;
  //
  //   /mangle.js (every `_mangleThis` is in a KEY/INDEX position — direct or
  //   wrapped in a conditional / comma — so with mangleQuoted it is renamed):
  //     x['_mangleThis']; x?.['_mangleThis'];
  //     x[y ? '_mangleThis' : z]; x?.[y ? '_mangleThis' : z];
  //     x[y ? z : '_mangleThis']; x?.[y ? z : '_mangleThis'];
  //     x[y, '_mangleThis']; x?.[y, '_mangleThis'];
  //     ({ '_mangleThis': x }); ({ ['_mangleThis']: x }); ({ [(y, '_mangleThis')]: x });
  //     (class { '_mangleThis' = x }); (class { ['_mangleThis'] = x }); (class { [(y, '_mangleThis')] = x });
  //     var { '_mangleThis': x } = y; var { ['_mangleThis']: x } = y; var { [(z, '_mangleThis')]: x } = y;
  //     '_mangleThis' in x; (y ? '_mangleThis' : z) in x; (y ? z : '_mangleThis') in x; (y, '_mangleThis') in x;
  //
  // oxc supports `mangleQuoted: true` and handles ALL of esbuild's cases here,
  // including the wrapped conditional/comma forms in member-index, object/class
  // computed-key, destructuring computed-key, and `in`-LHS positions. A direct
  // string index/key is un-quoted (`x['_mangleThis']` -> `x.e`, `{'_mangleThis':x}`
  // -> `{e:x}`); a wrapped form keeps its structure and is renamed in place
  // (printed as a template literal, oxc's codegen choice for string keys in
  // computed/expression positions — same as the existing TestMangleNoQuotedProps).
  // `_keepThisProperty` is never a statically-known key, so it stays untouched.
  it("TestMangleQuotedProps", () => {
    // keep.js: nothing is a statically-known key -> no mangling.
    const keep = minifySync(
      "keep.js",
      `foo("_keepThisProperty");
foo((x, "_keepThisProperty"));
foo(x ? "_keepThisProperty" : "_keepThisPropertyToo");
x[foo("_keepThisProperty")];
x?.[foo("_keepThisProperty")];
({ [foo("_keepThisProperty")]: x });
(class { [foo("_keepThisProperty")] = x });
var { [foo("_keepThisProperty")]: x } = y;
foo("_keepThisProperty") in x;`,
      { mangleProps: "_", mangleQuoted: true, compress: false },
    );
    expect(keep.code).toBe(
      "foo(`_keepThisProperty`);foo((x,`_keepThisProperty`));foo(x?`_keepThisProperty`:`_keepThisPropertyToo`);x[foo(`_keepThisProperty`)];x?.[foo(`_keepThisProperty`)];({[foo(`_keepThisProperty`)]:x});(class{[foo(`_keepThisProperty`)]=x});var{[foo(`_keepThisProperty`)]:x}=y;foo(`_keepThisProperty`)in x;",
    );
    expect(keep.mangleCache).toEqual({});

    // mangle.js: every `_mangleThis` key/index occurrence is renamed to `e`.
    const mangle = minifySync(
      "mangle.js",
      `x['_mangleThis'];
x?.['_mangleThis'];
x[y ? '_mangleThis' : z];
x?.[y ? '_mangleThis' : z];
x[y ? z : '_mangleThis'];
x?.[y ? z : '_mangleThis'];
x[y, '_mangleThis'];
x?.[y, '_mangleThis'];
({ '_mangleThis': x });
({ ['_mangleThis']: x });
({ [(y, '_mangleThis')]: x });
(class { '_mangleThis' = x });
(class { ['_mangleThis'] = x });
(class { [(y, '_mangleThis')] = x });
var { '_mangleThis': x } = y;
var { ['_mangleThis']: x } = y;
var { [(z, '_mangleThis')]: x } = y;
'_mangleThis' in x;
(y ? '_mangleThis' : z) in x;
(y ? z : '_mangleThis') in x;
(y, '_mangleThis') in x;`,
      { mangleProps: "_", mangleQuoted: true, compress: false },
    );
    expect(mangle.code).toBe(
      "x.e;x?.[`e`];x[y?`e`:z];x?.[y?`e`:z];x[y?z:`e`];x?.[y?z:`e`];x[y,`e`];x?.[y,`e`];({e:x});({e:x});({[(y,`e`)]:x});(class{e=x});(class{e=x});(class{[(y,`e`)]=x});var{e:x}=y;var{e:x}=y;var{[(z,`e`)]:x}=y;`e`in x;(y?`e`:z)in x;(y?z:`e`)in x;(y,`e`)in x;",
    );
    // Every occurrence renamed consistently to one name; nothing kept as `_mangleThis`.
    expect(mangle.mangleCache).toEqual({ _mangleThis: "e" });
    expect(mangle.code).not.toContain("_mangleThis");
  });

  // Ported from bundler_default_test.go:7557 TestMangleQuotedPropsMinifySyntax
  // Same two parts as TestMangleQuotedProps, with MinifySyntax (compress: true).
  // `_keepThisProperty` is still never mangled; every `_mangleThis` key/index is
  // still renamed to `e`. compress additionally un-quotes some optional-chain
  // bracket accesses to dot form (`x?.['_mangleThis']` -> `x?.e`), folds the
  // pure object/class expression statements, and joins statements with commas —
  // but the property NAME decision is identical to the uncompressed case.
  it("TestMangleQuotedPropsMinifySyntax", () => {
    const keep = minifySync(
      "keep.js",
      `foo("_keepThisProperty");
foo((x, "_keepThisProperty"));
foo(x ? "_keepThisProperty" : "_keepThisPropertyToo");
x[foo("_keepThisProperty")];
x?.[foo("_keepThisProperty")];
({ [foo("_keepThisProperty")]: x });
(class { [foo("_keepThisProperty")] = x });
var { [foo("_keepThisProperty")]: x } = y;
foo("_keepThisProperty") in x;`,
      { mangleProps: "_", mangleQuoted: true, compress: true },
    );
    expect(keep.code).toBe(
      "foo(`_keepThisProperty`),foo(`_keepThisProperty`),foo(x?`_keepThisProperty`:`_keepThisPropertyToo`),x[foo(`_keepThisProperty`)],x?.[foo(`_keepThisProperty`)],foo(`_keepThisProperty`),foo(`_keepThisProperty`);var{[foo(`_keepThisProperty`)]:x}=y;foo(`_keepThisProperty`)in x;",
    );
    expect(keep.mangleCache).toEqual({});

    const mangle = minifySync(
      "mangle.js",
      `x['_mangleThis'];
x?.['_mangleThis'];
x[y ? '_mangleThis' : z];
x?.[y ? '_mangleThis' : z];
x[y ? z : '_mangleThis'];
x?.[y ? z : '_mangleThis'];
x[y, '_mangleThis'];
x?.[y, '_mangleThis'];
({ '_mangleThis': x });
({ ['_mangleThis']: x });
({ [(y, '_mangleThis')]: x });
(class { '_mangleThis' = x });
(class { ['_mangleThis'] = x });
(class { [(y, '_mangleThis')] = x });
var { '_mangleThis': x } = y;
var { ['_mangleThis']: x } = y;
var { [(z, '_mangleThis')]: x } = y;
'_mangleThis' in x;
(y ? '_mangleThis' : z) in x;
(y ? z : '_mangleThis') in x;
(y, '_mangleThis') in x;`,
      { mangleProps: "_", mangleQuoted: true, compress: true },
    );
    expect(mangle.code).toBe(
      "x.e,x?.e,x[y?`e`:z],x?.[y?`e`:z],x[y?z:`e`],x?.[y?z:`e`],x[y,`e`],x?.[y,`e`],y,y;var{e:x}=y,{e:x}=y,{[(z,`e`)]:x}=y;`e`in x,(y?`e`:z)in x,(y?z:`e`)in x,(y,`e`)in x;",
    );
    expect(mangle.mangleCache).toEqual({ _mangleThis: "e" });
    expect(mangle.code).not.toContain("_mangleThis");
  });

  // Ported from bundler_default_test.go:7623 TestManglePropsKeyComment
  // esbuild input (MangleProps: regexp("_"), MinifySyntax unset -> compress:false):
  //   x(/* __KEY__ */ '_doNotMangleThis', /* __KEY__ */ `_doNotMangleThis`)
  //   x._mangleThis(/* @__KEY__ */ '_mangleThis', /* @__KEY__ */ `_mangleThis`)
  //   x._mangleThisToo(/* #__KEY__ */ '_mangleThisToo', /* #__KEY__ */ `_mangleThisToo`)
  //   x._someKey = /* #__KEY__ */ '_someKey' in y
  //   x([
  //     `foo.${/* @__KEY__ */ '_mangleThis'} = bar.${/* @__KEY__ */ '_mangleThisToo'}`,
  //     `foo.${/* @__KEY__ */ 'notMangled'} = bar.${/* @__KEY__ */ 'notMangledEither'}`,
  //   ])
  //
  // A string/no-substitution-template directly preceded by `/* @__KEY__ */` or
  // `/* #__KEY__ */` is treated as a property name even outside a key position,
  // so it is mangled (the regex still gates it). A bare `/* __KEY__ */` (no `@`/`#`)
  // is NOT an annotation, so `_doNotMangleThis` is left untouched. `_mangleThis` /
  // `_mangleThisToo` / `_someKey` are renamed consistently (the member `.foo`
  // positions AND the annotated string args share one global map). `notMangled` /
  // `notMangledEither` are annotated but do not match the regex `_`, so they stay.
  //
  // oxc's base54 names start at `e`/`t`/`n` (vs esbuild's `a`/`b`/`c`), and oxc's
  // codegen prints string call-arguments as template literals — so the
  // unannotated `'_doNotMangleThis'` arg is printed as `` `_doNotMangleThis` ``
  // purely by codegen, NOT by any rename (confirmed: this happens with no
  // mangleProps at all).
  it("TestManglePropsKeyComment", () => {
    const r = minifySync(
      "entry.js",
      `x(/* __KEY__ */ '_doNotMangleThis', /* __KEY__ */ \`_doNotMangleThis\`)
x._mangleThis(/* @__KEY__ */ '_mangleThis', /* @__KEY__ */ \`_mangleThis\`)
x._mangleThisToo(/* #__KEY__ */ '_mangleThisToo', /* #__KEY__ */ \`_mangleThisToo\`)
x._someKey = /* #__KEY__ */ '_someKey' in y
x([
  \`foo.\${/* @__KEY__ */ '_mangleThis'} = bar.\${/* @__KEY__ */ '_mangleThisToo'}\`,
  \`foo.\${/* @__KEY__ */ 'notMangled'} = bar.\${/* @__KEY__ */ 'notMangledEither'}\`,
])`,
      { mangleProps: "_", compress: false },
    );
    expect(r.code).toBe(
      "x(`_doNotMangleThis`,`_doNotMangleThis`);x.e(`e`,`e`);x.t(`t`,`t`);x.n=`n`in y;x([`foo.${`e`} = bar.${`t`}`,`foo.${`notMangled`} = bar.${`notMangledEither`}`]);",
    );
    // _mangleThis -> e, _mangleThisToo -> t, _someKey -> n (consistent member + annotation).
    expect(r.mangleCache).toEqual({ _mangleThis: "e", _mangleThisToo: "t", _someKey: "n" });
    // The non-annotated `_doNotMangleThis` and the regex-non-matching annotated
    // strings survive verbatim.
    expect(r.code).toContain("_doNotMangleThis");
    expect(r.code).toContain("notMangled");
    expect(r.code).toContain("notMangledEither");
  });

  // Ported from bundler_default_test.go:7646 TestManglePropsKeyCommentMinify
  // esbuild input (MangleProps: regexp("_"), MinifySyntax: true -> compress:true):
  //   x = class {
  //     _mangleThis = 1;
  //     [/* @__KEY__ */ '_mangleThisToo'] = 2;
  //     '_doNotMangleThis' = 3;
  //   }
  //   x = {
  //     _mangleThis: 1,
  //     [/* @__KEY__ */ '_mangleThisToo']: 2,
  //     '_doNotMangleThis': 3,
  //   }
  //   x._mangleThis = 1
  //   x[/* @__KEY__ */ '_mangleThisToo'] = 2
  //   x['_doNotMangleThis'] = 3
  //   x([
  //     `${foo}.${/* @__KEY__ */ '_mangleThis'} = bar.${/* @__KEY__ */ '_mangleThisToo'}`,
  //     `${foo}.${/* @__KEY__ */ 'notMangled'} = bar.${/* @__KEY__ */ 'notMangledEither'}`,
  //   ])
  //
  // `_mangleThis` is a real (unquoted) class-field/object/member key -> mangled to
  // `e`. `_mangleThisToo` appears only as an annotated computed key/index
  // (`[/* @__KEY__ */ '_mangleThisToo']`) -> mangled to `t` (compress un-quotes the
  // resulting static key). `'_doNotMangleThis'` is a quoted key with no annotation
  // and mangleQuoted off -> reserved program-wide (kept). The annotated strings
  // inside the template interpolations are renamed BEFORE compress folds them into
  // the quasi, so `${...}` interpolations become `.e`/`.t`; `notMangled` /
  // `notMangledEither` are annotated but do not match `_`, so they stay.
  it("TestManglePropsKeyCommentMinify", () => {
    const r = minifySync(
      "entry.js",
      `x = class {
  _mangleThis = 1;
  [/* @__KEY__ */ '_mangleThisToo'] = 2;
  '_doNotMangleThis' = 3;
}
x = {
  _mangleThis: 1,
  [/* @__KEY__ */ '_mangleThisToo']: 2,
  '_doNotMangleThis': 3,
}
x._mangleThis = 1
x[/* @__KEY__ */ '_mangleThisToo'] = 2
x['_doNotMangleThis'] = 3
x([
  \`\${foo}.\${/* @__KEY__ */ '_mangleThis'} = bar.\${/* @__KEY__ */ '_mangleThisToo'}\`,
  \`\${foo}.\${/* @__KEY__ */ 'notMangled'} = bar.\${/* @__KEY__ */ 'notMangledEither'}\`,
])`,
      { mangleProps: "_", compress: true },
    );
    expect(r.code).toBe(
      "x=class{e=1;t=2;_doNotMangleThis=3},x={e:1,t:2,_doNotMangleThis:3},x.e=1,x.t=2,x._doNotMangleThis=3,x([`${foo}.e = bar.t`,`${foo}.notMangled = bar.notMangledEither`]);",
    );
    // _mangleThis -> e (real key), _mangleThisToo -> t (annotated computed key).
    expect(r.mangleCache).toEqual({ _mangleThis: "e", _mangleThisToo: "t" });
    // Quoted-but-unannotated `_doNotMangleThis` reserved; regex-non-matching
    // annotated strings survive.
    expect(r.code).toContain("_doNotMangleThis");
    expect(r.code).toContain("notMangled");
    expect(r.code).toContain("notMangledEither");
  });
});
