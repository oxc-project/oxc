use oxc_allocator::Allocator;
use oxc_ast::ast::StringLiteral;
use oxc_ast_visit::{Visit, walk};
use oxc_codegen::Codegen;
use oxc_minifier::{
    CompressOptions, ManglePropertiesOptions, ManglePropertyCache, Minifier, MinifierOptions,
    PropertyKeyOrigin, PropertyKeyProvenance, PropertyMangler,
};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

fn options(pattern: &str) -> ManglePropertiesOptions {
    ManglePropertiesOptions::new(lazy_regex::Regex::new(pattern).unwrap())
}

fn codegen(source: &str, source_type: SourceType) -> String {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    Codegen::new().build(&parsed.program).code
}

fn mangle_with(
    source: &str,
    source_type: SourceType,
    options: ManglePropertiesOptions,
    provenance: Option<&PropertyKeyProvenance>,
) -> (String, ManglePropertyCache) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut program = parsed.program;
    let mut mangler = PropertyMangler::new(options);
    mangler.collect(&program, provenance);
    mangler.assign();
    mangler.rewrite(&mut program, &allocator, provenance);
    let code = Codegen::new().build(&program).code;
    (code, mangler.into_cache())
}

#[track_caller]
fn test(source: &str, expected: &str, options: ManglePropertiesOptions) {
    let source_type = SourceType::mjs();
    let (actual, _) = mangle_with(source, source_type, options, None);
    assert_eq!(actual, codegen(expected, source_type), "source: {source}");
}

#[test]
fn rewrites_runtime_property_positions() {
    test(
        "let _foo; ({ _foo }); const { _foo: local } = obj; ({ _foo } = obj); class C { _foo() {} } obj._foo;",
        "let _foo; ({ e: _foo }); const { e: local } = obj; ({ e: _foo } = obj); class C { e() {} } obj.e;",
        options("^_"),
    );
}

#[test]
fn quoted_behavior_is_per_occurrence() {
    test(
        "obj._foo; obj['_foo']; ({ _foo: 1, '_foo': 2 });",
        "obj.e; obj['_foo']; ({ e: 1, '_foo': 2 });",
        options("^_"),
    );

    let mut quoted = options("^_");
    quoted.mangle_quoted = true;
    test(
        "obj._foo; obj['_foo']; ({ _foo: 1, '_foo': 2 }); '_foo' in obj;",
        "obj.e; obj.e; ({ e: 1, e: 2 }); 'e' in obj;",
        quoted,
    );

    test(
        "obj._field; obj[x ? '_field' : y]; obj?.[x, '_field']; (x ? '_field' : y) in obj;",
        "obj.e; obj[x ? '_field' : y]; obj?.[x, '_field']; (x ? '_field' : y) in obj;",
        options("^_"),
    );

    let mut wrapped_quoted = options("^_");
    wrapped_quoted.mangle_quoted = true;
    test(
        "obj[x ? '_field' : y]; obj?.[x, '_field']; ({ [(x, '_field')]: 1 }); (x ? '_field' : y) in obj;",
        "obj[x ? 'e' : y]; obj?.[x, 'e']; ({ [(x, 'e')]: 1 }); (x ? 'e' : y) in obj;",
        wrapped_quoted,
    );
}

#[test]
fn optional_chain_properties_are_rewritten_consistently() {
    test(
        "x._foo; x._foo?.(); x?._foo; x?._foo(); x?._foo._bar; x?._foo._bar();",
        "x.e; x.e?.(); x?.e; x?.e(); x?.e.t; x?.e.t();",
        options("^_"),
    );
}

#[test]
fn hard_reserved_constructor_keeps_super_calls_valid() {
    test(
        "class Base {} class Derived extends Base { constructor() { super(); } _method() { return super._method(); } }",
        "class Base {} class Derived extends Base { constructor() { super(); } e() { return super.e(); } }",
        options("."),
    );
}

#[test]
fn module_and_commonjs_member_positions_are_rewritten() {
    test(
        "export const value = namespace._field; export { value as _binding };",
        "export const value = namespace.e; export { value as _binding };",
        options("^_"),
    );

    let source_type = SourceType::cjs();
    let (actual, _) = mangle_with(
        "exports._field = require('pkg')._other; let _local = 1;",
        source_type,
        options("^_"),
        None,
    );
    assert_eq!(actual, codegen("exports.e = require('pkg').t; let _local = 1;", source_type));
}

#[test]
fn automatic_names_avoid_source_property_spellings() {
    test("obj.e; obj.t; obj._foo; obj._bar;", "obj.e; obj.t; obj.r; obj.n;", options("^_"));
}

