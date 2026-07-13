use oxc_allocator::Allocator;
use oxc_ast_visit::Visit;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_minifier::{CompressOptions, Compressor};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer_plugins::{ReplaceGlobalDefines, ReplaceGlobalDefinesConfig};

use crate::codegen;

#[track_caller]
pub fn test(source_text: &str, expected: &str, config: &ReplaceGlobalDefinesConfig) {
    let source_type = SourceType::ts().with_module(true);
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    assert!(ret.diagnostics.is_empty());
    let mut program = ret.program;
    let mut scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    let ret = ReplaceGlobalDefines::new(&allocator, config.clone()).build(scoping, &mut program);
    assert_eq!(ret.changed, source_text != expected);
    // Mirror the pipeline in crates/oxc/src/compiler.rs: it treats scoping as
    // dirty whenever ReplaceGlobalDefines changed the AST (`scoping_dirty |=
    // ret.changed`) and rebuilds it before DCE. Reuse RGD's scoping only on
    // the unchanged path.
    scoping = if ret.changed {
        SemanticBuilder::new().build(&program).semantic.into_scoping()
    } else {
        ret.scoping
    };
    AssertAst.visit_program(&program);
    // Run DCE, to align pipeline in crates/oxc/src/compiler.rs
    Compressor::new(&allocator).dead_code_elimination_with_scoping(
        &mut program,
        scoping,
        CompressOptions::smallest(),
    );
    let result = Codegen::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&program)
        .code;
    let expected = codegen(expected, source_type);
    assert_eq!(result, expected, "for source {source_text}");
}

#[track_caller]
fn test_define_only(source_text: &str, expected: &str, config: &ReplaceGlobalDefinesConfig) {
    let source_type = SourceType::ts().with_module(true);
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    assert!(ret.diagnostics.is_empty());
    let mut program = ret.program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    let ret = ReplaceGlobalDefines::new(&allocator, config.clone()).build(scoping, &mut program);
    assert_eq!(ret.changed, source_text != expected);
    AssertAst.visit_program(&program);
    let result = Codegen::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&program)
        .code;
    let expected = codegen(expected, source_type);
    assert_eq!(result, expected, "for source {source_text}");
}

#[track_caller]
fn test_same(source_text: &str, config: &ReplaceGlobalDefinesConfig) {
    test(source_text, source_text, config);
}

struct AssertAst;

impl Visit<'_> for AssertAst {
    fn visit_identifier_reference(&mut self, ident: &oxc_ast::ast::IdentifierReference<'_>) {
        assert!(ident.reference_id.get().is_some());
    }
}

fn config<S: AsRef<str>>(defines: &[(S, S)]) -> ReplaceGlobalDefinesConfig {
    ReplaceGlobalDefinesConfig::new(defines).unwrap()
}

#[test]
fn simple() {
    let config = config(&[("id", "text"), ("str", "'text'")]);
    test("foo([id, str])", "foo([text, 'text'])", &config);
}

#[test]
fn shadowed() {
    let config =
        config(&[("undefined", "text"), ("NaN", "'text'"), ("process.env.NODE_ENV", "'test'")]);
    test_same("(function (undefined) { foo(typeof undefined) })()", &config);
    test_same("(function (NaN) { foo(typeof NaN) })()", &config);
    test_same("(function (process) { foo(process.env.NODE_ENV) })()", &config);
}

#[test]
fn typeof_define() {
    let config = config(&[
        ("typeof window", "'undefined'"),
        ("typeof process.env", "'object'"),
        ("typeof import.meta", "'object'"),
        ("typeof import.meta.env", "'object'"),
    ]);

    test_define_only(
        "foo(typeof window, typeof (window), typeof process.env, typeof (process).env, typeof process['env'], typeof (process)['env'], typeof process?.env, typeof import.meta, typeof import.meta.env)",
        "foo('undefined', 'undefined', 'object', 'object', 'object', 'object', 'object', 'object', 'object')",
        &config,
    );
    test("if (typeof window === 'undefined') server(); else client();", "server()", &config);
    test(
        "if (typeof window !== 'undefined') import('./client'); else import('./server');",
        "import('./server')",
        &config,
    );
}

#[test]
fn typeof_define_is_exact() {
    let config = config(&[("typeof window", "'undefined'"), ("typeof process.env", "'object'")]);

    test_same("foo(typeof window.document)", &config);
    test_same("foo(typeof process)", &config);
    test_same("foo(typeof process.env.NODE_ENV)", &config);
}

