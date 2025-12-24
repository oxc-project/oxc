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
    let source_type = SourceType::ts();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    assert!(ret.errors.is_empty());
    let mut program = ret.program;
    let mut scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    let ret = ReplaceGlobalDefines::new(&allocator, config.clone()).build(scoping, &mut program);
    assert_eq!(ret.changed, source_text != expected);
    // Use the updated scoping, instead of recreating one.
    scoping = ret.scoping;
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
fn optional_chain() {
    let config = config(&[("a.b.c", "1")]);
    test("foo(a.b.c)", "foo(1)", &config);
    test("foo(a?.b.c)", "foo(1)", &config);
    test("foo(a.b?.c)", "foo(1)", &config);
    test("foo(a['b']['c'])", "foo(1)", &config);
    test("foo(a?.['b']['c'])", "foo(1)", &config);
    test("foo(a['b']?.['c'])", "foo(1)", &config);

    test_same("a[b][c]", &config);
    test_same("a?.[b][c]", &config);
    test_same("a[b]?.[c]", &config);
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
        // This code should be the same as above
        (() => { ok( this, this.foo, this.foo.bar, this.foo.baz, this.bar,) })();
    ",
        "
        // This code should be the same as above
        ok(1, 2, 3, 2 .baz, 1 .bar);",
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
