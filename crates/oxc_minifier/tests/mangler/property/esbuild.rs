//! Conformance tests ported from esbuild's property-mangling bundler tests:
//!   esbuild/internal/bundler_tests/bundler_default_test.go
//!
//! These do NOT assert esbuild's literal output. esbuild's mangled names start at
//! `a`/`b`/... and its codegen formatting differs from oxc's (oxc's base54 names
//! start at `e`/`t`/`n`/...). Instead, each ported test feeds esbuild's INPUT to
//! the oxc property mangler / minifier with the equivalent options and asserts
//! oxc's ACTUAL output, which verifies that the CORRECT properties were renamed
//! consistently and the right ones were reserved.
//!
//! Option mapping:
//!   esbuild MangleProps: regexp("re")   -> mangle regex "re"
//!   esbuild ReserveProps: regexp("re")  -> `opts.reserve`
//!   esbuild MinifySyntax: true          -> full `Minifier` with compress (the `min_*` helpers)
//!   esbuild MinifySyntax: false/unset   -> `PropertyMangler` directly (the `mangle_*` helpers)
//!   esbuild MinifyIdentifiers: true     -> identifier mangle on (`min_*_mangle`)
//!   esbuild MangleQuoted: true          -> `opts.mangle_quoted = true`
//!   `/* @__KEY__ */` annotations        -> parsed from `program.comments` (handled by `collect`)
//!   import/export ESM case              -> parsed with `SourceType::mjs()`
//!   CJS `exports.`/`require()` case     -> parsed with a normal source type
//!
//! Scope notes (port of the vitest comments):
//!   * Single self-contained JS program. Multi-file/bundler & cross-module
//!     consistency tests are out of scope (see the `#[ignore]` block at the end).
//!   * JS-only. TypeScript-feature tests are out of scope.
//!   * `mangleQuoted` is supported. By DEFAULT (the flag off), a quoted property
//!     access (`o['_x']`) RESERVES the unquoted name `_x` program-wide rather than
//!     mangling it (the safe terser-"strict" behavior). With `mangle_quoted = true`,
//!     quoted keys in key/index positions (including the wrapped conditional /
//!     comma forms esbuild handles) become candidates and are renamed consistently
//!     with their unquoted siblings.
//!   * `/* @__KEY__ */` / `/* #__KEY__ */` annotation comments ARE supported: a
//!     string / no-substitution-template directly preceded by one is treated as a
//!     property NAME to mangle even outside a key position (the regex still gates
//!     it). A bare `/* __KEY__ */` (no `@`/`#`) is NOT an annotation.
//!   * Syntax lowering (optional-chain / class-field lowering) is a transformer
//!     concern, not minify; the non-lowered essence is covered.

use std::collections::BTreeMap;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_minifier::{
    CacheValue, CompressOptions, MangleOptions, ManglePropertiesOptions, Minifier, MinifierOptions,
    PropertyMangleCache, PropertyMangler,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use rustc_hash::FxHashSet;

/// Build `ManglePropertiesOptions` from a mangle regex.
fn opts(regex: &str) -> ManglePropertiesOptions {
    ManglePropertiesOptions {
        mangle: Some(lazy_regex::Regex::new(regex).unwrap()),
        reserve: None,
        reserved: FxHashSet::default(),
        mangle_quoted: false,
        debug: false,
        cache: PropertyMangleCache::default(),
    }
}

/// Flatten a `PropertyMangleCache` into a sorted `old -> new` map (reserved names are dropped,
/// matching esbuild's `mangleCache` which only records the mangled names — reserved entries
/// are stored as `false` and are not asserted here).
fn cache_map(cache: &PropertyMangleCache) -> BTreeMap<String, String> {
    cache
        .map
        .iter()
        .filter_map(|(old, value)| match value {
            CacheValue::Name(new) => Some((old.to_string(), new.to_string())),
            CacheValue::Reserved => None,
        })
        .collect()
}

/// Run the `PropertyMangler` directly (no compress), returning `(code, old->new map)`.
fn mangle(
    src: &str,
    source_type: SourceType,
    mut options: ManglePropertiesOptions,
) -> (String, BTreeMap<String, String>) {
    let alloc = Allocator::default();
    let mut program = Parser::new(&alloc, src, source_type).parse().program;
    let mut m = PropertyMangler::new(std::mem::take(&mut options));
    m.collect(&program);
    let cache = m.rewrite(&mut program, &alloc);
    let code = Codegen::new().build(&program).code;
    (code, cache_map(&cache))
}

/// Run the full `Minifier` (compress on), optionally with identifier mangle, returning
/// `(code, old->new map)`.
fn minify(
    src: &str,
    source_type: SourceType,
    prop_options: ManglePropertiesOptions,
    identifier_mangle: bool,
) -> (String, BTreeMap<String, String>) {
    let alloc = Allocator::default();
    let mut program = Parser::new(&alloc, src, source_type).parse().program;
    let options = MinifierOptions {
        mangle: identifier_mangle.then(MangleOptions::default),
        compress: Some(CompressOptions::default()),
        mangle_properties: Some(prop_options),
    };
    let ret = Minifier::new(options).minify(&alloc, &mut program);
    let code = Codegen::new().with_scoping(ret.scoping).build(&program).code;
    let cache = ret.property_mappings.unwrap();
    (code, cache_map(&cache))
}

/// Build an `old -> new` expectation map from string pairs.
fn map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs.iter().map(|(k, v)| ((*k).to_string(), (*v).to_string())).collect()
}

