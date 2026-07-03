mod esbuild;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_minifier::{
    CompressOptions, ManglePropertiesOptions, Minifier, MinifierOptions, PropertyMangleBailKind,
    PropertyMangler,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use rustc_hash::FxHashSet;

fn opts(regex: &str) -> ManglePropertiesOptions {
    ManglePropertiesOptions {
        regex: Some(lazy_regex::Regex::new(regex).unwrap()),
        exclude: None,
        reserved: FxHashSet::default(),
        mangle_quoted: false,
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

/// PropertyMangler-direct run that also sets an `exclude` regex (carve-out).
fn mangle_with_exclude(src: &str, mangle_re: &str, exclude_re: &str) -> String {
    let alloc = Allocator::default();
    let mut program = Parser::new(&alloc, src, SourceType::mjs()).parse().program;
    let mut o = opts(mangle_re);
    o.exclude = Some(lazy_regex::Regex::new(exclude_re).unwrap());
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
fn parenthesized_direct_eval_bails() {
    // Per ECMA-262, `(eval)(x)` is still a DIRECT eval — parentheses preserve the
    // reference — and the parser keeps the `ParenthesizedExpression` node, so the bail
    // must see through it.
    let alloc = Allocator::default();
    let program = Parser::new(&alloc, "(eval)('a._x'); o._foo;", SourceType::mjs()).parse().program;
    let mut m = PropertyMangler::new(opts("^_"));
    m.collect(&program);
    assert_eq!(m.bail().map(|b| b.kind), Some(PropertyMangleBailKind::DirectEval));
}

#[test]
fn parenthesized_function_constructor_bails() {
    // `new (Function)('x')` still reaches the `Function` constructor through the parens.
    let alloc = Allocator::default();
    let program =
        Parser::new(&alloc, "new (Function)('x'); o._foo;", SourceType::mjs()).parse().program;
    let mut m = PropertyMangler::new(opts("^_"));
    m.collect(&program);
    assert_eq!(m.bail().map(|b| b.kind), Some(PropertyMangleBailKind::FunctionConstructor));
}

#[test]
fn bail_is_surfaced_through_minifier_return() {
    // A bailing input (direct `eval`) disables property mangling for the whole file. The bail
    // must be observable on `MinifierReturn` (kind + span) so callers can warn instead of
    // silently shipping unmangled property names in a shared-cache build.
    let alloc = Allocator::default();
    let mut program = Parser::new(&alloc, "eval('x'); o._foo;", SourceType::mjs()).parse().program;
    let options = MinifierOptions {
        mangle: None,
        compress: Some(CompressOptions::default()),
        mangle_properties: Some(opts("^_")),
    };
    let bail = Minifier::new(options)
        .minify(&alloc, &mut program)
        .property_mangle_bail
        .expect("a direct `eval` must surface a bail");
    assert_eq!(bail.kind, PropertyMangleBailKind::DirectEval);
    assert_eq!(bail.span.start, 0, "the bail span points at the `eval(...)` call");

    // A non-bailing input reports `None`.
    let mut clean = Parser::new(&alloc, "o._foo;", SourceType::mjs()).parse().program;
    let options = MinifierOptions {
        mangle: None,
        compress: Some(CompressOptions::default()),
        mangle_properties: Some(opts("^_")),
    };
    assert!(
        Minifier::new(options).minify(&alloc, &mut clean).property_mangle_bail.is_none(),
        "a clean program must not report a bail"
    );
}

#[test]
fn exclude_regex_carves_out() {
    // `regex: ^_` plus `exclude: _keep$`: `_keep` stays, `_foo` -> `e`.
    let got = mangle_with_exclude("o._keep; o._foo;", "^_", "_keep$");
    let want = codegen("o._keep; o.e;", SourceType::mjs());
    assert_eq!(got, want, "\nexpect {want}\ngot {got}");
}

/// PropertyMangler-direct run with explicit `reserved` names.
fn mangle_with_reserved(src: &str, mangle_re: &str, reserved: &[&str]) -> String {
    let alloc = Allocator::default();
    let mut program = Parser::new(&alloc, src, SourceType::mjs()).parse().program;
    let mut o = opts(mangle_re);
    o.reserved = reserved.iter().map(|name| (*name).into()).collect();
    let mut m = PropertyMangler::new(o);
    m.collect(&program);
    m.rewrite(&mut program, &alloc);
    Codegen::new().build(&program).code
}

#[test]
fn reserved_names_not_generated() {
    // `reserved: ['t']` must keep `t` out of the GENERATED names too (terser/esbuild
    // parity), not just reserve source-seen `t`s: `_a` -> `e`, then `_b` skips `t` -> `n`.
    let got = mangle_with_reserved("o._a; o._b;", "^_", &["t"]);
    let want = codegen("o.e; o.n;", SourceType::mjs());
    assert_eq!(got, want, "\nexpect {want}\ngot {got}");
    assert!(!got.contains(".t"), "the user-reserved `t` must never be handed out: {got}");
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

// ---------------------------------------------------------------------------
// Apply-once semantics: the rename map must be applied at most ONCE per position.
// `assign` keeps generated names disjoint from EVERY property name seen in the
// program, so a freshly-written new name (an un-quoted key, a renamed annotated
// literal, or a key compress un-quotes later) is never itself a key in the map
// and can never be re-matched by name. These tests pin that: a source property
// named `e` (base54's first output) forces the generator to skip `e`.
// ---------------------------------------------------------------------------

/// PropertyMangler run mirroring the full `Minifier` ordering (collect ->
/// rename_annotated_literals -> rewrite), but WITHOUT compress, so the two-pass
/// annotated-literal handling is exercised deterministically.
fn mangle_pipeline(src: &str, regex: &str, quoted: bool) -> String {
    let alloc = Allocator::default();
    let mut program = Parser::new(&alloc, src, SourceType::mjs()).parse().program;
    let mut o = opts(regex);
    o.mangle_quoted = quoted;
    let mut m = PropertyMangler::new(o);
    m.collect(&program);
    m.rename_annotated_literals(&mut program, &alloc);
    m.rewrite(&mut program, &alloc);
    Codegen::new().build(&program).code
}

#[test]
fn unquote_member_not_double_renamed() {
    // The source property `e` forces the generator to skip `e`: `_a` -> `t`, `e` -> `n`.
    // Un-quoting `x['_a']` to `x.t` writes a name outside the map, so a re-visit could
    // never rename it again.
    let got = mangle_quoted("x.e; x['_a'];", "^(_a|e)$");
    let want = codegen("x.n; x.t;", SourceType::mjs());
    assert_eq!(got, want, "\nexpect {want}\ngot {got}");
}

#[test]
fn unquote_object_key_not_double_renamed() {
    // Un-quoting `'_a'` (-> `t`) as an object key writes a name outside the map; the
    // sibling source key `e` renames to `n`, never colliding with the fresh `t`.
    let got = mangle_quoted("({ e: 1, '_a': 2 });", "^(_a|e)$");
    let want = codegen("({ n: 1, t: 2 });", SourceType::mjs());
    assert_eq!(got, want, "\nexpect {want}\ngot {got}");
}

#[test]
fn annotated_literal_not_double_renamed() {
    // `rename_annotated_literals` renames `'_a'` (-> `t`) pre-compress; the value at that
    // span is now a name outside the map, so the later `rewrite` pass leaves it alone.
    let got = mangle_pipeline("f(/* @__KEY__ */ '_a'); x.e;", "^(_a|e)$", false);
    let want = codegen("f(/* @__KEY__ */ 't'); x.n;", SourceType::mjs());
    assert_eq!(got, want, "\nexpect {want}\ngot {got}");
}

#[test]
fn compress_unquoted_annotated_key_not_double_renamed() {
    // Through the FULL pipeline: `assign` maps `_a` -> `t` and `e` -> `n` (the generator
    // skips the source property name `e`). The pre-compress annotated pass renames
    // `'_a'` -> `'t'`; compress un-quotes `x['t']` -> `x.t`; the post-mangle rewrite then
    // sees `t`, which is not a key in the map, and leaves it alone — so the annotated key
    // is renamed exactly once and the two source properties keep DISTINCT final names.
    test_min("o.e = 1; x[/* @__KEY__ */ '_a'] = 2;", "o.n = 1; x.t = 2;", "^[e_]");
}

#[test]
fn annotated_key_and_sibling_share_name() {
    // `assign` maps `_x` -> `t` and `e` -> `n` (skipping the source name `e`). The annotated
    // computed key is renamed once (pre-compress) to `t`; the in-place `rename_key_expression`
    // path of the later rewrite finds `t` outside the map and leaves it alone, so the key and
    // its unquoted sibling `o._x` end up with the SAME name, distinct from `e`'s new name.
    let got = mangle_pipeline("o[(0, /* @__KEY__ */ '_x')]; o._x; o.e;", "^[_e]", true);
    let want = codegen("o[(0, /* @__KEY__ */ 't')]; o.t; o.n;", SourceType::mjs());
    assert_eq!(got, want, "\nexpect {want}\ngot {got}");
}

#[test]
fn unquote_avoids_reserved_nonmatching_member() {
    // Only `_a` matches `^_`; the seen member `e` is a NON-matching property, so it is reserved
    // program-wide. `_a` must mangle to `t` (avoiding the reserved `e`), and `x.e` stays `x.e`.
    let got = mangle_quoted("x.e; x['_a'];", "^_");
    let want = codegen("x.e; x.t;", SourceType::mjs());
    assert_eq!(got, want, "\nexpect {want}\ngot {got}");
}

// ---------------------------------------------------------------------------
// A TRAILING `/* @__KEY__ */` comment has `attached_to == 0` (trailing attachment is
// never computed), so it must not falsely annotate a literal that happens to start at
// offset 0.
// ---------------------------------------------------------------------------

#[test]
fn trailing_key_annotation_does_not_annotate_offset_zero() {
    // The program-leading `'_x'` is at span.start == 0. A TRAILING `@__KEY__` (attached_to == 0)
    // must NOT mark it as a property name: it stays a reserved quoted `in`-LHS string.
    let src = "'_x' in o ? f() : g();\nh(); /* @__KEY__ */\nq();";
    let got = mangle(src, "^_", SourceType::mjs());
    assert!(got.contains("\"_x\" in o"), "`_x` must survive: {got}");
    assert!(!got.contains("\"e\" in"), "`_x` must not be mangled: {got}");
}

#[test]
fn leading_key_annotation_still_renames() {
    // Control: a genuine LEADING `@__KEY__` still marks the string as a property name.
    let got = mangle("f(/* @__KEY__ */ '_x');", "^_", SourceType::mjs());
    assert!(got.contains("\"e\""), "annotated `_x` should mangle to `e`: {got}");
    assert!(!got.contains("_x"), "{got}");
}

// ---------------------------------------------------------------------------
// Numeric property keys: `obj['0']` / `obj[0]` / `{ 0: 1 }` all address the SAME
// property, so a numeric-looking string must be reserved (never mangled), and a
// numeric key/index must reserve its canonical JS string spelling.
// ---------------------------------------------------------------------------

/// Full `Minifier` (compress on) with `mangle_quoted` enabled.
fn minify_with_props_quoted(src: &str, regex: &str) -> String {
    let alloc = Allocator::default();
    let mut program = Parser::new(&alloc, src, SourceType::mjs()).parse().program;
    let mut o = opts(regex);
    o.mangle_quoted = true;
    let options = MinifierOptions {
        mangle: None,
        compress: Some(CompressOptions::default()),
        mangle_properties: Some(o),
    };
    let ret = Minifier::new(options).minify(&alloc, &mut program);
    Codegen::new().with_scoping(ret.scoping).build(&program).code
}

#[test]
fn numeric_string_reserved_vs_numeric_index() {
    // `x['0']` aliases `x[0]`, so `'0'` must be reserved even under `mangle_quoted`.
    let got = mangle_quoted("x['0'] = 1; y = x[0];", "^0$");
    let want = codegen("x['0'] = 1; y = x[0];", SourceType::mjs());
    assert_eq!(got, want, "\nexpect {want}\ngot {got}");
}

#[test]
fn numeric_in_operator_and_index_reserved() {
    let got = mangle_quoted("'0' in x; y = x[0];", "^0$");
    let want = codegen("'0' in x; y = x[0];", SourceType::mjs());
    assert_eq!(got, want, "\nexpect {want}\ngot {got}");
}

#[test]
fn numeric_key_uses_js_spelling() {
    // `{ 1e21: 1 }` addresses property "1e+21" (JS ToString), so the quoted `x['1e+21']`
    // aliases it and must be reserved; neither is mangled even with `.` (mangle everything).
    let got = mangle_quoted("x = { 1e21: 1 }; y = x['1e+21'];", ".");
    let want = codegen("x = { 1e21: 1 }; y = x['1e+21'];", SourceType::mjs());
    assert_eq!(got, want, "\nexpect {want}\ngot {got}");
}

#[test]
fn non_numeric_quoted_still_mangles() {
    // Control: a normal (non-numeric) quoted key is still mangled under `mangle_quoted`.
    let got = mangle_quoted("x['_a'];", "^_");
    let want = codegen("x.e;", SourceType::mjs());
    assert_eq!(got, want, "\nexpect {want}\ngot {got}");
}

#[test]
fn numeric_reserved_through_full_minify() {
    // Through the full pipeline (compress un-quotes accesses), the numeric alias `'0'`/`[0]`
    // must stay reserved. `globalThis` keeps the reads observable so they survive DCE.
    let got =
        minify_with_props_quoted("'0' in globalThis.x; globalThis.y = globalThis.x[0];", "^0$");
    assert!(!got.contains("\"e\""), "numeric `'0'` must not be mangled: {got}");
    assert!(got.contains("\"0\" in"), "the `in`-LHS \"0\" must survive: {got}");
    assert!(got.contains("[0]"), "numeric index must survive: {got}");
}

// ---------------------------------------------------------------------------
// JSX member expressions (`<ns._comp/>`): the property is a props key, so the
// collector must reserve it (esbuild-style), keeping the plain-JS `ns._comp` write
// consistent with the JSX usage.
// ---------------------------------------------------------------------------

#[test]
fn jsx_member_expression_reserved() {
    let st = SourceType::jsx();
    let src = "ns._comp = X; export const a = <ns._comp/>;";
    let got = mangle(src, "^_", st);
    let want = codegen(src, st);
    assert_eq!(got, want, "\nJSX member property must be reserved\nexpect {want}\ngot {got}");
    assert!(got.contains("_comp"), "`_comp` must survive: {got}");
}

#[test]
fn jsx_member_reserved_but_other_prop_mangles() {
    let st = SourceType::jsx();
    // `_comp` is reserved (JSX member) so its plain-JS write stays; the unrelated `_other`
    // member is the only candidate and still mangles to `e`.
    let src = "ns._comp = X; o._other; export const a = <ns._comp/>;";
    let got = mangle(src, "^_", st);
    assert!(got.contains("ns._comp = X"), "`_comp` write must be reserved: {got}");
    assert!(!got.contains("_other"), "`_other` should be mangled: {got}");
    assert!(got.contains("o.e"), "`_other` should mangle to `e`: {got}");
}
