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
//   * No `/* @__KEY__ */` annotation support -> it.skip.
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

  // bundler_default_test.go:7090 TestManglePropsImportExport
  // SKIP: multi-file (/esm.js + /cjs.js) and depends on ESM-vs-CJS module-record
  // semantics (export/import names don't count as properties; CJS `exports.foo_`
  // does). oxc-minify v1 is a single self-contained JS program with no module
  // graph, so this cannot be reproduced.
  it.skip("TestManglePropsImportExport (multi-file / module-record semantics)", () => {});

  // bundler_default_test.go:7117 TestManglePropsImportExportBundled
  // SKIP: bundler test (config.ModeBundle) across four files. Out of scope for
  // single-file oxc-minify.
  it.skip("TestManglePropsImportExportBundled (bundler/multi-file)", () => {});

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
  // SKIP: TypeScript-only — parameter properties, namespace exports, and enum
  // members. oxc-minify v1 is JS-only and does not run the TS transform.
  it.skip("TestManglePropsTypeScriptFeatures (TypeScript-only)", () => {});

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

  // bundler_default_test.go:7623 TestManglePropsKeyComment
  // SKIP: depends on `/* @__KEY__ */` annotation comments to opt string literals
  // into property mangling. oxc-minify v1 does not support `@__KEY__`
  // annotations.
  it.skip("TestManglePropsKeyComment (@__KEY__ annotations not supported)", () => {});

  // bundler_default_test.go:7646 TestManglePropsKeyCommentMinify
  // SKIP: same `/* @__KEY__ */` annotation dependency, with MinifySyntax. Not
  // supported in oxc-minify v1.
  it.skip("TestManglePropsKeyCommentMinify (@__KEY__ annotations not supported)", () => {});
});