// ════════════════════════════════════════════════════════════════════════════
// compress:false ports (PropertyMangler-direct via `mangle`)
// ════════════════════════════════════════════════════════════════════════════

// Ported from bundler_default_test.go:6882 TestMangleProps
// esbuild input (/entry2.js): `export default { bar_: 0, 'baz_': 1 }`.
// `bar_` matches `_$` -> renamed; the quoted `'baz_'` is reserved (no mangleQuoted), kept.
#[test]
fn mangle_props_entry2_default_object() {
    let (code, cache) =
        mangle("export default { bar_: 0, 'baz_': 1 }", SourceType::mjs(), opts("_$"));
    assert_eq!(code, "export default {\n\te: 0,\n\t\"baz_\": 1\n};\n");
    assert_eq!(cache, map(&[("bar_", "e")]));
}

// Ported from bundler_default_test.go:6882 TestMangleProps (entry1.js, `shouldMangle` half).
// `baz_`->`e` and `foo_`->`t` rename consistently (member, object-literal, class-field and
// class-method positions). `bar_` is NOT mangled: it appears as a shorthand destructuring
// ASSIGNMENT target `({ bar_ } = foo)`, which oxc conservatively reserves program-wide. The
// result is still consistent (every `bar_` is preserved). Intentional, safe difference from
// esbuild, which tracks the bound local and mangles these.
#[test]
fn mangle_props_entry1_should_mangle_body() {
    let (code, cache) = mangle(
        "export function shouldMangle() {
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
}",
        SourceType::mjs(),
        opts("_$"),
    );
    assert_eq!(
        code,
        "export function shouldMangle() {\n\tlet foo = {\n\t\tbar_: 0,\n\t\te() {}\n\t};\n\
         \tlet { bar_ } = foo;\n\t({bar_} = foo);\n\tclass foo_ {\n\t\tbar_ = 0;\n\t\te() {}\n\
         \t\tstatic bar_ = 0;\n\t\tstatic e() {}\n\t}\n\treturn {\n\t\tbar_,\n\t\tt: foo_\n\t};\n}\n"
    );
    assert_eq!(cache, map(&[("baz_", "e"), ("foo_", "t")]));
}

// Ported from bundler_default_test.go:6998 TestManglePropsKeywordPropertyMinify
// esbuild input: `class Foo { static bar = { get baz() { return 123 } } }`.
// esbuild uses MangleProps: regexp(".") (mangle EVERYTHING) to verify mangled names never
// collide with reserved keywords. oxc's base54 names start at `e` and skip keywords by
// construction. We assert the substantive behavior: every property (`bar`, and the GETTER
// `baz`) is renamed. compress:false so the unreferenced class is not tree-shaken.
#[test]
fn mangle_props_keyword_property_getter_mangled() {
    let (code, cache) = mangle(
        "class Foo {
  static bar = { get baz() { return 123 } }
}",
        SourceType::mjs(),
        opts("."),
    );
    assert_eq!(code, "class Foo {\n\tstatic e = { get t() {\n\t\treturn 123;\n\t} };\n}\n");
    assert_eq!(cache, map(&[("bar", "e"), ("baz", "t")]));
}

