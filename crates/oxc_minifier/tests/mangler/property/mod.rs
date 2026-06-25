mod esbuild;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_minifier::{
    CompressOptions, ManglePropertiesOptions, Minifier, MinifierOptions, PropertyMangleCache,
    PropertyMangler,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use rustc_hash::FxHashSet;

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

fn mangle(src: &str, regex: &str, source_type: SourceType) -> String {
    let alloc = Allocator::default();
    let mut program = Parser::new(&alloc, src, source_type).parse().program;
    let mut m = PropertyMangler::new(opts(regex));
    m.collect(&program);
    m.rewrite(&mut program, &alloc);
    Codegen::new().build(&program).code
}

fn codegen(src: &str, source_type: SourceType) -> String {
    let alloc = Allocator::default();
    let program = Parser::new(&alloc, src, source_type).parse().program;
    Codegen::new().build(&program).code
}

#[track_caller]
fn test(src: &str, expected: &str, regex: &str) {
    let st = SourceType::mjs();
    let got = mangle(src, regex, st);
    let want = codegen(expected, st);
    assert_eq!(got, want, "\nsrc {src}\nexpect {want}\ngot {got}");
}

// `base54` produces names in frequency order, so the first assigned name is `e`, not `a`.

#[test]
fn basic() {
    test("({ _foo: 1 })._foo", "({ e: 1 }).e", "^_");
}

#[test]
fn object_shorthand_expands() {
    // The key is renamed but the value (an `IdentifierReference`) is not, so the shorthand
    // can no longer collapse: codegen prints `{ e: _foo }`.
    test("let _foo; ({ _foo })", "let _foo; ({ e: _foo })", "^_");
}

#[test]
fn binding_shorthand_expands() {
    test("const { _foo } = o;", "const { e: _foo } = o;", "^_");
}

#[test]
fn quoted_member_reserves_unquoted() {
    test("o['_foo']; o._foo;", "o['_foo']; o._foo;", "^_");
}

#[test]
fn getter_and_data_rename_together() {
    test("({ get _foo(){} }); ({ _foo: 1 }); x._foo;", "({ get e(){} }); ({ e: 1 }); x.e;", "^_");
}

#[test]
fn assignment_target_shorthand_reserved() {
    test("({ _foo } = o); o._foo;", "({ _foo } = o); o._foo;", "^_");
}

#[test]
fn assignment_target_property_renamed() {
    test("({ _foo: bar } = o); o._foo;", "({ e: bar } = o); o.e;", "^_");
}

#[test]
fn computed_and_private_untouched() {
    test("o[k]; class C { #_foo = 1 }", "o[k]; class C { #_foo = 1 }", "^_");
}

#[test]
fn protocol_names_survive() {
    // Broad regex matches everything, but `then` is in the protocol denylist so it survives.
    // `o` and `p` are member objects, not properties, so the only candidate is `_foo` -> `e`.
    test("p.then; o._foo;", "p.then; o.e;", ".");
}

#[test]
fn jsx_attr_reserved() {
    // The JSX attribute `_foo` becomes a props key, so it is reserved program-wide.
    // The member `o._foo` must therefore stay `_foo` despite matching `^_`.
    let st = SourceType::jsx();
    let src = "function C({ _foo }) { return _foo; } (<C _foo={1} />); o._foo;";
    let got = mangle(src, "^_", st);
    let want = codegen(src, st);
    assert_eq!(got, want, "\nJSX attribute should reserve `_foo`\nexpect {want}\ngot {got}");
    assert!(got.contains("_foo"), "`_foo` must survive: {got}");
}

#[test]
fn off_when_no_match() {
    test("o.addEventListener();", "o.addEventListener();", "^_");
}

// ---------------------------------------------------------------------------
// Full-minifier integration: exercises the collect (pre-compress) / rewrite
// (post-mangle) wiring inside `Minifier::build`, so the compress pass runs in
// between (and un-quotes string keys, etc.).
// ---------------------------------------------------------------------------

/// Run the full minifier (compress on, variable mangle off) with property mangling enabled.
fn minify_with_props(src: &str, regex: Option<&str>) -> String {
    let alloc = Allocator::default();
    let mut program = Parser::new(&alloc, src, SourceType::mjs()).parse().program;
    let options = MinifierOptions {
        mangle: None,
        compress: Some(CompressOptions::default()),
        mangle_properties: regex.map(opts),
    };
    let ret = Minifier::new(options).minify(&alloc, &mut program);
    Codegen::new().with_scoping(ret.scoping).build(&program).code
}

/// Compare the full minifier output (property mangling on via `regex`) against the
/// compress-only output of `expected` (so codegen formatting matches).
#[track_caller]
fn test_min(src: &str, expected: &str, regex: &str) {
    let got = minify_with_props(src, Some(regex));
    let want = minify_with_props(expected, None);
    assert_eq!(got, want, "\nsrc {src}\nexpect {want}\ngot {got}");
}

#[test]
fn quoted_key_reserved_across_compress() {
    // Compress un-quotes `o['_foo']` -> `o._foo`, but collect ran pre-compress and reserved
    // `_foo` (it was quoted), so it must NOT be renamed even after un-quoting.
    test_min("o['_foo']; o._foo;", "o._foo; o._foo;", "^_");
}

#[test]
fn default_off_leaves_keys() {
    // `mangle_properties: None` => property names are left completely untouched.
    // Use `globalThis` so the member access has an observable side effect and survives DCE.
    let src = "globalThis.addEventListener(); globalThis._foo;";
    let got = minify_with_props(src, None);
    assert!(got.contains("addEventListener"), "method name must survive: {got}");
    assert!(got.contains("_foo"), "property name must survive: {got}");
}

#[test]
fn renames_through_full_minify() {
    // A clear case: `_foo` is an unquoted member/key everywhere, so it mangles to `e`
    // (base54 frequency order) and survives the full compress + rewrite pipeline.
    test_min("({ _foo: 1 })._foo;", "({ e: 1 }).e;", "^_");
}

// ---------------------------------------------------------------------------
// Regression tests: nesting, class members, bail conditions, and reservations.
// Single-candidate cases assert the literal `e` (base54 frequency order); the
// invariant under test is that every occurrence of one source name shares one
// output name while reserved/untouched names are left unchanged.
// ---------------------------------------------------------------------------

/// Like [`test`] but parses as the given source type for both sides (e.g. a script for
/// `with`, which is illegal in module/strict mode).
#[track_caller]
fn test_st(src: &str, expected: &str, regex: &str, st: SourceType) {
    let got = mangle(src, regex, st);
    let want = codegen(expected, st);
    assert_eq!(got, want, "\nsrc {src}\nexpect {want}\ngot {got}");
}

/// PropertyMangler-direct run that also sets a `reserve` regex (carve-out).
fn mangle_with_reserve(src: &str, mangle_re: &str, reserve_re: &str) -> String {
    let alloc = Allocator::default();
    let mut program = Parser::new(&alloc, src, SourceType::mjs()).parse().program;
    let mut o = opts(mangle_re);
    o.reserve = Some(lazy_regex::Regex::new(reserve_re).unwrap());
    let mut m = PropertyMangler::new(o);
    m.collect(&program);
    m.rewrite(&mut program, &alloc);
    Codegen::new().build(&program).code
}

#[test]
fn deep_nested_member_chain() {
    // The same name renames consistently through a deep member chain.
    test("a.b._foo.c._foo; a._foo;", "a.b.e.c.e; a.e;", "^_");
    // A nested object literal: `outer` doesn't match `^_` (untouched), `_foo` -> `e`.
    test("({ outer: { _foo: 1 } })._foo;", "({ outer: { e: 1 } }).e;", "^_");
}

#[test]
fn nested_in_array_and_call_args() {
    // The walk recurses into array elements and call arguments: `_foo` -> `e` everywhere.
    test("[{ _foo: 1 }]; f({ _foo: 2 })._foo;", "[{ e: 1 }]; f({ e: 2 }).e;", "^_");
}

#[test]
fn class_method_field_accessor() {
    // Method declaration + access rename together.
    test("class C { _foo() {} } new C()._foo();", "class C { e() {} } new C().e();", "^_");
    // Field declaration + access.
    test("class C { _foo = 1 } new C()._foo;", "class C { e = 1 } new C().e;", "^_");
    // Getter/setter pair + access.
    test(
        "class C { get _foo(){return 1} set _foo(v){} } o._foo;",
        "class C { get e(){return 1} set e(v){} } o.e;",
        "^_",
    );
    // Auto-accessor field + access.
    test("class C { accessor _foo = 1 } o._foo;", "class C { accessor e = 1 } o.e;", "^_");
}

#[test]
fn static_class_member() {
    test("class C { static _foo() {} } C._foo();", "class C { static e() {} } C.e();", "^_");
}

#[test]
fn this_and_super_member() {
    test("class C { _m() { return this._m } }", "class C { e() { return this.e } }", "^_");
    // A subclass calling `super._m()` renames consistently with the base declaration.
    test(
        "class C extends B { _m() { return super._m() } }",
        "class C extends B { e() { return super.e() } }",
        "^_",
    );
}

#[test]
fn optional_chain_member() {
    test("a?._foo; a._foo;", "a?.e; a.e;", "^_");
}

#[test]
fn delete_member() {
    test("delete o._foo; o._foo;", "delete o.e; o.e;", "^_");
}

#[test]
fn numeric_keys_untouched() {
    // Numeric keys can't be candidates; nothing is renamed.
    test("({ 0: 1 }); o[0];", "({ 0: 1 }); o[0];", "^_");
}

#[test]
fn in_operator_reserves_name() {
    // The `in`-LHS string reserves `_foo`, so the member is NOT renamed.
    test("'_foo' in o; o._foo;", "'_foo' in o; o._foo;", "^_");
}

#[test]
fn quoted_object_key_reserves_member() {
    // A quoted object key reserves `_foo` program-wide, so the unquoted member survives.
    test("({ '_foo': 1 }); o._foo;", "({ '_foo': 1 }); o._foo;", "^_");
}

#[test]
fn with_statement_bails() {
    // `with` makes the whole program bail. `with` is illegal in module/strict mode, so
    // this must parse as a script (mirrors the inline `collect_bails_on_with_and_eval`).
    test_st("with (o) {} o._foo;", "with (o) {} o._foo;", "^_", SourceType::cjs());
}

#[test]
fn eval_bails() {
    // A direct `eval` makes the whole program bail; `o._foo` is left unchanged.
    test("eval('x'); o._foo;", "eval('x'); o._foo;", "^_");
}

#[test]
fn reserve_regex_carves_out() {
    // `mangle: ^_` plus `reserve: _keep$`: `_keep` stays, `_foo` -> `e`.
    let got = mangle_with_reserve("o._keep; o._foo;", "^_", "_keep$");
    let want = codegen("o._keep; o.e;", SourceType::mjs());
    assert_eq!(got, want, "\nexpect {want}\ngot {got}");
}

// ---------------------------------------------------------------------------
// No-substitution template literals (`` `_foo` ``) in key/index positions are
// runtime-equivalent to the quoted string `'_foo'`, so they must be classified
// identically: reserved by default, candidates under `mangle_quoted`.
// ---------------------------------------------------------------------------

/// PropertyMangler-direct run with `mangle_quoted` enabled.
fn mangle_quoted(src: &str, regex: &str) -> String {
    let alloc = Allocator::default();
    let mut program = Parser::new(&alloc, src, SourceType::mjs()).parse().program;
    let mut o = opts(regex);
    o.mangle_quoted = true;
    let mut m = PropertyMangler::new(o);
    m.collect(&program);
    m.rewrite(&mut program, &alloc);
    Codegen::new().build(&program).code
}

#[test]
fn template_member_reserves_unquoted() {
    // A template index `o[`_foo`]` reserves `_foo`, so the unquoted member must NOT be
    // renamed (else `o.e` and `o[`_foo`]` would access different properties).
    test("o[`_foo`]; o._foo;", "o[`_foo`]; o._foo;", "^_");
}

#[test]
fn template_in_operator_reserves_name() {
    test("`_foo` in o; o._foo;", "`_foo` in o; o._foo;", "^_");
}

#[test]
fn template_object_key_reserves_member() {
    test("({ [`_foo`]: 1 }); o._foo;", "({ [`_foo`]: 1 }); o._foo;", "^_");
}

#[test]
fn substitution_template_is_not_a_key() {
    // A template WITH a substitution is a dynamic key, never statically `_foo`, so it does
    // not reserve `_foo`; the unquoted member is still a candidate and renames to `e`.
    test("o[`_foo${x}`]; o._foo;", "o[`_foo${x}`]; o.e;", "^_");
}

#[test]
fn mangle_quoted_renames_template_member() {
    // With `mangle_quoted`, the template index becomes a candidate and is renamed in place
    // (kept as a template), consistently with the unquoted member.
    let got = mangle_quoted("o[`_foo`]; o._foo;", "^_");
    let want = codegen("o[`e`]; o.e;", SourceType::mjs());
    assert_eq!(got, want, "\nexpect {want}\ngot {got}");
}

#[test]
fn mangle_quoted_renames_template_in_operator_and_key() {
    let got = mangle_quoted("`_foo` in o; ({ [`_foo`]: 1 }); o._foo;", "^_");
    let want = codegen("`e` in o; ({ [`e`]: 1 }); o.e;", SourceType::mjs());
    assert_eq!(got, want, "\nexpect {want}\ngot {got}");
}

#[test]
fn template_member_reserved_through_full_minify() {
    // The real bug: through the full pipeline (compress runs between the pre-compress collect
    // and the post-mangle rewrite), the template index reserves `_foo`, so neither occurrence
    // is renamed regardless of how compress reshapes the access. `globalThis` keeps the reads
    // observable so they survive DCE.
    test_min(
        "globalThis.o[`_foo`]; globalThis.o._foo;",
        "globalThis.o[`_foo`]; globalThis.o._foo;",
        "^_",
    );
}