#[test]
fn typeof_define_is_scope_aware() {
    let config =
        config(&[("typeof window", "'undefined'"), ("typeof window.document", "'object'")]);

    test_define_only(
        "foo(typeof window, typeof window.document); function f(window) { foo(typeof window, typeof window.document) }",
        "foo('undefined', 'object'); function f(window) { foo(typeof window, typeof window.document) }",
        &config,
    );
    test_define_only(
        "const window = {}; foo(typeof window)",
        "const window = {}; foo(typeof window)",
        &config,
    );
    test_define_only("foo(typeof window); var window", "foo(typeof window); var window", &config);
    test_define_only(
        "try {} catch (window) { foo(typeof window) }",
        "try {} catch (window) { foo(typeof window) }",
        &config,
    );
    test_define_only(
        "import window from 'window'; foo(typeof window)",
        "import window from 'window'; foo(typeof window)",
        &config,
    );
    test_define_only(
        "declare const window: Window; foo(typeof window)",
        "declare const window: Window; foo('undefined')",
        &config,
    );
    test_define_only(
        "type WindowType = typeof window; foo(typeof window)",
        "type WindowType = typeof window; foo('undefined')",
        &config,
    );
}

#[test]
fn typeof_define_this_expr() {
    let config = config(&[("typeof this", "'object'")]);

    test_define_only("foo(typeof this)", "foo('object')", &config);
    test_define_only("(() => foo(typeof this))()", "(() => foo('object'))()", &config);
    test_define_only(
        "function f() { foo(typeof this); (() => foo(typeof this))() }",
        "function f() { foo(typeof this); (() => foo(typeof this))() }",
        &config,
    );
    test_define_only(
        "class C { field = typeof this; static field = typeof this; static { foo(typeof this) } }",
        "class C { field = typeof this; static field = typeof this; static { foo(typeof this) } }",
        &config,
    );
    test_define_only(
        "class C { [typeof this] = typeof this }",
        "class C { ['object'] = typeof this }",
        &config,
    );
}

#[test]
fn typeof_define_takes_precedence_over_identifier_define() {
    let config = config(&[("window", "globalThis"), ("typeof window", "'undefined'")]);
    test_define_only("foo(window, typeof window)", "foo(globalThis, 'undefined')", &config);
}

#[test]
fn invalid_typeof_define_key() {
    for key in ["typeof ", "typeof  window", "typeof window.*", "typeof window[0]"] {
        assert!(ReplaceGlobalDefinesConfig::new(&[(key, "'undefined'")]).is_err(), "{key}");
    }
}

#[test]
fn dot() {
    let config = config(&[("process.env.NODE_ENV", "production")]);
    test("foo(process.env.NODE_ENV)", "foo(production)", &config);
    test("foo(process.env)", "foo(process.env)", &config);
    test("foo(process.env.foo.bar)", "foo(process.env.foo.bar)", &config);
    test("foo(process)", "foo(process)", &config);

    // computed member expression
    test("foo(process['env'].NODE_ENV)", "foo(production)", &config);
}

#[test]
fn dot_with_partial_overlap() {
    let config = config(&[("import.meta.foo.bar", "1")]);
    test("console.log(import.meta.bar)", "console.log(import.meta.bar)", &config);
    test("console.log(import.meta.foo)", "console.log(import.meta.foo)", &config);
    test("console.log(import.meta.foo.bar)", "console.log(1)", &config);
}

#[test]
fn dot_with_overlap() {
    let config =
        config(&[("import.meta.env.FOO", "import.meta.env.BAR"), ("import.meta.env", "__foo__")]);
    test("const _ = import.meta.env", "__foo__", &config);
    test("const _ = import.meta.env.FOO", "import.meta.env.BAR", &config);
    test("const _ = import.meta.env.NODE_ENV", "__foo__.NODE_ENV", &config);

    test("_ = import.meta.env", "_ = __foo__", &config);
    test("_ = import.meta.env.FOO", "_ = import.meta.env.BAR", &config);
    test("_ = import.meta.env.NODE_ENV", "_ = __foo__.NODE_ENV", &config);

    test("import.meta.env = 0", "__foo__ = 0", &config);
    test("import.meta.env.NODE_ENV = 0", "__foo__.NODE_ENV = 0", &config);
    test("import.meta.env.FOO = 0", "import.meta.env.BAR = 0", &config);
}