// Ported from bundler_default_test.go:7019 TestManglePropsOptionalChain
// `foo_`/`bar_` rename consistently across dot, optional-dot, and call positions. The
// bracketed `['foo_']`/`['bar_']` are quoted accesses -> oxc reserves them (kept as computed
// string keys, here printed as templates), so those two sites are NOT renamed. The quoted
// occurrences reserve both names program-wide, so nothing is mangled.
#[test]
fn mangle_props_optional_chain() {
    let (code, cache) = mangle(
        "export default function(x) {
  x.foo_;
  x.foo_?.();
  x?.foo_;
  x?.foo_();
  x?.foo_.bar_;
  x?.foo_.bar_();
  x?.['foo_'].bar_;
  x?.foo_['bar_'];
}",
        SourceType::mjs(),
        opts("_$"),
    );
    assert_eq!(
        code,
        "export default function(x) {\n\tx.foo_;\n\tx.foo_?.();\n\tx?.foo_;\n\tx?.foo_();\n\
         \tx?.foo_.bar_;\n\tx?.foo_.bar_();\n\tx?.[\"foo_\"].bar_;\n\tx?.foo_[\"bar_\"];\n}\n"
    );
    assert_eq!(cache, map(&[]));
}

// Ported from bundler_default_test.go:7070 TestReserveProps
// esbuild input: `export default { foo_: 0, _bar_: 1 }`. MangleProps: "_$", ReserveProps:
// "^_.*_$". `foo_` matches mangle and not reserve -> renamed. `_bar_` matches mangle but
// ALSO matches reserve -> kept.
#[test]
fn reserve_props() {
    let mut o = opts("_$");
    o.reserve = Some(lazy_regex::Regex::new("^_.*_$").unwrap());
    let (code, cache) = mangle("export default { foo_: 0, _bar_: 1 }", SourceType::mjs(), o);
    assert_eq!(code, "export default {\n\te: 0,\n\t_bar_: 1\n};\n");
    assert_eq!(cache, map(&[("foo_", "e")]));
}

// Ported from bundler_default_test.go:7239 TestManglePropsAvoidCollisions
// esbuild input has `foo_`, `bar_` (must not collide with the existing `a`/`b`), plus `a`,
// `b`, and `__proto__` (always avoided). oxc's names start at `e`/`t`, so no collision is
// possible, and `__proto__` is left intact.
#[test]
fn mangle_props_avoid_collisions() {
    let (code, cache) = mangle(
        "export default {
  foo_: 0,
  bar_: 1,
  a: 2,
  b: 3,
  __proto__: {},
}",
        SourceType::mjs(),
        opts("_$"),
    );
    assert_eq!(code, "export default {\n\tt: 0,\n\te: 1,\n\ta: 2,\n\tb: 3,\n\t__proto__: {}\n};\n");
    assert_eq!(cache, map(&[("foo_", "t"), ("bar_", "e")]));
    // Renamed names never collide with the pre-existing `a`/`b`, and `__proto__` is intact.
    assert!(code.contains("a: 2"), "{code}");
    assert!(code.contains("b: 3"), "{code}");
    assert!(code.contains("__proto__: {}"), "{code}");
}

// Ported from bundler_default_test.go:7431 TestManglePropsSuperCall
// Regression test (esbuild#1976): `constructor` must NEVER be mangled even with
// MangleProps: regexp(".") (mangle everything), otherwise the method stops being a
// constructor and `super()` becomes a parse error. oxc correctly leaves `constructor`
// intact (nothing here is renamed).
#[test]
fn mangle_props_super_call_constructor_not_mangled() {
    let (code, cache) = mangle(
        "class Foo {}
class Bar extends Foo {
  constructor() {
    super();
  }
}",
        SourceType::mjs(),
        opts("."),
    );
    assert_eq!(
        code,
        "class Foo {}\nclass Bar extends Foo {\n\tconstructor() {\n\t\tsuper();\n\t}\n}\n"
    );
    assert!(code.contains("constructor"), "{code}");
    assert_eq!(cache, map(&[]));
}