#[test]
fn annotations_override_quoted_behavior() {
    test(
        "obj._foo; helper(/* @__KEY__ */ '_foo'); helper(/* #__KEY__ */ `_foo`); helper('_foo');",
        "obj.e; helper(/* @__KEY__ */ 'e'); helper(/* #__KEY__ */ `e`); helper('_foo');",
        options("^_"),
    );
}

#[test]
fn exact_reservations_do_not_grow_into_a_denylist() {
    test(
        "obj.__proto__; obj.constructor; obj.prototype; obj.then; obj.toJSON;",
        "obj.__proto__; obj.constructor; obj.prototype; obj.e; obj.t;",
        options("."),
    );
}

#[test]
fn frequency_controls_assignment() {
    test("a._often; b._rare; c._often; d._often;", "a.e; b.t; c.e; d.e;", options("^_"));
}

#[test]
fn cache_is_authoritative_and_duplicate_targets_are_allowed() {
    let mut options = options("^_");
    options.cache.insert("_first".into(), Some("A".into())).unwrap();
    options.cache.insert("_second".into(), Some("A".into())).unwrap();
    options.cache.insert("_keep".into(), None).unwrap();
    let (actual, cache) = mangle_with(
        "obj._first; obj._second; obj._keep; obj._automatic;",
        SourceType::mjs(),
        options,
        None,
    );
    assert_eq!(actual, codegen("obj.A; obj.A; obj._keep; obj.e;", SourceType::mjs()));
    assert_eq!(cache["_first"].as_deref(), Some("A"));
    assert_eq!(cache["_second"].as_deref(), Some("A"));
    assert_eq!(cache["_keep"], None);
    assert_eq!(cache["_automatic"].as_deref(), Some("e"));
}

#[test]
fn non_matching_cache_entries_are_inert_and_preserved() {
    let mut options = options("^_");
    options.cache.insert("public".into(), Some("A".into())).unwrap();
    let (actual, cache) =
        mangle_with("obj.public; obj._private;", SourceType::mjs(), options, None);
    assert_eq!(actual, codegen("obj.public; obj.e;", SourceType::mjs()));
    assert_eq!(cache["public"].as_deref(), Some("A"));
}

#[test]
fn automatic_names_avoid_unchanged_and_cached_spellings() {
    let mut options = options("^_");
    options.cache.insert("unused".into(), Some("e".into())).unwrap();
    test("obj._foo; obj['t'];", "obj.n; obj['t'];", options);
}

#[test]
fn cache_keys_occupy_the_automatic_output_namespace_across_calls() {
    let mut first_options = options(".");
    first_options.cache.insert("e".into(), None).unwrap();
    let (first_output, cache) = mangle_with("obj.foo;", SourceType::mjs(), first_options, None);
    assert_eq!(first_output, codegen("obj.t;", SourceType::mjs()));
    assert_eq!(cache["e"], None);
    assert_eq!(cache["foo"].as_deref(), Some("t"));

    let mut second_options = options(".");
    second_options.cache = cache;
    test("obj.e; obj.foo;", "obj.e; obj.t;", second_options);

    let mut inert_cache_key = options("^_");
    inert_cache_key.cache.insert("e".into(), Some("A".into())).unwrap();
    test("obj._foo;", "obj.t;", inert_cache_key);
}

#[test]
fn jsx_properties_are_rewritten_but_namespaces_are_not() {
    let source_type = SourceType::jsx();
    let (actual, _) = mangle_with(
        "const x = <Components._Widget _prop={1} ns:_keep={2} />;",
        source_type,
        options("^_"),
        None,
    );
    assert_eq!(actual, codegen("const x = <Components.e t={1} ns:_keep={2} />;", source_type));
}

#[test]
fn debug_names_are_readable() {
    let mut readable = options("^_");
    readable.debug = true;
    test(
        "obj._alpha; obj._beta; obj._gamma;",
        "obj._$_alpha$_; obj._$_beta$_; obj._$_gamma$_;",
        readable,
    );

    let mut collision = options("^_");
    collision.debug = true;
    test("obj['_$_field$_']; obj._field;", "obj['_$_field$_']; obj._$_field$1$_;", collision);

    let mut non_identifier = options(".");
    non_identifier.debug = true;
    non_identifier.mangle_quoted = true;
    test("obj['not-valid'];", "obj._$property0$_;", non_identifier);
}

#[test]
fn assignment_is_deterministic_for_equal_frequencies() {
    let (_, forward) =
        mangle_with("obj._alpha; obj._beta; obj._gamma;", SourceType::mjs(), options("^_"), None);
    let (_, reverse) =
        mangle_with("obj._gamma; obj._beta; obj._alpha;", SourceType::mjs(), options("^_"), None);
    assert_eq!(forward, reverse);
}

