use oxc_allocator::Allocator;
use oxc_ast_visit::VisitMut;
use oxc_codegen::{Codegen, CodegenOptions, CommentOptions};
use oxc_minifier::{
    CompressOptions, MangleOptions, ManglePropertiesOptions, ManglePropertyCache, Minifier,
    MinifierOptions, PropertyMangleCollection, PropertyMangler,
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
) -> (String, ManglePropertyCache) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut program = parsed.program;
    let mut mangler = PropertyMangler::new(options);
    mangler.collect(&program);
    mangler.assign();
    mangler.rewrite(&mut program, &allocator);
    let code = Codegen::new().build(&program).code;
    (code, mangler.into_cache())
}

#[track_caller]
fn test(source: &str, expected: &str, options: ManglePropertiesOptions) {
    let source_type = SourceType::mjs();
    let (actual, _) = mangle_with(source, source_type, options);
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
fn mangle_quoted_rewrites_no_substitution_template_keys() {
    let mut quoted = options("^_");
    quoted.mangle_quoted = true;
    test(
        "obj[`_foo`] = 1; obj._foo = 2; `_foo` in obj; ({ [`_foo`]: 3 }); obj[`${suffix}_foo`]; use(`_foo`);",
        "obj[`e`] = 1; obj.e = 2; `e` in obj; ({ [`e`]: 3 }); obj[`${suffix}_foo`]; use(`_foo`);",
        quoted,
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
fn property_key_annotations_survive_codegen_before_mangling() {
    let source = "const object = { _field: 1 }; const key = /* #__KEY__ */ '_field'; object[key];";
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let rendered = Codegen::new()
        .with_options(CodegenOptions {
            comments: CommentOptions { normal: false, ..CommentOptions::default() },
            ..CodegenOptions::default()
        })
        .build(&parsed.program)
        .code;
    assert!(rendered.contains("/* #__KEY__ */"));

    let (actual, _) = mangle_with(&rendered, SourceType::mjs(), options("^_"));
    assert_eq!(
        actual,
        codegen(
            "const object = { e: 1 }; const key = /* #__KEY__ */ 'e'; object[key];",
            SourceType::mjs(),
        )
    );
}

#[test]
fn key_annotations_do_not_rewrite_directives() {
    let source_type = SourceType::mjs();
    let (actual, cache) =
        mangle_with("/* @__KEY__ */ '_directive'; obj._field;", source_type, options("^_"));
    assert_eq!(actual, codegen("/* @__KEY__ */ '_directive'; obj.e;", source_type));
    assert!(cache.get("_directive").is_none());
    assert_eq!(cache.get("_field").and_then(Option::as_deref), Some("e"));
}

#[test]
fn property_key_annotation_without_a_literal_does_not_annotate_offset_zero() {
    let source = "'_x' in obj ? yes() : no();\nwork(); /* @__KEY__ */\ndone();";
    test(source, source, options("^_"));
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
fn exclude_keeps_selected_names_unchanged() {
    let mut excluded = options("^_");
    excluded.exclude = Some(lazy_regex::Regex::new("^_keep(?:One|Two)$").unwrap());
    test("obj._keepOne; obj._mangle;", "obj._keepOne; obj.e;", excluded);
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
    );
    assert_eq!(actual, codegen("obj.A; obj.A; obj._keep; obj.e;", SourceType::mjs()));
    assert_eq!(cache.get("_first").and_then(Option::as_deref), Some("A"));
    assert_eq!(cache.get("_second").and_then(Option::as_deref), Some("A"));
    assert_eq!(cache.get("_keep"), Some(&None));
    assert_eq!(cache.get("_automatic").and_then(Option::as_deref), Some("e"));
}

#[test]
fn cache_targets_are_not_remapped_during_rewrite() {
    let mut options = options("^_");
    options.mangle_quoted = true;
    options.cache.insert("_first".into(), Some("_second".into())).unwrap();
    options.cache.insert("_second".into(), Some("final".into())).unwrap();
    test(
        "obj._first; obj['_first']; ({ '_first': 1 }); helper(/* @__KEY__ */ '_first'); obj._second;",
        "obj._second; obj._second; ({ _second: 1 }); helper(/* @__KEY__ */ '_second'); obj.final;",
        options,
    );
}

#[test]
fn cache_cycles_are_applied_once() {
    let mut options = options(".");
    options.cache.insert("a".into(), Some("b".into())).unwrap();
    options.cache.insert("b".into(), Some("a".into())).unwrap();
    test("obj.a; obj.b;", "obj.b; obj.a;", options);
}

#[test]
fn non_matching_cache_entries_are_inert_and_preserved() {
    let mut options = options("^_");
    options.cache.insert("public".into(), Some("A".into())).unwrap();
    let (actual, cache) = mangle_with("obj.public; obj._private;", SourceType::mjs(), options);
    assert_eq!(actual, codegen("obj.public; obj.e;", SourceType::mjs()));
    assert_eq!(cache.get("public").and_then(Option::as_deref), Some("A"));
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
    let (first_output, cache) = mangle_with("obj.foo;", SourceType::mjs(), first_options);
    assert_eq!(first_output, codegen("obj.t;", SourceType::mjs()));
    assert_eq!(cache.get("e"), Some(&None));
    assert_eq!(cache.get("foo").and_then(Option::as_deref), Some("t"));

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
    );
    assert_eq!(actual, codegen("const x = <Components.e t={1} ns:_keep={2} />;", source_type));
}

#[test]
fn typescript_type_space_properties_are_not_rewritten() {
    let source_type = SourceType::ts();
    let (actual, _) = mangle_with(
        "interface Shape { _field: string; _method(): void } const object = { _field: 1, _method() {} }; object._field; object._method();",
        source_type,
        options("^_"),
    );
    assert_eq!(
        actual,
        codegen(
            "interface Shape { _field: string; _method(): void } const object = { e: 1, t() {} }; object.e; object.t();",
            source_type,
        )
    );
}

#[test]
fn rewrites_defaults_class_fields_and_accessors() {
    test(
        "let { _field: local = init } = obj; ({ _field: target = init } = obj); class C { _field = 1; accessor _field = 2; get _field() { return this._field; } }",
        "let { e: local = init } = obj; ({ e: target = init } = obj); class C { e = 1; accessor e = 2; get e() { return this.e; } }",
        options("^_"),
    );
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
        mangle_with("obj._alpha; obj._beta; obj._gamma;", SourceType::mjs(), options("^_"));
    let (_, reverse) =
        mangle_with("obj._gamma; obj._beta; obj._alpha;", SourceType::mjs(), options("^_"));
    assert_eq!(forward, reverse);
}

#[test]
fn property_mangling_is_idempotent_when_outputs_do_not_match_include() {
    let (once, _) = mangle_with("obj._field;", SourceType::mjs(), options("^_"));
    let (twice, second_cache) = mangle_with(&once, SourceType::mjs(), options("^_"));
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
fn numeric_spellings_are_never_mangled_and_template_keys_follow_quoted() {
    let mut options = options(".");
    options.mangle_quoted = true;
    test(
        "obj['0']; obj[0]; obj[`template`]; obj.regular;",
        "obj['0']; obj[0]; obj[`t`]; obj.e;",
        options,
    );
}

#[test]
fn numeric_keys_use_javascript_string_spelling() {
    let mut quoted = options(".");
    quoted.mangle_quoted = true;
    let source = "const obj = { 1e21: 1 }; obj['1e+21'];";
    test(source, source, quoted);
}

#[test]
fn eval_function_and_with_do_not_disable_property_mangling() {
    let source_type = SourceType::script();
    let source = "eval('dynamic'); Function('return 0'); with (obj) obj._field;";
    let (actual, _) = mangle_with(source, source_type, options("^_"));
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
fn property_identifier_and_compression_pipeline_runs_together() {
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        "export function read(_object) { const _local = _object; return _local._field + _local._field; }",
        SourceType::mjs(),
    )
    .parse();
    let mut program = parsed.program;
    let result = Minifier::new(MinifierOptions {
        mangle: Some(MangleOptions::default()),
        mangle_properties: Some(options("^_")),
        compress: Some(CompressOptions::default()),
    })
    .minify(&allocator, &mut program);
    let actual = Codegen::new().with_scoping(result.scoping).build(&program).code;
    assert!(!actual.contains("_field"), "{actual}");
    assert!(!actual.contains("_local"), "{actual}");
    assert!(actual.matches(".e").count() >= 2, "{actual}");
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

#[test]
fn automatic_names_avoid_keys_materialized_by_compression() {
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        "export function f(obj) { const key = 'e'; return obj[key] + obj._foo + obj._foo; }",
        SourceType::mjs(),
    )
    .parse();
    let mut program = parsed.program;
    let result = Minifier::new(MinifierOptions {
        mangle: None,
        mangle_properties: Some(options("^_")),
        compress: Some(CompressOptions::default()),
    })
    .minify(&allocator, &mut program);
    let actual = Codegen::new().with_scoping(result.scoping).build(&program).code;
    assert_eq!(
        actual,
        codegen("export function f(obj) { return obj.e + obj.t + obj.t; }", SourceType::mjs())
    );
}

#[derive(Default)]
struct ZeroSpans;

impl VisitMut<'_> for ZeroSpans {
    fn visit_span(&mut self, span: &mut Span) {
        *span = Span::default();
    }
}

#[test]
fn duplicated_spans_do_not_skip_property_rewrites() {
    let allocator = Allocator::default();
    let parsed =
        Parser::new(&allocator, "x = { _a: 1, _b: 2 }; use_(x._a, x._b);", SourceType::mjs())
            .parse();
    let mut program = parsed.program;
    ZeroSpans.visit_program(&mut program);

    let mut mangler = PropertyMangler::new(options("^_"));
    mangler.collect(&program);
    mangler.assign();
    mangler.rewrite(&mut program, &allocator);

    assert_eq!(
        Codegen::new().build(&program).code,
        codegen("x = { e: 1, t: 2 }; use_(x.e, x.t);", SourceType::mjs())
    );
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
    mangler.collect(&program_a);
    mangler.collect(&program_b);
    mangler.assign();
    mangler.rewrite(&mut program_a, &allocator_a);
    mangler.rewrite(&mut program_b, &allocator_b);

    assert_eq!(Codegen::new().build(&program_a).code, codegen("a.e; a.t;", SourceType::mjs()));
    assert_eq!(
        Codegen::new().build(&program_b).code,
        codegen("b.e; b.e; b['_quoted'];", SourceType::mjs())
    );
}

#[test]
fn independently_collected_programs_can_be_merged_before_assignment() {
    let allocator_a = Allocator::default();
    let allocator_b = Allocator::default();
    let mut program_a =
        Parser::new(&allocator_a, "a._shared; a._local;", SourceType::mjs()).parse().program;
    let mut program_b =
        Parser::new(&allocator_b, "b._shared; b._shared; b['_quoted'];", SourceType::mjs())
            .parse()
            .program;

    let options = options("^_");
    let collected_a = PropertyMangleCollection::from_program(&options, &program_a);
    let collected_b = PropertyMangleCollection::from_program(&options, &program_b);

    let mut mangler = PropertyMangler::new(options);
    mangler.merge_collected(collected_a);
    mangler.merge_collected(collected_b);
    mangler.assign();
    mangler.rewrite(&mut program_a, &allocator_a);
    mangler.rewrite(&mut program_b, &allocator_b);

    assert_eq!(Codegen::new().build(&program_a).code, codegen("a.e; a.t;", SourceType::mjs()));
    assert_eq!(
        Codegen::new().build(&program_b).code,
        codegen("b.e; b.e; b['_quoted'];", SourceType::mjs())
    );
}