// Ported from bundler_default_test.go:7452 TestMangleNoQuotedProps
// esbuild input (MangleProps: "_", MangleQuoted: false): all occurrences of
// `_doNotMangleThis` are QUOTED. With no mangleQuoted, every one is preserved. (oxc prints
// bracket string keys as templates and keeps object/class quoted keys as strings.)
#[test]
fn mangle_no_quoted_props() {
    let (code, cache) = mangle(
        "x['_doNotMangleThis'];
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
(y ? z : '_doNotMangleThis') in x;",
        SourceType::mjs(),
        opts("_"),
    );
    assert_eq!(
        code,
        "x[\"_doNotMangleThis\"];\nx?.[\"_doNotMangleThis\"];\nx[y ? \"_doNotMangleThis\" : z];\n\
         x?.[y ? \"_doNotMangleThis\" : z];\nx[y ? z : \"_doNotMangleThis\"];\n\
         x?.[y ? z : \"_doNotMangleThis\"];\n({ \"_doNotMangleThis\": x });\n\
         (class {\n\t\"_doNotMangleThis\" = x;\n});\nvar { \"_doNotMangleThis\": x } = y;\n\
         \"_doNotMangleThis\" in x;\n(y ? \"_doNotMangleThis\" : z) in x;\n\
         (y ? z : \"_doNotMangleThis\") in x;\n"
    );
    assert_eq!(cache, map(&[]));
    // The property name survives everywhere (never renamed to a base54 name).
    assert!(!code.contains(".e"), "{code}");
}

