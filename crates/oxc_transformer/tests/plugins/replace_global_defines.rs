use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{ReplaceGlobalDefines, ReplaceGlobalDefinesConfig};

use super::run;

pub(crate) fn test(source_text: &str, expected: &str, config: ReplaceGlobalDefinesConfig) {
    let source_type = SourceType::default();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let program = allocator.alloc(ret.program);
    let (symbols, scopes) = SemanticBuilder::new(source_text)
        .build(program)
        .semantic
        .into_symbol_table_and_scope_tree();
    let _ = ReplaceGlobalDefines::new(&allocator, config).build(symbols, scopes, program);
    let result = CodeGenerator::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(program)
        .source_text;
    let expected = run(expected, source_type);
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
    test("process", "process", config);
}

#[test]
fn dot_nested() {
    let config = ReplaceGlobalDefinesConfig::new(&[("process", "production")]).unwrap();
    test("foo.process.NODE_ENV", "foo.process.NODE_ENV", config);
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
