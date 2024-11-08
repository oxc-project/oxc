use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{
    ReplaceGlobalDefines, ReplaceGlobalDefinesConfig, ReplaceGlobalDefinesReturn,
};

use crate::codegen;

/// `semantic_info_changed` is used to assert that the semantic information has changed.
pub(crate) fn test(
    source_text: &str,
    expected: &str,
    config: ReplaceGlobalDefinesConfig,
    expect_semantic_info_changed: bool,
) {
    let source_type = SourceType::default();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let mut program = ret.program;
    let (symbols, scopes) =
        SemanticBuilder::new().build(&program).semantic.into_symbol_table_and_scope_tree();
    let ReplaceGlobalDefinesReturn { changed, .. } =
        ReplaceGlobalDefines::new(&allocator, config).build(symbols, scopes, &mut program);
    let result = CodeGenerator::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&program)
        .code;
    let expected = codegen(expected, source_type);
    assert_eq!(result, expected, "for source {source_text}");
    assert_eq!(changed, expect_semantic_info_changed, "for source {source_text}");
}

fn test_same(source_text: &str, config: ReplaceGlobalDefinesConfig) {
    test(source_text, source_text, config, false);
}

#[test]
fn simple() {
    let config = ReplaceGlobalDefinesConfig::new(&[("id", "text"), ("str", "'text'")]).unwrap();
    test("id, str", "text, 'text'", config, true);
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
    test("process.env.NODE_ENV", "production", config.clone(), true);
    test("process.env", "process.env", config.clone(), false);
    test("process.env.foo.bar", "process.env.foo.bar", config.clone(), false);
    test("process", "process", config, false);
}

#[test]
fn dot_nested() {
    let config = ReplaceGlobalDefinesConfig::new(&[("process", "production")]).unwrap();
    test("foo.process.NODE_ENV", "foo.process.NODE_ENV", config, false);
}

#[test]
fn dot_with_postfix_wildcard() {
    let config = ReplaceGlobalDefinesConfig::new(&[("import.meta.env.*", "undefined")]).unwrap();
    test("import.meta.env.result", "undefined", config.clone(), true);
    test("import.meta.env", "import.meta.env", config, false);
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
    test("import.meta.env.result", "undefined", config.clone(), true);
    test("import.meta.env.result.many.nested", "undefined", config.clone(), true);
    test("import.meta.env", "env", config.clone(), true);
    test("import.meta.somethingelse", "metaProperty", config.clone(), true);
    test("import.meta", "1", config, true);
}