// Ported from bundler_default_test.go:7509 TestMangleQuotedProps (mangleQuoted: true).
// keep.js: nothing is a statically-known key -> no mangling. mangle.js: every `_mangleThis`
// key/index occurrence (direct or wrapped in a conditional / comma) is renamed to `e`. A
// direct string index/key is un-quoted; a wrapped form keeps its structure and is renamed in
// place (printed as a template literal).
#[test]
fn mangle_quoted_props_keep() {
    let mut o = opts("_");
    o.mangle_quoted = true;
    let (code, cache) = mangle(
        "foo(\"_keepThisProperty\");
foo((x, \"_keepThisProperty\"));
foo(x ? \"_keepThisProperty\" : \"_keepThisPropertyToo\");
x[foo(\"_keepThisProperty\")];
x?.[foo(\"_keepThisProperty\")];
({ [foo(\"_keepThisProperty\")]: x });
(class { [foo(\"_keepThisProperty\")] = x });
var { [foo(\"_keepThisProperty\")]: x } = y;
foo(\"_keepThisProperty\") in x;",
        SourceType::mjs(),
        o,
    );
    assert_eq!(
        code,
        "foo(\"_keepThisProperty\");\nfoo((x, \"_keepThisProperty\"));\n\
         foo(x ? \"_keepThisProperty\" : \"_keepThisPropertyToo\");\nx[foo(\"_keepThisProperty\")];\n\
         x?.[foo(\"_keepThisProperty\")];\n({ [foo(\"_keepThisProperty\")]: x });\n\
         (class {\n\t[foo(\"_keepThisProperty\")] = x;\n});\n\
         var { [foo(\"_keepThisProperty\")]: x } = y;\nfoo(\"_keepThisProperty\") in x;\n"
    );
    assert_eq!(cache, map(&[]));
}

#[test]
fn mangle_quoted_props_mangle() {
    let mut o = opts("_");
    o.mangle_quoted = true;
    let (code, cache) = mangle(
        "x['_mangleThis'];
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
(y, '_mangleThis') in x;",
        SourceType::mjs(),
        o,
    );
    assert_eq!(
        code,
        "x.e;\nx?.[\"e\"];\nx[y ? \"e\" : z];\nx?.[y ? \"e\" : z];\nx[y ? z : \"e\"];\n\
         x?.[y ? z : \"e\"];\nx[y, \"e\"];\nx?.[y, \"e\"];\n({ e: x });\n({ e: x });\n\
         ({ [(y, \"e\")]: x });\n(class {\n\te = x;\n});\n(class {\n\te = x;\n});\n\
         (class {\n\t[(y, \"e\")] = x;\n});\nvar { e: x } = y;\nvar { e: x } = y;\n\
         var { [(z, \"e\")]: x } = y;\n\"e\" in x;\n(y ? \"e\" : z) in x;\n(y ? z : \"e\") in x;\n\
         (y, \"e\") in x;\n"
    );
    // Every occurrence renamed consistently to one name; nothing kept as `_mangleThis`.
    assert_eq!(cache, map(&[("_mangleThis", "e")]));
    assert!(!code.contains("_mangleThis"), "{code}");
}

// Ported from bundler_default_test.go:7623 TestManglePropsKeyComment
// A string/no-substitution-template directly preceded by `/* @__KEY__ */` or `/* #__KEY__ */`
// is treated as a property name even outside a key position, so it is mangled (the regex still
// gates it). A bare `/* __KEY__ */` (no `@`/`#`) is NOT an annotation. `_mangleThis` /
// `_mangleThisToo` / `_someKey` are renamed consistently (the member positions AND the
// annotated string args share one global map). `notMangled` / `notMangledEither` are
// annotated but do not match the regex `_`, so they stay.
#[test]
fn mangle_props_key_comment() {
    let (code, cache) = mangle(
        "x(/* __KEY__ */ '_doNotMangleThis', /* __KEY__ */ `_doNotMangleThis`)
x._mangleThis(/* @__KEY__ */ '_mangleThis', /* @__KEY__ */ `_mangleThis`)
x._mangleThisToo(/* #__KEY__ */ '_mangleThisToo', /* #__KEY__ */ `_mangleThisToo`)
x._someKey = /* #__KEY__ */ '_someKey' in y
x([
  `foo.${/* @__KEY__ */ '_mangleThis'} = bar.${/* @__KEY__ */ '_mangleThisToo'}`,
  `foo.${/* @__KEY__ */ 'notMangled'} = bar.${/* @__KEY__ */ 'notMangledEither'}`,
])",
        SourceType::mjs(),
        opts("_"),
    );
    assert_eq!(
        code,
        "x(\n\t/* __KEY__ */\n\t\"_doNotMangleThis\",\n\t/* __KEY__ */\n\t`_doNotMangleThis`\n);\n\
         x.e(\n\t/* @__KEY__ */\n\t\"e\",\n\t/* @__KEY__ */\n\t`e`\n);\n\
         x.t(\n\t/* #__KEY__ */\n\t\"t\",\n\t/* #__KEY__ */\n\t`t`\n);\n\
         x.n = \"n\" in y;\n\
         x([`foo.${\"e\"} = bar.${\"t\"}`, `foo.${\"notMangled\"} = bar.${\"notMangledEither\"}`]);\n"
    );
    // _mangleThis -> e, _mangleThisToo -> t, _someKey -> n (consistent member + annotation).
    assert_eq!(cache, map(&[("_mangleThis", "e"), ("_mangleThisToo", "t"), ("_someKey", "n")]));
    // The non-annotated `_doNotMangleThis` and the regex-non-matching annotated strings survive.
    assert!(code.contains("_doNotMangleThis"), "{code}");
    assert!(code.contains("notMangled"), "{code}");
    assert!(code.contains("notMangledEither"), "{code}");
}

// Ported from bundler_default_test.go:7090 TestManglePropsImportExport (CJS half).
// `exports.foo_` and `require(...).bar_` are member accesses -> mangled; the `let baz_`
// variable binding is not a property -> kept. Parsed with a normal (script) source type.
#[test]
fn mangle_props_import_export_cjs() {
    let (code, cache) = mangle(
        "exports.foo_ = 123
let baz_ = require('xyz').bar_
console.log(baz_)",
        SourceType::cjs(),
        opts("_$"),
    );
    assert_eq!(code, "exports.t = 123;\nlet baz_ = require(\"xyz\").e;\nconsole.log(baz_);\n");
    // variable binding `baz_` NOT mangled.
    assert!(code.contains("let baz_"), "{code}");
    assert_eq!(cache, map(&[("bar_", "e"), ("foo_", "t")]));
}

// ════════════════════════════════════════════════════════════════════════════
// compress:true ports (full `Minifier` via `minify`)
// ════════════════════════════════════════════════════════════════════════════

// Ported from bundler_default_test.go:6938 TestManglePropsMinify
// Same input shape as TestMangleProps but with MinifySyntax + MinifyIdentifiers. Under
// compress, `baz_`->`e` and `foo_`->`t` are mangled consistently across object-literal,
// class-field and class-method positions, while `bar_` stays reserved (shorthand assignment
// target). Locals get joined/renamed.
#[test]
fn mangle_props_minify() {
    let (code, cache) = minify(
        "export function shouldMangle_XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX() {
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
}",
        SourceType::mjs(),
        opts("_$"),
        true,
    );
    assert_eq!(
        code,
        "export function shouldMangle_XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX() {\n\tlet e = {\n\
         \t\tbar_: 0,\n\t\te() {}\n\t}, { bar_: t } = e;\n\t({bar_: t} = e);\n\tclass n {\n\
         \t\tbar_ = 0;\n\t\te() {}\n\t\tstatic bar_ = 0;\n\t\tstatic e() {}\n\t}\n\
         \treturn {\n\t\tbar_: t,\n\t\tt: n\n\t};\n}\n"
    );
    // baz_ and foo_ mangled; bar_ reserved (shorthand assignment target).
    assert_eq!(cache, map(&[("baz_", "e"), ("foo_", "t")]));
}

// Ported from bundler_default_test.go:7369 TestManglePropsShorthand
// esbuild input: `export let yyyyy = ({ xxxxx }) => ({ xxxxx })`. MangleProps: "x" +
// MinifyIdentifiers. Because the local binding and the property are renamed to the same
// base54 name, the object/parameter shorthand is preserved.
#[test]
fn mangle_props_shorthand_preserved() {
    let (code, cache) =
        minify("export let yyyyy = ({ xxxxx }) => ({ xxxxx })", SourceType::mjs(), opts("x"), true);
    assert_eq!(code, "export let yyyyy = ({ e }) => ({ e });\n");
    assert_eq!(cache, map(&[("xxxxx", "e")]));
}

// Ported from bundler_default_test.go:7480 TestMangleNoQuotedPropsMinifySyntax
// Same input as TestMangleNoQuotedProps but with MinifySyntax. `_doNotMangleThis` is still
// never mangled. compress additionally normalizes some quoted bracket accesses back into
// dot/identifier form and joins statements, but the property NAME is preserved everywhere.
#[test]
fn mangle_no_quoted_props_minify_syntax() {
    let (code, cache) = minify(
        "x['_doNotMangleThis'];
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
(y ? z : '_doNotMangleThis') in x;",
        SourceType::mjs(),
        opts("_"),
        false,
    );
    assert_eq!(
        code,
        "x._doNotMangleThis, x?._doNotMangleThis, x[y ? \"_doNotMangleThis\" : z], \
         x?.[y ? \"_doNotMangleThis\" : z], x[y ? z : \"_doNotMangleThis\"], \
         x?.[y ? z : \"_doNotMangleThis\"];\nvar { _doNotMangleThis: x } = y;\n\
         \"_doNotMangleThis\" in x, (y ? \"_doNotMangleThis\" : z) in x, \
         (y ? z : \"_doNotMangleThis\") in x;\n"
    );
    assert_eq!(cache, map(&[]));
    // Never renamed to a base54 name.
    assert!(!code.contains(".e"), "{code}");
    assert!(code.contains("_doNotMangleThis"), "{code}");
}

// Ported from bundler_default_test.go:7557 TestMangleQuotedPropsMinifySyntax
// Same two parts as TestMangleQuotedProps, with MinifySyntax. `_keepThisProperty` is still
// never mangled; every `_mangleThis` key/index is still renamed to `e`. compress additionally
// un-quotes some optional-chain bracket accesses to dot form, folds the pure object/class
// expression statements, and joins statements with commas.
#[test]
fn mangle_quoted_props_minify_syntax_keep() {
    let mut o = opts("_");
    o.mangle_quoted = true;
    let (code, cache) = minify(
        "foo(\"_keepThisProperty\");
foo((x, \"_keepThisProperty\"));
foo(x ? \"_keepThisProperty\" : \"_keepThisPropertyToo\");
x[foo(\"_keepThisProperty\")];
x?.[foo(\"_keepThisProperty\")];
({ [foo(\"_keepThisProperty\")]: x });
(class { [foo(\"_keepThisProperty\")] = x });
var { [foo(\"_keepThisProperty\")]: x } = y;
foo(\"_keepThisProperty\") in x;",
        SourceType::mjs(),
        o,
        false,
    );
    assert_eq!(
        code,
        "foo(\"_keepThisProperty\"), foo(\"_keepThisProperty\"), \
         foo(x ? \"_keepThisProperty\" : \"_keepThisPropertyToo\"), x[foo(\"_keepThisProperty\")], \
         x?.[foo(\"_keepThisProperty\")], foo(\"_keepThisProperty\"), foo(\"_keepThisProperty\");\n\
         var { [foo(\"_keepThisProperty\")]: x } = y;\nfoo(\"_keepThisProperty\") in x;\n"
    );
    assert_eq!(cache, map(&[]));
}

#[test]
fn mangle_quoted_props_minify_syntax_mangle() {
    let mut o = opts("_");
    o.mangle_quoted = true;
    let (code, cache) = minify(
        "x['_mangleThis'];
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
(y, '_mangleThis') in x;",
        SourceType::mjs(),
        o,
        false,
    );
    assert_eq!(
        code,
        "x.e, x?.e, x[y ? \"e\" : z], x?.[y ? \"e\" : z], x[y ? z : \"e\"], x?.[y ? z : \"e\"], \
         x[y, \"e\"], x?.[y, \"e\"], y, y;\nvar { e: x } = y, { e: x } = y, { [(z, \"e\")]: x } = y;\n\
         \"e\" in x, (y ? \"e\" : z) in x, (y ? z : \"e\") in x, (y, \"e\") in x;\n"
    );
    assert_eq!(cache, map(&[("_mangleThis", "e")]));
    assert!(!code.contains("_mangleThis"), "{code}");
}

// Ported from bundler_default_test.go:7646 TestManglePropsKeyCommentMinify
// `_mangleThis` is a real (unquoted) class-field/object/member key -> mangled to `e`.
// `_mangleThisToo` appears only as an annotated computed key/index -> mangled to `t` (compress
// un-quotes the resulting static key). `'_doNotMangleThis'` is a quoted key with no annotation
// and mangleQuoted off -> reserved program-wide (kept). The annotated strings inside template
// interpolations are renamed BEFORE compress folds them into the quasi; `notMangled` /
// `notMangledEither` are annotated but do not match `_`, so they stay.
#[test]
fn mangle_props_key_comment_minify() {
    let (code, cache) = minify(
        "x = class {
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
  `${foo}.${/* @__KEY__ */ '_mangleThis'} = bar.${/* @__KEY__ */ '_mangleThisToo'}`,
  `${foo}.${/* @__KEY__ */ 'notMangled'} = bar.${/* @__KEY__ */ 'notMangledEither'}`,
])",
        SourceType::mjs(),
        opts("_"),
        false,
    );
    assert_eq!(
        code,
        "x = class {\n\te = 1;\n\tt = 2;\n\t_doNotMangleThis = 3;\n}, x = {\n\te: 1,\n\tt: 2,\n\
         \t_doNotMangleThis: 3\n}, x.e = 1, x.t = 2, x._doNotMangleThis = 3, \
         x([`${foo}.e = bar.t`, `${foo}.notMangled = bar.notMangledEither`]);\n"
    );
    // _mangleThis -> e (real key), _mangleThisToo -> t (annotated computed key).
    assert_eq!(cache, map(&[("_mangleThis", "e"), ("_mangleThisToo", "t")]));
    // Quoted-but-unannotated `_doNotMangleThis` reserved; regex-non-matching annotated survive.
    assert!(code.contains("_doNotMangleThis"), "{code}");
    assert!(code.contains("notMangled"), "{code}");
    assert!(code.contains("notMangledEither"), "{code}");
}

// Ported from bundler_default_test.go:7090 TestManglePropsImportExport (ESM half).
// `export let foo_` binding kept; the `import { bar_ }` specifier name kept (only its local
// binding is shortened by identifier minification); `o.member_` member access mangled. Parsed
// with `SourceType::mjs()` and identifier mangle on (to exercise the `import { bar_ as e }`
// local rename).
#[test]
fn mangle_props_import_export_esm() {
    let (code, cache) = minify(
        "export let foo_ = 123
import { bar_ } from 'xyz'
console.log(bar_)
o.member_ = 1",
        SourceType::mjs(),
        opts("_$"),
        true,
    );
    assert_eq!(
        code,
        "export let foo_ = 123;\nimport { bar_ as e } from \"xyz\";\nconsole.log(e), o.e = 1;\n"
    );
    // export binding NOT mangled; import specifier NOT mangled.
    assert!(code.contains("export let foo_ = 123"), "{code}");
    assert!(code.contains("import { bar_ as"), "{code}");
    assert_eq!(cache, map(&[("member_", "e")]));
}

// ════════════════════════════════════════════════════════════════════════════
// oxc-specific behavioral observation (derived from porting the above)
// ════════════════════════════════════════════════════════════════════════════

// Documents oxc's conservative handling of a shorthand destructuring ASSIGNMENT target,
// which is why `bar_` is reserved in the `mangle_props_*` ports above. Compare:
//   ({ bar_ } = foo)        -> `bar_` reserved program-wide (kept), `baz_` still mangled
//   ({ bar_: v } = foo)     -> `bar_` mangled normally
// The output is always CONSISTENT (no half-renamed property), so this is a safe choice, not a
// bug. esbuild instead un-shorthands and mangles these.
#[test]
fn shorthand_assignment_target_reserves() {
    let (shorthand_code, shorthand_cache) =
        mangle("({ bar_ } = foo); x.bar_; x.baz_;", SourceType::cjs(), opts("_$"));
    assert_eq!(shorthand_code, "({bar_} = foo);\nx.bar_;\nx.e;\n");
    assert_eq!(shorthand_cache, map(&[("baz_", "e")]));

    let (explicit_code, explicit_cache) =
        mangle("({ bar_: v } = foo); x.bar_;", SourceType::cjs(), opts("_$"));
    assert_eq!(explicit_code, "({e: v} = foo);\nx.e;\n");
    assert_eq!(explicit_cache, map(&[("bar_", "e")]));
}

// ════════════════════════════════════════════════════════════════════════════
// Out-of-scope esbuild tests (documented for completeness; not ported).
//
// Every one of esbuild's property-mangling tests is accounted for: the active
// ones above, plus the following that are N/A for a single-file, post-transform,
// JS-only minifier. They are recorded as `#[ignore]`d empty tests so `cargo test`
// lists them with their reason.
// ════════════════════════════════════════════════════════════════════════════

// bundler_default_test.go:7044 TestManglePropsLoweredOptionalChain
#[test]
#[ignore = "syntax lowering (?. lowering) is a transformer concern, not minify; the \
            non-lowered essence is covered by mangle_props_optional_chain"]
fn mangle_props_lowered_optional_chain() {}

// bundler_default_test.go:7117 TestManglePropsImportExportBundled
#[test]
#[ignore = "four-file bundler test (cross-module name consistency); oxc-minify is not a \
            bundler. The single-file essence is covered by mangle_props_import_export_{esm,cjs}"]
fn mangle_props_import_export_bundled() {}

// bundler_default_test.go:7162 TestManglePropsJSXTransform
#[test]
#[ignore = "JSX transform wiring (factory/fragment to mangled members); oxc-minify runs \
            post-transform on plain JS and does not perform JSX transform"]
fn mangle_props_jsx_transform() {}

// bundler_default_test.go:7194 TestManglePropsJSXPreserve
#[test]
#[ignore = "JSX preserve (emit JSX unchanged); oxc-minify input is plain post-transform JS"]
fn mangle_props_jsx_preserve() {}

// bundler_default_test.go:7219 TestManglePropsJSXTransformNamespace
#[test]
#[ignore = "JSX namespaced elements; JSX-specific parsing/transform, out of scope for \
            plain-JS minify input"]
fn mangle_props_jsx_transform_namespace() {}

// bundler_default_test.go:7261 TestManglePropsTypeScriptFeatures
#[test]
#[ignore = "TS-only syntax (parameter properties, namespaces, enums) is erased/lowered by \
            the TS transform before JS minify runs; nothing TS-specific remains to mangle"]
fn mangle_props_typescript_features() {}

// bundler_default_test.go:7387 TestManglePropsNoShorthand
#[test]
#[ignore = "relies on ObjectExtensions lowering to un-shorthand `{ y }` -> `{ y: y }`, a \
            transformer concern; the shorthand-PRESERVING essence is covered by \
            mangle_props_shorthand_preserved"]
fn mangle_props_no_shorthand() {}

// bundler_default_test.go:7406 TestManglePropsLoweredClassFields
#[test]
#[ignore = "class-field lowering is a transformer concern; the non-lowered class-field \
            essence is covered by mangle_props_entry1_should_mangle_body"]
fn mangle_props_lowered_class_fields() {}
