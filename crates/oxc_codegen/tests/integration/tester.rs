use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions, CodegenReturn};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;

pub fn default_options() -> CodegenOptions {
    // Ensure sourcemap do not crash.
    CodegenOptions { source_map_path: Some("test.js".into()), ..CodegenOptions::default() }
}

#[track_caller]
pub fn test_with_parse_options(source_text: &str, expected: &str, parse_options: ParseOptions) {
    let allocator = Allocator::default();
    let ret =
        Parser::new(&allocator, source_text, SourceType::tsx()).with_options(parse_options).parse();
    assert!(ret.errors.is_empty());
    let result = Codegen::new().with_options(default_options()).build(&ret.program).code;
    assert_eq!(result, expected, "\nfor source: {source_text}");
}

#[track_caller]
pub fn test(source_text: &str, expected: &str) {
    test_options(source_text, expected, default_options());
}

#[track_caller]
pub fn test_same(source_text: &str) {
    test(source_text, source_text);
}

#[track_caller]
pub fn test_options(source_text: &str, expected: &str, options: CodegenOptions) {
    test_options_with_source_type(source_text, expected, SourceType::tsx(), options);
}

#[track_caller]
pub fn test_tsx(source_text: &str, expected: &str) {
    test_options_with_source_type(source_text, expected, SourceType::tsx(), default_options());
}

#[track_caller]
pub fn test_options_with_source_type(
    source_text: &str,
    expected: &str,
    source_type: SourceType,
    options: CodegenOptions,
) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    assert!(ret.errors.is_empty(), "Parse errors: {:?}", ret.errors);
    let result = Codegen::new().with_options(options).build(&ret.program).code;
    assert_eq!(result, expected, "\nfor source: {source_text:?}");
}

#[track_caller]
pub fn test_same_ignore_parse_errors(source_text: &str) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, SourceType::tsx()).parse();
    let result = Codegen::new().with_options(default_options()).build(&ret.program).code;
    assert_eq!(result, source_text, "\nfor source: {source_text:?}");
}

#[track_caller]
pub fn test_minify(source_text: &str, expected: &str) {
    let source_type = SourceType::jsx();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    assert!(ret.errors.is_empty(), "Parse errors: {:?}", ret.errors);
    let result = Codegen::new()
        .with_options(CodegenOptions { minify: true, ..CodegenOptions::default() })
        .build(&ret.program)
        .code;
    assert_eq!(result, expected, "\nfor minify source: {source_text}");
}

#[track_caller]
pub fn test_minify_same(source_text: &str) {
    test_minify(source_text, source_text);
}

#[track_caller]
pub fn codegen(source_text: &str) -> String {
    codegen_options(source_text, &default_options()).code
}

#[track_caller]
pub fn codegen_options(source_text: &str, options: &CodegenOptions) -> CodegenReturn {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    assert!(ret.errors.is_empty(), "Parse errors: {:?}", ret.errors);
    let mut options = options.clone();
    options.single_quote = true;
    Codegen::new().with_options(options).build(&ret.program)
}

#[track_caller]
pub fn snapshot(name: &str, cases: &[&str]) {
    snapshot_options(name, cases, &default_options());
}

#[track_caller]
pub fn snapshot_options(name: &str, cases: &[&str], options: &CodegenOptions) {
    use std::fmt::Write;

    let snapshot = cases.iter().enumerate().fold(String::new(), |mut w, (i, case)| {
        let result = codegen_options(case, options).code;
        write!(w, "########## {i}\n{case}\n----------\n{result}\n",).unwrap();
        w
    });

    insta::with_settings!({ prepend_module_to_snapshot => false, snapshot_suffix => "", omit_expression => true }, {
        insta::assert_snapshot!(name, snapshot);
    });
}
