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