#[test]
fn dot_define_is_member_expr_postfix() {
    let config = config(&[
        ("__OBJ__", r#"{"process":{"env":{"SOMEVAR":"foo"}}}"#),
        ("process.env.SOMEVAR", "\"SOMEVAR\""),
    ]);
    test(
        "console.log(__OBJ__.process.env.SOMEVAR)",
        "console.log({ 'process': { 'env': { 'SOMEVAR': 'foo' } } }.process.env.SOMEVAR);\n",
        &config,
    );
}

#[test]
fn dot_nested() {
    let config = config(&[("process", "production")]);
    test("foo.process.NODE_ENV", "foo.process.NODE_ENV", &config);
    // computed member expression
    test("foo['process'].NODE_ENV", "foo['process'].NODE_ENV", &config);
}

#[test]
fn dot_with_postfix_wildcard() {
    let config = config(&[("import.meta.env.*", "undefined")]);
    test("foo(import.meta.env.result)", "foo(void 0)", &config);
    test("foo(import.meta.env)", "foo(import.meta.env)", &config);
}

#[test]
fn dot_with_postfix_mixed() {
    let config = config(&[
        ("import.meta.env.*", "undefined"),
        ("import.meta.env", "env"),
        ("import.meta.*", "metaProperty"),
        ("import.meta", "1"),
    ]);
    test("foo(import.meta.env.result)", "foo(void 0)", &config);
    test("foo(import.meta.env.result.many.nested)", "foo((void 0).many.nested)", &config);
    test("foo(import.meta.env)", "foo(env)", &config);
    test("foo(import.meta.somethingelse)", "foo(metaProperty)", &config);
    test("foo(import.meta.somethingelse.nested.one)", "foo(metaProperty.nested.one)", &config);
    test("foo(import.meta)", "foo(1)", &config);
}

#[test]
fn meta_property_not_new_target() {
    // `new.target` is also a `MetaProperty`, but an `import.meta.*` define must only match
    // `import.meta`, never `new.target`.
    let config = config(&[("import.meta.env", "__foo__"), ("import.meta.env.*", "undefined")]);
    test(
        "export function f() { return new.target.env; }",
        "export function f() { return new.target.env; }",
        &config,
    );
    test(
        "export function f() { return new.target.env.FOO; }",
        "export function f() { return new.target.env.FOO; }",
        &config,
    );
    // sanity: the same config still rewrites the real `import.meta`
    test("const _ = import.meta.env", "__foo__", &config);
}

// ---- adversarial: cases exercising the trailing-name buckets (this PR's data structure) ----

#[test]
fn bucket_multiple_dot_defines() {
    // Two distinct chains share the trailing name `X` -> one bucket, len > 1.
    // `find` must pick the entry whose full chain matches, not just the first in the bucket.
    let c = config(&[("a.b.X", "1"), ("c.d.X", "2")]);
    test("foo(a.b.X)", "foo(1)", &c);
    test("foo(c.d.X)", "foo(2)", &c);
    test_same("foo(e.f.X)", &c);

    // Shorter vs longer chain in the same bucket.
    let c = config(&[("b.X", "10"), ("a.b.X", "20")]);
    test("foo(b.X)", "foo(10)", &c);
    test("foo(a.b.X)", "foo(20)", &c);
}

#[test]
fn bucket_multiple_meta_defines() {
    // Two meta defines share the trailing name `X`.
    let config = config(&[("import.meta.a.X", "1"), ("import.meta.b.X", "2")]);
    test("foo(import.meta.a.X)", "foo(1)", &config);
    test("foo(import.meta.b.X)", "foo(2)", &config);
    test_same("foo(import.meta.c.X)", &config);
}

#[test]
fn dot_and_meta_share_trailing_name() {
    // A dot define and a meta define bucket under the same name `env`.
    // The dot map is consulted first, then the meta map; neither must steal the other's match.
    let config = config(&[("app.env", "1"), ("import.meta.env", "2")]);
    test("foo(app.env)", "foo(1)", &config);
    test("foo(import.meta.env)", "foo(2)", &config);
    test_same("foo(other.env)", &config);
}

#[test]
fn duplicate_keys_keep_first() {
    // Identifier duplicate.
    let c = config(&[("DUP", "1"), ("DUP", "2")]);
    test("foo(DUP)", "foo(1)", &c);
    // Dot duplicate (same chain twice -> two entries in one bucket).
    let c = config(&[("a.b.DUP", "1"), ("a.b.DUP", "2")]);
    test("foo(a.b.DUP)", "foo(1)", &c);
}

#[test]
fn specific_meta_beats_wildcard_regardless_of_config_order() {
    // Wildcard listed BEFORE the specific define; the specific must still win, because the
    // keyed map is always consulted before the wildcard list (the old sort is gone).
    let config = config(&[("import.meta.env.*", "0"), ("import.meta.env.MODE", "1")]);
    test("foo(import.meta.env.MODE)", "foo(1)", &config);
    test("foo(import.meta.env.OTHER)", "foo(0)", &config);
}

#[test]
fn computed_key_dispatches_to_bucket() {
    let config = config(&[("a.b.C", "1")]);
    test("foo(a.b['C'])", "foo(1)", &config);
    test("foo(a['b'].C)", "foo(1)", &config);
    // Single-quasi template literal key resolves to a static name too.
    test("foo(a.b[`C`])", "foo(1)", &config);
    // Dynamic key can never match a define.
    test_same("foo(a.b[C])", &config);
}

#[test]
fn global_meta_dot_define_does_not_match_import_meta() {
    // A dot define rooted at a global `meta` shares the bucket key `X` with nothing else, but its
    // chain walks onto a `MetaProperty`. The `import.meta` guard must keep `import.meta.X` from
    // matching a `meta.X` define meant for a global identifier named `meta`.
    let c = config(&[("meta.X", "1")]);
    test("foo(meta.X)", "foo(1)", &c);
    test_same("foo(import.meta.X)", &c);
}

#[test]
fn optional_chain_dispatches_to_bucket() {
    // Optional chaining must still resolve to the right bucket entry.
    let c = config(&[("a.b.X", "1"), ("c.d.X", "2")]);
    test("foo(a?.b.X)", "foo(1)", &c);
    test("foo(c.d?.X)", "foo(2)", &c);
}

#[test]
fn assignment_target_dispatches_to_bucket() {
    // The no-optimize assignment path shares the same bucket lookup.
    let c = config(&[("a.b.X", "lhs1"), ("c.d.X", "lhs2")]);
    test("a.b.X = 0", "lhs1 = 0", &c);
    test("c.d.X = 0", "lhs2 = 0", &c);
}

#[test]
fn optional_chain() {
    let config = config(&[("a.b.c", "1"), ("process.env", "{}")]);
    test("foo(a.b.c)", "foo(1)", &config);
    test("foo(a?.b.c)", "foo(1)", &config);
    test("foo(a.b?.c)", "foo(1)", &config);
    test("foo(a['b']['c'])", "foo(1)", &config);
    test("foo(a?.['b']['c'])", "foo(1)", &config);
    test("foo(a['b']?.['c'])", "foo(1)", &config);

    // `process?.env` replaced by `{}`, ChainExpression unwrapped since no optional markers remain.
    test("process?.env[0]", "({})[0]", &config);

    // Chains where optional markers remain should NOT be unwrapped.
    test_same("a?.[b][c]", &config);
    test_same("a[b]?.[c]", &config);
    test_same("a[b][c]", &config);
}

#[test]
fn dot_define_with_destruct() {
    let c = config(&[("process.env.NODE_ENV", "{'a': 1, b: 2, c: true, d: {a: b}}")]);
    test("const {a, c} = process.env.NODE_ENV", "const { a, c } = {\n\t'a': 1,\n\tc: true};", &c);
    // bailout
    test(
        "const {[any]: alias} = process.env.NODE_ENV",
        "const { [any]: alias } = {\n\t'a': 1,\n\tb: 2,\n\tc: true,\n\td: { a: b }\n};",
        &c,
    );

    // should filterout unused key even rhs objectExpr has SpreadElement

    let c = config(&[("process.env.NODE_ENV", "{'a': 1, b: 2, c: true, ...unknown}")]);
    test("const {a} = process.env.NODE_ENV", "const { a } = {\n\t'a': 1,\n\t...unknown\n};\n", &c);
}

#[test]
fn this_expr() {
    let config = config(&[("this", "1"), ("this.foo", "2"), ("this.foo.bar", "3")]);
    test(
        "f(this); f(this.foo), f(this.foo.bar), f(this.foo.baz), f(this.bar)",
        "f(1), f(2), f(3), f(2 .baz), f(1 .bar)",
        &config,
    );

    test(
        r"
        // The IIFE wrapper is preserved under DCE-only mode — inlining IIFE
        // bodies is a peephole rewrite, not DCE. See oxc_minifier PR #22547.
        (() => { ok( this, this.foo, this.foo.bar, this.foo.baz, this.bar,) })();
    ",
        "
        // The IIFE wrapper is preserved under DCE-only mode — inlining IIFE
        // bodies is a peephole rewrite, not DCE. See oxc_minifier PR #22547.
        (() => { ok(1, 2, 3, 2 .baz, 1 .bar); })();",
        &config,
    );

    test_same(
        r"
// Nothing should be substituted in this code
(function() {
	doNotSubstitute(
		this,
		this.foo,
		this.foo.bar,
		this.foo.baz,
		this.bar,
	);
})();
    ",
        &config,
    );

    test_define_only(
        "class C { field = this; static field = this.foo; static { foo(this, this.foo) } }",
        "class C { field = this; static field = this.foo; static { foo(this, this.foo) } }",
        &config,
    );
}

#[test]
fn this_expr_nested_functions() {
    let config = config(&[("this", "1"), ("this.foo", "2")]);

    // Arrow inside regular function: `this` captures function's `this`, should NOT be replaced.
    test_same("export function foo() { return () => this.foo }", &config);

    // Nested arrows without enclosing function: `this` captures module-level `this`, SHOULD be replaced.
    test("export const f = () => () => this.foo", "export const f = () => () => 2;\n", &config);

    // Arrow inside regular function inside arrow: innermost `this` is function's `this`.
    test_same("export const outer = () => function() { return () => this.foo }", &config);
}

#[test]
fn dot_define_with_destruct_nested() {
    // Destructuring optimization should NOT apply when the define is nested inside a function call
    let c = config(&[("process.env.NODE_ENV", "{'a': 1, b: 2, c: true}")]);
    test(
        "const {a} = foo(process.env.NODE_ENV)",
        "const { a } = foo({\n\t'a': 1,\n\tb: 2,\n\tc: true\n});\n",
        &c,
    );
}

#[test]
fn assignment_target() {
    let config =
        config(&[("d", "ident"), ("e.f", "ident"), ("g", "dot.chain"), ("h.i", "dot.chain")]);

    test(
        r"
console.log(
	[a = 0, b.c = 0, b['c'] = 0],
	[d = 0, e.f = 0, e['f'] = 0],
	[g = 0, h.i = 0, h['i'] = 0],
)
        ",
        "console.log([a = 0,b.c = 0,b['c'] = 0], [ident = 0,ident = 0,ident = 0], [dot.chain = 0,dot.chain = 0,dot.chain = 0\n]);",
        &config,
    );
}

#[test]
fn replace_with_undefined() {
    let c = config(&[("Foo", "undefined")]);
    test("new Foo()", "new (void 0)()", &c);

    let c = config(&[("Foo", "Bar")]);
    test("Foo = 0", "Bar = 0", &c);
}

#[test]
fn declare_const() {
    let config = config(&[("IS_PROD", "true")]);
    test("declare const IS_PROD: boolean; if (IS_PROD) {} foo(IS_PROD)", "foo(true)", &config);
}

#[test]
fn declare_const_assignment_target_bailout() {
    // `IS_PROD = 0` cannot be rewritten to `true = 0` (literals are not valid
    // assignment targets), so the LHS replacement bails out and the original
    // identifier stays in the AST, its reference intact — downstream DCE must
    // not treat IS_PROD as unused and drop the `declare const`.
    let config = config(&[("IS_PROD", "true")]);
    test(
        "declare const IS_PROD: boolean; IS_PROD = 0;",
        "declare const IS_PROD: boolean; IS_PROD = 0;",
        &config,
    );
}

#[test]
fn declare_dot_define() {
    let config = config(&[("process.env.NODE_ENV", "'production'")]);
    test_define_then_transform_ts(
        "declare let process: { env: { NODE_ENV: string } }; foo(process.env.NODE_ENV)",
        "foo('production')",
        &config,
    );
    test_define_only(
        "declare let process: { env: { NODE_ENV: string } }; foo(process.env.NODE_ENV)",
        "declare let process: { env: { NODE_ENV: string } }; foo('production')",
        &config,
    );
}

#[cfg(not(miri))]
#[test]
fn test_sourcemap() {
    use oxc_sourcemap::SourcemapVisualizer;

    let c = config(&[
        ("__OBJECT__", r#"{"hello": "test"}"#),
        ("__STRING__", r#""development""#),
        ("__MEMBER__", r"xx.yy.zz"),
    ]);
    let source_text = r"
1;
__OBJECT__;
2;
__STRING__;
3;
log(__OBJECT__);
4;
log(__STRING__);
5;
__OBJECT__.hello;
6;
log(__MEMBER__);
7;
"
    .trim_start();

    let source_type = SourceType::default();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let mut program = ret.program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    let _ = ReplaceGlobalDefines::new(&allocator, c).build(scoping, &mut program);
    let result = Codegen::new()
        .with_options(CodegenOptions {
            single_quote: true,
            source_map_path: Some(std::path::Path::new(&"test.js.map").to_path_buf()),
            ..CodegenOptions::default()
        })
        .build(&program);

    let output = result.code;
    let output_map = result.map.unwrap();
    let visualizer = SourcemapVisualizer::new(&output, &output_map);
    let snapshot = visualizer.get_text();
    insta::assert_snapshot!("test_sourcemap", snapshot);
}

/// Run ReplaceGlobalDefines then Transformer (like the playground pipeline).
/// This reproduces the panic when define replaces the member expression that
/// carries `optional: true`, leaving a `ChainExpression` with no optional markers.
#[track_caller]
fn test_define_then_transform(
    source_text: &str,
    expected: &str,
    define_config: &ReplaceGlobalDefinesConfig,
) {
    test_define_then_transform_impl(source_text, expected, define_config, SourceType::mjs());
}

#[track_caller]
fn test_define_then_transform_ts(
    source_text: &str,
    expected: &str,
    define_config: &ReplaceGlobalDefinesConfig,
) {
    test_define_then_transform_impl(
        source_text,
        expected,
        define_config,
        SourceType::ts().with_module(true),
    );
}

#[track_caller]
fn test_define_then_transform_impl(
    source_text: &str,
    expected: &str,
    define_config: &ReplaceGlobalDefinesConfig,
    source_type: SourceType,
) {
    use oxc_transformer::{TransformOptions, Transformer};
    use std::path::Path;

    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    assert!(ret.diagnostics.is_empty());
    let mut program = ret.program;

    // Step 1: Run define plugin first (like the playground does)
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    let _ret =
        ReplaceGlobalDefines::new(&allocator, define_config.clone()).build(scoping, &mut program);

    // Step 2: Rebuild semantic for transformer
    let scoping =
        SemanticBuilder::new().with_excess_capacity(2.0).build(&program).semantic.into_scoping();

    // Step 3: Run transformer with ES2019 target (lowers optional chaining)
    let options = TransformOptions::from_target("es2019").unwrap();
    let filename = if source_type.is_typescript() { "test.ts" } else { "test.mjs" };
    let ret = Transformer::new(&allocator, Path::new(filename), &options)
        .build_with_scoping(scoping, &mut program);
    assert!(ret.diagnostics.is_empty());

    let result = Codegen::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&program)
        .code;
    let expected = codegen(expected, source_type);
    assert_eq!(result, expected, "for source {source_text}");
}

#[test]
fn define_then_transform_optional_chain() {
    let c = config(&[("process.env", "{}")]);

    // All optional markers removed → ChainExpression unwrapped, no panic.
    test_define_then_transform("console.log(process?.env[0]);", "console.log({}[0])", &c);
    test_define_then_transform("process?.env[0]", "({})[0]", &c);

    // Optional markers hidden behind TS non-null assertion should still be detected.
    // `process?.env!` — TSNonNullExpression wraps the optional member.
    test_define_then_transform_ts("process?.env!", "({})", &c);

    // Parenthesized expression wrapping an optional chain.
    test_define_then_transform("(process?.env)[0]", "({})[0]", &c);

    // Nested chain: the inner `a?.b` is replaced by `'replaced'`, but outer `?.c` keeps optional.
    let c2 = config(&[("a.b", "'replaced'")]);
    test_define_then_transform(
        "a?.b?.c",
        "var _replaced; (_replaced = 'replaced') === null || _replaced === void 0 ? void 0 : _replaced.c",
        &c2,
    );
}