#[test]
fn property_mangling_is_idempotent_when_outputs_do_not_match_include() {
    let (once, _) = mangle_with("obj._field;", SourceType::mjs(), options("^_"), None);
    let (twice, second_cache) = mangle_with(&once, SourceType::mjs(), options("^_"), None);
    assert_eq!(twice, once);
    assert!(second_cache.is_empty());
}

#[test]
fn reserved_names_are_not_automatic_outputs() {
    let mut options = options("^_");
    options.reserved.insert("e".into());
    test("obj._field;", "obj.t;", options);
}

#[test]
fn numeric_spellings_and_template_keys_are_never_mangled() {
    let mut options = options(".");
    options.mangle_quoted = true;
    test(
        "obj['0']; obj[0]; obj[`template`]; obj.regular;",
        "obj['0']; obj[0]; obj[`template`]; obj.e;",
        options,
    );
}

#[test]
fn eval_function_and_with_do_not_disable_property_mangling() {
    let source_type = SourceType::script();
    let source = "eval('dynamic'); Function('return 0'); with (obj) obj._field;";
    let (actual, _) = mangle_with(source, source_type, options("^_"), None);
    assert_eq!(
        actual,
        codegen("eval('dynamic'); Function('return 0'); with (obj) obj.e;", source_type)
    );
}

#[test]
fn rewrite_happens_before_compression_erases_quotes() {
    let allocator = Allocator::default();
    let parsed =
        Parser::new(&allocator, "globalThis._foo; globalThis['_foo'];", SourceType::mjs()).parse();
    let mut program = parsed.program;
    let result = Minifier::new(MinifierOptions {
        mangle: None,
        mangle_properties: Some(options("^_")),
        compress: Some(CompressOptions::default()),
    })
    .minify(&allocator, &mut program);
    let actual = Codegen::new().with_scoping(result.scoping).build(&program).code;
    assert!(actual.contains("globalThis.e"), "{actual}");
    assert!(actual.contains("globalThis._foo"), "{actual}");
}

#[test]
fn compression_does_not_revisit_folded_computed_keys() {
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        "export const o = { ['f' + 'oo_']: 1 }; export const v = o.foo_;",
        SourceType::mjs(),
    )
    .parse();
    let mut program = parsed.program;
    let result = Minifier::new(MinifierOptions {
        mangle: None,
        mangle_properties: Some(options("_$")),
        compress: Some(CompressOptions::default()),
    })
    .minify(&allocator, &mut program);
    let actual = Codegen::new().with_scoping(result.scoping).build(&program).code;
    assert_eq!(
        actual,
        codegen("export const o = { foo_: 1 }; export const v = o.e;", SourceType::mjs())
    );
}

#[derive(Default)]
struct StringSpans(Vec<Span>);

impl<'a> Visit<'a> for StringSpans {
    fn visit_string_literal(&mut self, literal: &StringLiteral<'a>) {
        self.0.push(literal.span);
        walk::walk_string_literal(self, literal);
    }
}

#[test]
fn provenance_preserves_original_quote_class() {
    let source = "helper('_unquoted'); helper('_quoted'); obj._unquoted; obj._quoted;";
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    let mut spans = StringSpans::default();
    spans.visit_program(&parsed.program);
    assert_eq!(spans.0.len(), 2);

    let provenance = PropertyKeyProvenance::from_iter([
        (spans.0[0], PropertyKeyOrigin::Unquoted),
        (spans.0[1], PropertyKeyOrigin::Quoted),
    ]);
    let (actual, _) = mangle_with(source, SourceType::mjs(), options("^_"), Some(&provenance));
    assert_eq!(actual, codegen("helper('e'); helper('_quoted'); obj.e; obj.t;", SourceType::mjs()));
}

#[test]
fn one_assignment_can_be_shared_across_programs() {
    let allocator_a = Allocator::default();
    let allocator_b = Allocator::default();
    let mut program_a =
        Parser::new(&allocator_a, "a._shared; a._local;", SourceType::mjs()).parse().program;
    let mut program_b =
        Parser::new(&allocator_b, "b._shared; b._shared; b['_quoted'];", SourceType::mjs())
            .parse()
            .program;

    let mut mangler = PropertyMangler::new(options("^_"));
    mangler.collect(&program_a, None);
    mangler.collect(&program_b, None);
    mangler.assign();
    mangler.rewrite(&mut program_a, &allocator_a, None);
    mangler.rewrite(&mut program_b, &allocator_b, None);

    assert_eq!(Codegen::new().build(&program_a).code, codegen("a.e; a.t;", SourceType::mjs()));
    assert_eq!(
        Codegen::new().build(&program_b).code,
        codegen("b.e; b.e; b['_quoted'];", SourceType::mjs())
    );
}
