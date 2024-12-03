use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{ReplaceGlobalDefines, ReplaceGlobalDefinesConfig};

use crate::codegen;

pub(crate) fn test(source_text: &str, expected: &str, config: ReplaceGlobalDefinesConfig) {
    let source_type = SourceType::default();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let mut program = ret.program;
    let (symbols, scopes) =
        SemanticBuilder::new().build(&program).semantic.into_symbol_table_and_scope_tree();
    let _ = ReplaceGlobalDefines::new(&allocator, config).build(symbols, scopes, &mut program);
    let result = CodeGenerator::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&program)
        .code;
    let expected = codegen(expected, source_type);
    assert_eq!(result, expected, "for source {source_text}");
}

fn test_same(source_text: &str, config: ReplaceGlobalDefinesConfig) {
    test(source_text, source_text, config);
}

#[test]
fn simple() {
    let config = ReplaceGlobalDefinesConfig::new(&[("id", "text"), ("str", "'text'")]).unwrap();
    test("id, str", "text, 'text'", config);
}

#[test]
fn shadowed() {
    let config = ReplaceGlobalDefinesConfig::new(&[
        ("undefined", "text"),
        ("NaN", "'text'"),
        ("process.env.NODE_ENV", "'test'"),
    ])
    .unwrap();
    test_same("(function (undefined) { let x = typeof undefined })()", config.clone());
    test_same("(function (NaN) { let x = typeof NaN })()", config.clone());
    test_same("(function (process) { let x = process.env.NODE_ENV })()", config.clone());
}

#[test]
fn dot() {
    let config =
        ReplaceGlobalDefinesConfig::new(&[("process.env.NODE_ENV", "production")]).unwrap();
    test("process.env.NODE_ENV", "production", config.clone());
    test("process.env", "process.env", config.clone());
    test("process.env.foo.bar", "process.env.foo.bar", config.clone());
    test("process", "process", config.clone());

    // computed member expression
    test("process['env'].NODE_ENV", "production", config.clone());
}

#[ignore]
#[test]
fn dot_with_overlap() {
    let config = ReplaceGlobalDefinesConfig::new(&[("import.meta.env.FOO", "import.meta.env.FOO"), ("import.meta.env", "__foo__")]).unwrap();
    test("import.meta.env", "__foo__", config.clone());
    test("import.meta.env.NODE_ENV", "import.meta.env.NODE_ENV", config.clone());
}

#[test]
fn dot_nested() {
    let config = ReplaceGlobalDefinesConfig::new(&[("process", "production")]).unwrap();
    test("foo.process.NODE_ENV", "foo.process.NODE_ENV", config.clone());
    // computed member expression
    test("foo['process'].NODE_ENV", "foo['process'].NODE_ENV", config);
}

#[test]
fn dot_with_postfix_wildcard() {
    let config = ReplaceGlobalDefinesConfig::new(&[("import.meta.env.*", "undefined")]).unwrap();
    test("import.meta.env.result", "undefined", config.clone());
    test("import.meta.env", "import.meta.env", config);
}

#[test]
fn dot_with_postfix_mixed() {
    let config = ReplaceGlobalDefinesConfig::new(&[
        ("import.meta.env.*", "undefined"),
        ("import.meta.env", "env"),
        ("import.meta.*", "metaProperty"),
        ("import.meta", "1"),
    ])
    .unwrap();
    test("import.meta.env.result", "undefined", config.clone());
    test("import.meta.env.result.many.nested", "undefined", config.clone());
    test("import.meta.env", "env", config.clone());
    test("import.meta.somethingelse", "metaProperty", config.clone());
    test("import.meta", "1", config);
}

#[test]
fn optional_chain() {
    let config = ReplaceGlobalDefinesConfig::new(&[("a.b.c", "1")]).unwrap();
    test("a.b.c", "1", config.clone());
    test("a?.b.c", "1", config.clone());
    test("a.b?.c", "1", config.clone());
    test("a['b']['c']", "1", config.clone());
    test("a?.['b']['c']", "1", config.clone());
    test("a['b']?.['c']", "1", config.clone());

    test_same("a[b][c]", config.clone());
    test_same("a?.[b][c]", config.clone());
    test_same("a[b]?.[c]", config.clone());
}

#[test]
fn dot_define_with_destruct() {
    let config = ReplaceGlobalDefinesConfig::new(&[(
        "process.env.NODE_ENV",
        "{'a': 1, b: 2, c: true, d: {a: b}}",
    )])
    .unwrap();
    test(
        "const {a, c} = process.env.NODE_ENV",
        "const { a, c } = {\n\t'a': 1,\n\tc: true};",
        config.clone(),
    );
    // bailout
    test(
        "const {[any]: alias} = process.env.NODE_ENV",
        "const { [any]: alias } = {\n\t'a': 1,\n\tb: 2,\n\tc: true,\n\td: { a: b }\n};",
        config.clone(),
    );

    // should filterout unused key even rhs objectExpr has SpreadElement

    let config = ReplaceGlobalDefinesConfig::new(&[(
        "process.env.NODE_ENV",
        "{'a': 1, b: 2, c: true, ...unknown}",
    )])
    .unwrap();
    test(
        "const {a} = process.env.NODE_ENV",
        "const { a } = {\n\t'a': 1,\n\t...unknown\n};\n",
        config.clone(),
    );
}

#[test]
fn this_expr() {
    let config =
        ReplaceGlobalDefinesConfig::new(&[("this", "1"), ("this.foo", "2"), ("this.foo.bar", "3")])
            .unwrap();
    test(
        "this, this.foo, this.foo.bar, this.foo.baz, this.bar",
        "1, 2, 3, 2 .baz, 1 .bar;\n",
        config.clone(),
    );

    test(
        r"
// This code should be the same as above
(() => {
	ok(
		this,
		this.foo,
		this.foo.bar,
		this.foo.baz,
		this.bar,
	);
})();
    ",
        "(() => {\n\tok(1, 2, 3, 2 .baz, 1 .bar);\n})();\n",
        config.clone(),
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
        config,
    );
}

#[test]
fn assignment_target() {
    let config = ReplaceGlobalDefinesConfig::new(&[
        ("d", "ident"),
        ("e.f", "ident"),
        ("g", "dot.chain"),
        ("h.i", "dot.chain"),
    ])
    .unwrap();

    test(
        r"
console.log(
	[a = 0, b.c = 0, b['c'] = 0],
	[d = 0, e.f = 0, e['f'] = 0],
	[g = 0, h.i = 0, h['i'] = 0],
)
        ",
        "console.log([a = 0,b.c = 0,b['c'] = 0], [ident = 0,ident = 0,ident = 0], [dot.chain = 0,dot.chain = 0,dot.chain = 0\n]);",
        config.clone(),
    );
}
